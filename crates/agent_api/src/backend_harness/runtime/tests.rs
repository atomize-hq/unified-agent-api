use std::collections::VecDeque;
use std::future::Future;
use std::pin::Pin;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Mutex,
};
use std::task::{Context, Poll};
use std::time::Duration;

use futures_core::Stream;
use futures_util::{task::noop_waker, StreamExt};
use tokio::sync::{mpsc, oneshot};

use super::super::test_support::{toy_kind, ToyAdapter, ToyBackendError, ToyEvent};
use super::*;
use crate::AgentWrapperEventKind;

#[tokio::test]
async fn cancel_signal_is_idempotent_and_supports_late_subscribers() {
    let cancel = HarnessCancelSignal::new();

    let waiter = tokio::spawn({
        let cancel = cancel.clone();
        async move {
            cancel.cancelled().await;
        }
    });

    tokio::task::yield_now().await;

    cancel.cancel();
    cancel.cancel();
    assert!(cancel.is_cancelled());
    waiter.await.expect("waiter observes cancellation");

    cancel.cancelled().await;
}

#[tokio::test]
async fn pump_backend_events_smoke_forwards_in_order() {
    let adapter = std::sync::Arc::new(ToyAdapter { fail_spawn: false });
    let events = futures_util::stream::iter([
        Ok::<ToyEvent, ToyBackendError>(ToyEvent::Text("one".to_string())),
        Ok::<ToyEvent, ToyBackendError>(ToyEvent::Text("two".to_string())),
    ]);
    let events: DynBackendEventStream<_, _> = Box::pin(events);

    let (tx, mut rx) =
        mpsc::channel::<AgentWrapperEvent>(super::super::DEFAULT_EVENT_CHANNEL_CAPACITY);
    let handle = tokio::spawn(pump_backend_events(adapter, events, tx));

    let mut texts = Vec::<String>::new();
    while let Some(ev) = rx.recv().await {
        if ev.kind == AgentWrapperEventKind::TextOutput {
            if let Some(text) = ev.text {
                texts.push(text);
            }
        }
    }

    handle.await.expect("pump task completes");
    assert_eq!(texts, vec!["one".to_string(), "two".to_string()]);
}

#[tokio::test]
async fn pump_blocks_under_backpressure_until_receiver_polls() {
    #[derive(Default)]
    struct BackpressureAdapter {
        call_count: AtomicUsize,
        second_mapped_tx: Mutex<Option<oneshot::Sender<()>>>,
    }

    impl BackendHarnessAdapter for BackpressureAdapter {
        fn kind(&self) -> crate::AgentWrapperKind {
            toy_kind()
        }

        fn supported_extension_keys(&self) -> &'static [&'static str] {
            &[]
        }

        type Policy = ();

        fn validate_and_extract_policy(
            &self,
            _request: &crate::AgentWrapperRunRequest,
        ) -> Result<Self::Policy, crate::AgentWrapperError> {
            Ok(())
        }

        type BackendEvent = String;
        type BackendCompletion = ();
        type BackendError = ();

        fn spawn(
            &self,
            _req: super::super::contract::NormalizedRequest<Self::Policy>,
        ) -> std::pin::Pin<
            Box<
                dyn std::future::Future<
                        Output = Result<
                            super::super::contract::BackendSpawn<
                                Self::BackendEvent,
                                Self::BackendCompletion,
                                Self::BackendError,
                            >,
                            Self::BackendError,
                        >,
                    > + Send
                    + 'static,
            >,
        > {
            panic!("spawn unused in pump tests");
        }

        fn map_event(&self, event: Self::BackendEvent) -> Vec<crate::AgentWrapperEvent> {
            let mapped = self.call_count.fetch_add(1, Ordering::SeqCst);
            if mapped == 1 {
                if let Some(tx) = self.second_mapped_tx.lock().unwrap().take() {
                    let _ = tx.send(());
                }
            }

            vec![crate::AgentWrapperEvent {
                agent_kind: toy_kind(),
                kind: AgentWrapperEventKind::TextOutput,
                channel: Some("assistant".to_string()),
                text: Some(event),
                message: None,
                data: None,
            }]
        }

        fn map_completion(
            &self,
            _completion: Self::BackendCompletion,
        ) -> Result<crate::AgentWrapperCompletion, crate::AgentWrapperError> {
            panic!("map_completion unused in pump tests");
        }

        fn redact_error(
            &self,
            _phase: BackendHarnessErrorPhase,
            _err: &Self::BackendError,
        ) -> String {
            "unused".to_string()
        }
    }

    let (second_mapped_tx, second_mapped_rx) = oneshot::channel::<()>();
    let adapter = std::sync::Arc::new(BackpressureAdapter {
        call_count: AtomicUsize::new(0),
        second_mapped_tx: Mutex::new(Some(second_mapped_tx)),
    });

    let events = futures_util::stream::iter([
        Ok::<String, ()>("one".to_string()),
        Ok::<String, ()>("two".to_string()),
    ]);
    let events: DynBackendEventStream<_, _> = Box::pin(events);

    let (tx, mut rx) = mpsc::channel::<crate::AgentWrapperEvent>(1);
    let handle = tokio::spawn(pump_backend_events(adapter, events, tx));

    second_mapped_rx.await.expect("second event mapped");
    tokio::task::yield_now().await;
    assert!(
        !handle.is_finished(),
        "pump must be blocked on bounded send"
    );

    let mut texts = Vec::<String>::new();
    while let Some(ev) = rx.recv().await {
        if ev.kind == AgentWrapperEventKind::TextOutput {
            if let Some(text) = ev.text {
                texts.push(text);
            }
        }
    }

    handle.await.expect("pump task completes");
    assert_eq!(texts, vec!["one".to_string(), "two".to_string()]);
}

struct CountingStream<E, BE> {
    items: VecDeque<Result<E, BE>>,
    consumed: std::sync::Arc<AtomicUsize>,
}

impl<E, BE> Unpin for CountingStream<E, BE> {}

impl<E, BE> Stream for CountingStream<E, BE> {
    type Item = Result<E, BE>;

    fn poll_next(
        self: Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let this = self.get_mut();
        let next = this.items.pop_front();
        if next.is_some() {
            this.consumed.fetch_add(1, Ordering::SeqCst);
        }
        std::task::Poll::Ready(next)
    }
}

#[tokio::test]
async fn pump_stops_forwarding_after_receiver_drop_but_drains_to_end() {
    struct DrainAdapter;

    impl BackendHarnessAdapter for DrainAdapter {
        fn kind(&self) -> crate::AgentWrapperKind {
            toy_kind()
        }

        fn supported_extension_keys(&self) -> &'static [&'static str] {
            &[]
        }

        type Policy = ();

        fn validate_and_extract_policy(
            &self,
            _request: &crate::AgentWrapperRunRequest,
        ) -> Result<Self::Policy, crate::AgentWrapperError> {
            Ok(())
        }

        type BackendEvent = String;
        type BackendCompletion = ();
        type BackendError = ();

        fn spawn(
            &self,
            _req: super::super::contract::NormalizedRequest<Self::Policy>,
        ) -> std::pin::Pin<
            Box<
                dyn std::future::Future<
                        Output = Result<
                            super::super::contract::BackendSpawn<
                                Self::BackendEvent,
                                Self::BackendCompletion,
                                Self::BackendError,
                            >,
                            Self::BackendError,
                        >,
                    > + Send
                    + 'static,
            >,
        > {
            panic!("spawn unused in pump tests");
        }

        fn map_event(&self, event: Self::BackendEvent) -> Vec<crate::AgentWrapperEvent> {
            vec![crate::AgentWrapperEvent {
                agent_kind: toy_kind(),
                kind: AgentWrapperEventKind::TextOutput,
                channel: Some("assistant".to_string()),
                text: Some(event),
                message: None,
                data: None,
            }]
        }

        fn map_completion(
            &self,
            _completion: Self::BackendCompletion,
        ) -> Result<crate::AgentWrapperCompletion, crate::AgentWrapperError> {
            panic!("map_completion unused in pump tests");
        }

        fn redact_error(
            &self,
            _phase: BackendHarnessErrorPhase,
            _err: &Self::BackendError,
        ) -> String {
            "unused".to_string()
        }
    }

    let total = 20usize;
    let consumed = std::sync::Arc::new(AtomicUsize::new(0));
    let items: VecDeque<Result<String, ()>> = (0..total)
        .map(|i| Ok::<String, ()>(format!("ev-{i}")))
        .collect();

    let events = CountingStream {
        items,
        consumed: consumed.clone(),
    };
    let events: DynBackendEventStream<_, _> = Box::pin(events);

    let adapter = std::sync::Arc::new(DrainAdapter);
    let (tx, mut rx) = mpsc::channel::<crate::AgentWrapperEvent>(1);
    let handle = tokio::spawn(pump_backend_events(adapter, events, tx));

    let first = rx.recv().await.expect("at least one forwarded event");
    assert_eq!(first.kind, AgentWrapperEventKind::TextOutput);
    drop(rx);

    handle.await.expect("pump task completes");
    assert_eq!(
        consumed.load(Ordering::SeqCst),
        total,
        "backend stream must be fully drained after receiver drop"
    );
}

struct GatedCountingStream<E, BE> {
    first: Option<Result<E, BE>>,
    rest: VecDeque<Result<E, BE>>,
    gate_rx: Option<oneshot::Receiver<()>>,
    consumed: std::sync::Arc<AtomicUsize>,
}

impl<E, BE> Unpin for GatedCountingStream<E, BE> {}

impl<E, BE> Stream for GatedCountingStream<E, BE> {
    type Item = Result<E, BE>;

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let this = self.get_mut();
        if let Some(first) = this.first.take() {
            this.consumed.fetch_add(1, Ordering::SeqCst);
            return std::task::Poll::Ready(Some(first));
        }

        if let Some(gate_rx) = &mut this.gate_rx {
            match Pin::new(gate_rx).poll(cx) {
                std::task::Poll::Ready(_) => {
                    this.gate_rx = None;
                }
                std::task::Poll::Pending => {
                    return std::task::Poll::Pending;
                }
            }
        }

        let next = this.rest.pop_front();
        if next.is_some() {
            this.consumed.fetch_add(1, Ordering::SeqCst);
        }
        std::task::Poll::Ready(next)
    }
}

#[tokio::test]
async fn pump_with_cancel_closes_universal_stream_but_still_drains_typed_stream() {
    struct DrainAdapter;

    impl BackendHarnessAdapter for DrainAdapter {
        fn kind(&self) -> crate::AgentWrapperKind {
            toy_kind()
        }

        fn supported_extension_keys(&self) -> &'static [&'static str] {
            &[]
        }

        type Policy = ();

        fn validate_and_extract_policy(
            &self,
            _request: &crate::AgentWrapperRunRequest,
        ) -> Result<Self::Policy, crate::AgentWrapperError> {
            Ok(())
        }

        type BackendEvent = String;
        type BackendCompletion = ();
        type BackendError = ();

        fn spawn(
            &self,
            _req: super::super::contract::NormalizedRequest<Self::Policy>,
        ) -> std::pin::Pin<
            Box<
                dyn std::future::Future<
                        Output = Result<
                            super::super::contract::BackendSpawn<
                                Self::BackendEvent,
                                Self::BackendCompletion,
                                Self::BackendError,
                            >,
                            Self::BackendError,
                        >,
                    > + Send
                    + 'static,
            >,
        > {
            panic!("spawn unused in pump tests");
        }

        fn map_event(&self, event: Self::BackendEvent) -> Vec<crate::AgentWrapperEvent> {
            vec![crate::AgentWrapperEvent {
                agent_kind: toy_kind(),
                kind: AgentWrapperEventKind::TextOutput,
                channel: Some("assistant".to_string()),
                text: Some(event),
                message: None,
                data: None,
            }]
        }

        fn map_completion(
            &self,
            _completion: Self::BackendCompletion,
        ) -> Result<crate::AgentWrapperCompletion, crate::AgentWrapperError> {
            panic!("map_completion unused in pump tests");
        }

        fn redact_error(
            &self,
            _phase: BackendHarnessErrorPhase,
            _err: &Self::BackendError,
        ) -> String {
            "unused".to_string()
        }
    }

    let total = 3usize;
    let consumed = std::sync::Arc::new(AtomicUsize::new(0));
    let (gate_tx, gate_rx) = oneshot::channel::<()>();

    let stream = GatedCountingStream {
        first: Some(Ok::<String, ()>("ev-0".to_string())),
        rest: (1..total)
            .map(|i| Ok::<String, ()>(format!("ev-{i}")))
            .collect(),
        gate_rx: Some(gate_rx),
        consumed: consumed.clone(),
    };
    let events: DynBackendEventStream<_, _> = Box::pin(stream);

    let adapter = std::sync::Arc::new(DrainAdapter);
    let (tx, mut rx) = mpsc::channel::<crate::AgentWrapperEvent>(8);
    let cancel = HarnessCancelSignal::new();
    let handle = tokio::spawn(pump_backend_events_with_cancel(
        adapter,
        events,
        tx,
        cancel.clone(),
    ));

    let first = rx.recv().await.expect("at least one forwarded event");
    assert_eq!(first.kind, AgentWrapperEventKind::TextOutput);

    cancel.cancel();

    assert!(
        tokio::time::timeout(Duration::from_secs(1), rx.recv())
            .await
            .expect("recv does not hang")
            .is_none(),
        "universal stream must close after cancellation"
    );

    let _ = gate_tx.send(());
    handle.await.expect("pump task completes");
    assert_eq!(
        consumed.load(Ordering::SeqCst),
        total,
        "typed backend stream must be drained to end after cancellation"
    );
}

#[tokio::test]
async fn pump_with_cancel_preserves_drain_on_drop_posture() {
    struct DrainAdapter;

    impl BackendHarnessAdapter for DrainAdapter {
        fn kind(&self) -> crate::AgentWrapperKind {
            toy_kind()
        }

        fn supported_extension_keys(&self) -> &'static [&'static str] {
            &[]
        }

        type Policy = ();

        fn validate_and_extract_policy(
            &self,
            _request: &crate::AgentWrapperRunRequest,
        ) -> Result<Self::Policy, crate::AgentWrapperError> {
            Ok(())
        }

        type BackendEvent = String;
        type BackendCompletion = ();
        type BackendError = ();

        fn spawn(
            &self,
            _req: super::super::contract::NormalizedRequest<Self::Policy>,
        ) -> std::pin::Pin<
            Box<
                dyn std::future::Future<
                        Output = Result<
                            super::super::contract::BackendSpawn<
                                Self::BackendEvent,
                                Self::BackendCompletion,
                                Self::BackendError,
                            >,
                            Self::BackendError,
                        >,
                    > + Send
                    + 'static,
            >,
        > {
            panic!("spawn unused in pump tests");
        }

        fn map_event(&self, event: Self::BackendEvent) -> Vec<crate::AgentWrapperEvent> {
            vec![crate::AgentWrapperEvent {
                agent_kind: toy_kind(),
                kind: AgentWrapperEventKind::TextOutput,
                channel: Some("assistant".to_string()),
                text: Some(event),
                message: None,
                data: None,
            }]
        }

        fn map_completion(
            &self,
            _completion: Self::BackendCompletion,
        ) -> Result<crate::AgentWrapperCompletion, crate::AgentWrapperError> {
            panic!("map_completion unused in pump tests");
        }

        fn redact_error(
            &self,
            _phase: BackendHarnessErrorPhase,
            _err: &Self::BackendError,
        ) -> String {
            "unused".to_string()
        }
    }

    let total = 20usize;
    let consumed = std::sync::Arc::new(AtomicUsize::new(0));
    let items: VecDeque<Result<String, ()>> = (0..total)
        .map(|i| Ok::<String, ()>(format!("ev-{i}")))
        .collect();

    let events = CountingStream {
        items,
        consumed: consumed.clone(),
    };
    let events: DynBackendEventStream<_, _> = Box::pin(events);

    let adapter = std::sync::Arc::new(DrainAdapter);
    let (tx, mut rx) = mpsc::channel::<crate::AgentWrapperEvent>(1);
    let cancel = HarnessCancelSignal::new();
    let handle = tokio::spawn(pump_backend_events_with_cancel(adapter, events, tx, cancel));

    let first = rx.recv().await.expect("at least one forwarded event");
    assert_eq!(first.kind, AgentWrapperEventKind::TextOutput);
    drop(rx);

    handle.await.expect("pump task completes");
    assert_eq!(
        consumed.load(Ordering::SeqCst),
        total,
        "backend stream must be fully drained after receiver drop"
    );
}

#[tokio::test]
async fn completion_sender_selects_cancelled_error_but_waits_for_backend_exit() {
    let adapter = std::sync::Arc::new(ToyAdapter { fail_spawn: false });
    let cancel = HarnessCancelSignal::new();

    let term_calls = std::sync::Arc::new(AtomicUsize::new(0));
    let request_termination: RequestTerminationHook = {
        let term_calls = term_calls.clone();
        std::sync::Arc::new(move || {
            term_calls.fetch_add(1, Ordering::SeqCst);
        })
    };

    let (backend_done_tx, backend_done_rx) = oneshot::channel::<()>();
    let completion: DynBackendCompletionFuture<_, _> = Box::pin(async move {
        backend_done_rx.await.expect("backend completion released");
        Ok::<super::super::test_support::ToyCompletion, ToyBackendError>(
            super::super::test_support::ToyCompletion,
        )
    });

    let (completion_tx, mut completion_rx) =
        oneshot::channel::<Result<AgentWrapperCompletion, AgentWrapperError>>();

    let task = tokio::spawn(send_completion_with_cancel(
        adapter,
        completion,
        cancel.clone(),
        Some(request_termination),
        completion_tx,
    ));

    cancel.cancel();

    let waker = noop_waker();
    let mut cx = Context::from_waker(&waker);
    assert!(matches!(
        Pin::new(&mut completion_rx).poll(&mut cx),
        Poll::Pending
    ));

    let _ = backend_done_tx.send(());
    let outcome = completion_rx.await.expect("completion sent");
    assert!(
        matches!(
            outcome,
            Err(AgentWrapperError::Backend { ref message }) if message == "cancelled"
        ),
        "completion must resolve to the pinned cancelled error"
    );
    assert_eq!(term_calls.load(Ordering::SeqCst), 1);

    task.await.expect("completion task completes");
}

#[tokio::test]
async fn completion_sender_preserves_backend_outcome_when_completion_wins() {
    let adapter = std::sync::Arc::new(ToyAdapter { fail_spawn: false });
    let cancel = HarnessCancelSignal::new();

    let term_calls = std::sync::Arc::new(AtomicUsize::new(0));
    let request_termination: RequestTerminationHook = {
        let term_calls = term_calls.clone();
        std::sync::Arc::new(move || {
            term_calls.fetch_add(1, Ordering::SeqCst);
        })
    };

    let completion: DynBackendCompletionFuture<_, _> =
        Box::pin(
            async move { Ok::<_, ToyBackendError>(super::super::test_support::ToyCompletion) },
        );

    let (completion_tx, completion_rx) =
        oneshot::channel::<Result<AgentWrapperCompletion, AgentWrapperError>>();

    let task = tokio::spawn(send_completion_with_cancel(
        adapter,
        completion,
        cancel.clone(),
        Some(request_termination),
        completion_tx,
    ));

    let outcome = completion_rx.await.expect("completion sent");
    assert!(
        outcome.is_ok(),
        "backend completion should win when ready first"
    );

    cancel.cancel();
    assert_eq!(term_calls.load(Ordering::SeqCst), 0);

    task.await.expect("completion task completes");
}

#[tokio::test]
async fn cancellation_closes_events_but_does_not_accelerate_completion() {
    let adapter = std::sync::Arc::new(ToyAdapter { fail_spawn: false });
    let cancel = HarnessCancelSignal::new();

    let (backend_done_tx, backend_done_rx) = oneshot::channel::<()>();
    let completion: DynBackendCompletionFuture<_, _> = Box::pin(async move {
        backend_done_rx.await.expect("backend completion released");
        Ok::<super::super::test_support::ToyCompletion, ToyBackendError>(
            super::super::test_support::ToyCompletion,
        )
    });

    let (events_finish_tx, events_finish_rx) = oneshot::channel::<()>();
    let events = ControlledEndStream::<ToyEvent, ToyBackendError> {
        first: Some(Ok::<ToyEvent, ToyBackendError>(ToyEvent::Text(
            "one".to_string(),
        ))),
        finish_rx: events_finish_rx,
    };
    let events: DynBackendEventStream<_, _> = Box::pin(events);

    let (tx, rx) = mpsc::channel::<crate::AgentWrapperEvent>(8);
    let (completion_tx, completion_rx) =
        oneshot::channel::<Result<AgentWrapperCompletion, AgentWrapperError>>();

    let pump_task = tokio::spawn(pump_backend_events_with_cancel(
        adapter.clone(),
        events,
        tx,
        cancel.clone(),
    ));
    let completion_task = tokio::spawn(send_completion_with_cancel(
        adapter,
        completion,
        cancel.clone(),
        None,
        completion_tx,
    ));

    let mut handle = crate::run_handle_gate::build_gated_run_handle(rx, completion_rx);

    let first = handle.events.next().await.expect("first event forwarded");
    assert_eq!(first.kind, AgentWrapperEventKind::TextOutput);

    cancel.cancel();

    assert!(
        tokio::time::timeout(Duration::from_secs(1), handle.events.next())
            .await
            .expect("events do not hang")
            .is_none(),
        "events stream must be closed after cancellation"
    );

    {
        let waker = noop_waker();
        let mut cx = Context::from_waker(&waker);
        assert!(matches!(
            handle.completion.as_mut().poll(&mut cx),
            Poll::Pending
        ));
    }

    let _ = backend_done_tx.send(());
    let outcome = handle.completion.await;
    assert!(
        matches!(
            outcome,
            Err(AgentWrapperError::Backend { ref message }) if message == "cancelled"
        ),
        "completion must resolve to the pinned cancelled error after backend exit"
    );

    let _ = events_finish_tx.send(());
    pump_task.await.expect("pump task completes");
    completion_task.await.expect("completion task completes");
}

#[tokio::test]
async fn control_entrypoint_exposes_cancel_handle_wired_to_driver() {
    struct ControlAdapter {
        events_finish_rx: Mutex<Option<oneshot::Receiver<()>>>,
        backend_done_rx: Mutex<Option<oneshot::Receiver<()>>>,
    }

    impl BackendHarnessAdapter for ControlAdapter {
        fn kind(&self) -> crate::AgentWrapperKind {
            toy_kind()
        }

        fn supported_extension_keys(&self) -> &'static [&'static str] {
            &[]
        }

        type Policy = ();

        fn validate_and_extract_policy(
            &self,
            _request: &crate::AgentWrapperRunRequest,
        ) -> Result<Self::Policy, crate::AgentWrapperError> {
            Ok(())
        }

        type BackendEvent = String;
        type BackendCompletion = ();
        type BackendError = ();

        fn spawn(
            &self,
            _req: super::super::contract::NormalizedRequest<Self::Policy>,
        ) -> std::pin::Pin<
            Box<
                dyn std::future::Future<
                        Output = Result<
                            super::super::contract::BackendSpawn<
                                Self::BackendEvent,
                                Self::BackendCompletion,
                                Self::BackendError,
                            >,
                            Self::BackendError,
                        >,
                    > + Send
                    + 'static,
            >,
        > {
            let finish_rx = self
                .events_finish_rx
                .lock()
                .unwrap()
                .take()
                .expect("events gate available once");
            let backend_done_rx = self
                .backend_done_rx
                .lock()
                .unwrap()
                .take()
                .expect("backend gate available once");

            let events = ControlledEndStream::<String, ()> {
                first: Some(Ok::<String, ()>("one".to_string())),
                finish_rx,
            };
            let completion: DynBackendCompletionFuture<(), ()> = Box::pin(async move {
                backend_done_rx.await.expect("backend completion released");
                Ok::<(), ()>(())
            });

            Box::pin(async move {
                Ok(BackendSpawn {
                    events: Box::pin(events),
                    completion,
                })
            })
        }

        fn map_event(&self, event: Self::BackendEvent) -> Vec<crate::AgentWrapperEvent> {
            vec![crate::AgentWrapperEvent {
                agent_kind: toy_kind(),
                kind: AgentWrapperEventKind::TextOutput,
                channel: Some("assistant".to_string()),
                text: Some(event),
                message: None,
                data: None,
            }]
        }

        fn map_completion(
            &self,
            _completion: Self::BackendCompletion,
        ) -> Result<crate::AgentWrapperCompletion, crate::AgentWrapperError> {
            Ok(crate::AgentWrapperCompletion {
                status: super::super::test_support::success_exit_status(),
                final_text: None,
                data: None,
            })
        }

        fn redact_error(
            &self,
            _phase: BackendHarnessErrorPhase,
            _err: &Self::BackendError,
        ) -> String {
            "unused".to_string()
        }
    }

    let (events_finish_tx, events_finish_rx) = oneshot::channel::<()>();
    let (backend_done_tx, backend_done_rx) = oneshot::channel::<()>();

    let adapter = std::sync::Arc::new(ControlAdapter {
        events_finish_rx: Mutex::new(Some(events_finish_rx)),
        backend_done_rx: Mutex::new(Some(backend_done_rx)),
    });

    let request = crate::AgentWrapperRunRequest {
        prompt: "hello".to_string(),
        ..Default::default()
    };

    let crate::AgentWrapperRunControl { mut handle, cancel } =
        run_harnessed_backend_control(adapter, BackendDefaults::default(), request, None)
            .expect("control entrypoint succeeds");

    let first = handle.events.next().await.expect("first event forwarded");
    assert_eq!(first.kind, AgentWrapperEventKind::TextOutput);

    cancel.cancel();
    tokio::task::yield_now().await;

    assert!(
        tokio::time::timeout(Duration::from_secs(1), handle.events.next())
            .await
            .expect("events do not hang")
            .is_none(),
        "events stream must be closed after cancellation"
    );

    {
        let waker = noop_waker();
        let mut cx = Context::from_waker(&waker);
        assert!(matches!(
            handle.completion.as_mut().poll(&mut cx),
            Poll::Pending
        ));
    }

    let _ = backend_done_tx.send(());
    let outcome = handle.completion.await;
    assert!(
        matches!(
            outcome,
            Err(AgentWrapperError::Backend { ref message }) if message == "cancelled"
        ),
        "completion must resolve to the pinned cancelled error after backend exit"
    );

    let _ = events_finish_tx.send(());
}

#[tokio::test]
async fn pump_enforces_bounds_before_forwarding() {
    struct BoundsAdapter;

    impl BackendHarnessAdapter for BoundsAdapter {
        fn kind(&self) -> crate::AgentWrapperKind {
            toy_kind()
        }

        fn supported_extension_keys(&self) -> &'static [&'static str] {
            &[]
        }

        type Policy = ();

        fn validate_and_extract_policy(
            &self,
            _request: &crate::AgentWrapperRunRequest,
        ) -> Result<Self::Policy, crate::AgentWrapperError> {
            Ok(())
        }

        type BackendEvent = ();
        type BackendCompletion = ();
        type BackendError = ();

        fn spawn(
            &self,
            _req: super::super::contract::NormalizedRequest<Self::Policy>,
        ) -> std::pin::Pin<
            Box<
                dyn std::future::Future<
                        Output = Result<
                            super::super::contract::BackendSpawn<
                                Self::BackendEvent,
                                Self::BackendCompletion,
                                Self::BackendError,
                            >,
                            Self::BackendError,
                        >,
                    > + Send
                    + 'static,
            >,
        > {
            panic!("spawn unused in pump tests");
        }

        fn map_event(&self, _event: Self::BackendEvent) -> Vec<crate::AgentWrapperEvent> {
            vec![crate::AgentWrapperEvent {
                agent_kind: toy_kind(),
                kind: AgentWrapperEventKind::Error,
                channel: Some("error".to_string()),
                text: None,
                message: Some("a".repeat(crate::bounds::MESSAGE_BOUND_BYTES + 10)),
                data: None,
            }]
        }

        fn map_completion(
            &self,
            _completion: Self::BackendCompletion,
        ) -> Result<crate::AgentWrapperCompletion, crate::AgentWrapperError> {
            panic!("map_completion unused in pump tests");
        }

        fn redact_error(
            &self,
            _phase: BackendHarnessErrorPhase,
            _err: &Self::BackendError,
        ) -> String {
            "unused".to_string()
        }
    }

    let events = futures_util::stream::iter([Ok::<(), ()>(())]);
    let events: DynBackendEventStream<_, _> = Box::pin(events);

    let adapter = std::sync::Arc::new(BoundsAdapter);
    let (tx, mut rx) = mpsc::channel::<crate::AgentWrapperEvent>(8);
    let handle = tokio::spawn(pump_backend_events(adapter, events, tx));

    let ev = rx.recv().await.expect("one event forwarded");
    let message = ev.message.as_deref().expect("message present");
    assert!(message.len() <= crate::bounds::MESSAGE_BOUND_BYTES);
    assert!(message.ends_with("…(truncated)"));

    while rx.recv().await.is_some() {}
    handle.await.expect("pump task completes");
}

struct ControlledEndStream<E, BE> {
    first: Option<Result<E, BE>>,
    finish_rx: oneshot::Receiver<()>,
}

impl<E, BE> Unpin for ControlledEndStream<E, BE> {}

impl<E, BE> Stream for ControlledEndStream<E, BE> {
    type Item = Result<E, BE>;

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let this = self.get_mut();
        if let Some(first) = this.first.take() {
            return std::task::Poll::Ready(Some(first));
        }

        match Pin::new(&mut this.finish_rx).poll(cx) {
            std::task::Poll::Ready(_) => std::task::Poll::Ready(None),
            std::task::Poll::Pending => std::task::Poll::Pending,
        }
    }
}

#[tokio::test]
async fn pump_finality_sender_dropped_only_after_backend_stream_ends() {
    struct FinalityAdapter;

    impl BackendHarnessAdapter for FinalityAdapter {
        fn kind(&self) -> crate::AgentWrapperKind {
            toy_kind()
        }

        fn supported_extension_keys(&self) -> &'static [&'static str] {
            &[]
        }

        type Policy = ();

        fn validate_and_extract_policy(
            &self,
            _request: &crate::AgentWrapperRunRequest,
        ) -> Result<Self::Policy, crate::AgentWrapperError> {
            Ok(())
        }

        type BackendEvent = String;
        type BackendCompletion = ();
        type BackendError = ();

        fn spawn(
            &self,
            _req: super::super::contract::NormalizedRequest<Self::Policy>,
        ) -> std::pin::Pin<
            Box<
                dyn std::future::Future<
                        Output = Result<
                            super::super::contract::BackendSpawn<
                                Self::BackendEvent,
                                Self::BackendCompletion,
                                Self::BackendError,
                            >,
                            Self::BackendError,
                        >,
                    > + Send
                    + 'static,
            >,
        > {
            panic!("spawn unused in pump tests");
        }

        fn map_event(&self, event: Self::BackendEvent) -> Vec<crate::AgentWrapperEvent> {
            vec![crate::AgentWrapperEvent {
                agent_kind: toy_kind(),
                kind: AgentWrapperEventKind::TextOutput,
                channel: Some("assistant".to_string()),
                text: Some(event),
                message: None,
                data: None,
            }]
        }

        fn map_completion(
            &self,
            _completion: Self::BackendCompletion,
        ) -> Result<crate::AgentWrapperCompletion, crate::AgentWrapperError> {
            panic!("map_completion unused in pump tests");
        }

        fn redact_error(
            &self,
            _phase: BackendHarnessErrorPhase,
            _err: &Self::BackendError,
        ) -> String {
            "unused".to_string()
        }
    }

    let (finish_tx, finish_rx) = oneshot::channel::<()>();
    let events = ControlledEndStream::<String, ()> {
        first: Some(Ok::<String, ()>("hello".to_string())),
        finish_rx,
    };
    let events: DynBackendEventStream<_, _> = Box::pin(events);

    let adapter = std::sync::Arc::new(FinalityAdapter);
    let (tx, mut rx) = mpsc::channel::<crate::AgentWrapperEvent>(8);
    let handle = tokio::spawn(pump_backend_events(adapter, events, tx));

    let ev = rx.recv().await.expect("first event forwarded");
    assert_eq!(ev.kind, AgentWrapperEventKind::TextOutput);

    tokio::task::yield_now().await;
    assert!(
        matches!(
            rx.try_recv(),
            Err(tokio::sync::mpsc::error::TryRecvError::Empty)
        ),
        "events stream must not be final before backend stream ends"
    );
    assert!(
        !handle.is_finished(),
        "pump must not finish before stream end"
    );

    let _ = finish_tx.send(());
    handle.await.expect("pump task completes");
    assert!(
        rx.recv().await.is_none(),
        "events stream must be final after backend stream ends"
    );
}
