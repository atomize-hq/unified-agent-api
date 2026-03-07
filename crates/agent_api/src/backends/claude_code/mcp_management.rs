use std::ffi::OsString;

use tokio::io::{self, AsyncRead, AsyncReadExt};

const CAPTURE_SCRATCH_BUFFER_BYTES: usize = 8 * 1024;

#[allow(dead_code)]
pub(super) fn claude_mcp_list_argv() -> Vec<OsString> {
    vec![
        OsString::from("claude"),
        OsString::from("mcp"),
        OsString::from("list"),
    ]
}

#[allow(dead_code)]
pub(super) fn claude_mcp_get_argv(name: &str) -> Vec<OsString> {
    vec![
        OsString::from("claude"),
        OsString::from("mcp"),
        OsString::from("get"),
        OsString::from(name),
    ]
}

#[allow(dead_code)]
pub(super) async fn capture_bounded<R: AsyncRead + Unpin>(
    mut reader: R,
    bound_bytes: usize,
) -> io::Result<(Vec<u8>, bool)> {
    let mut retained = Vec::with_capacity(bound_bytes.min(CAPTURE_SCRATCH_BUFFER_BYTES));
    let mut scratch = [0_u8; CAPTURE_SCRATCH_BUFFER_BYTES];
    let mut saw_more_bytes = false;

    loop {
        let read = reader.read(&mut scratch).await?;
        if read == 0 {
            break;
        }

        let chunk = &scratch[..read];
        let remaining = bound_bytes.saturating_sub(retained.len());
        if remaining > 0 {
            let keep = remaining.min(chunk.len());
            retained.extend_from_slice(&chunk[..keep]);
            if keep < chunk.len() {
                saw_more_bytes = true;
            }
        } else {
            saw_more_bytes = true;
        }
    }

    Ok((retained, saw_more_bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    use tokio::io::{duplex, AsyncWriteExt};

    #[test]
    fn claude_mcp_list_argv_is_pinned() {
        assert_eq!(
            claude_mcp_list_argv(),
            vec![
                OsString::from("claude"),
                OsString::from("mcp"),
                OsString::from("list"),
            ]
        );
    }

    #[test]
    fn claude_mcp_get_argv_is_pinned_for_standard_name() {
        assert_eq!(
            claude_mcp_get_argv("demo-server"),
            vec![
                OsString::from("claude"),
                OsString::from("mcp"),
                OsString::from("get"),
                OsString::from("demo-server"),
            ]
        );
    }

    #[test]
    fn claude_mcp_get_argv_preserves_name_as_single_item() {
        assert_eq!(
            claude_mcp_get_argv("demo server"),
            vec![
                OsString::from("claude"),
                OsString::from("mcp"),
                OsString::from("get"),
                OsString::from("demo server"),
            ]
        );
    }

    async fn capture_chunks(
        chunks: &[&[u8]],
        capacity: usize,
        bound_bytes: usize,
    ) -> (Vec<u8>, bool) {
        let (mut writer, reader) = duplex(capacity);
        let owned_chunks: Vec<Vec<u8>> = chunks.iter().map(|chunk| chunk.to_vec()).collect();
        let writer_task = tokio::spawn(async move {
            for chunk in owned_chunks {
                writer
                    .write_all(&chunk)
                    .await
                    .expect("duplex writer accepts chunk");
            }
            writer.shutdown().await.expect("duplex writer shuts down");
        });

        let captured = capture_bounded(reader, bound_bytes)
            .await
            .expect("capture should succeed");
        writer_task.await.expect("writer task completes");
        captured
    }

    #[tokio::test]
    async fn capture_bounded_returns_full_bytes_when_under_bound() {
        let (captured, saw_more_bytes) = capture_chunks(&[b"plain output"], 32, 32).await;

        assert_eq!(captured, b"plain output");
        assert!(!saw_more_bytes);
    }

    #[tokio::test]
    async fn capture_bounded_returns_full_bytes_when_exactly_at_bound() {
        let (captured, saw_more_bytes) = capture_chunks(&[b"12345678"], 16, 8).await;

        assert_eq!(captured, b"12345678");
        assert!(!saw_more_bytes);
    }

    #[tokio::test]
    async fn capture_bounded_retains_prefix_and_marks_overflow_for_multi_chunk_input() {
        let (captured, saw_more_bytes) = capture_chunks(&[b"abcd", b"efghi"], 4, 5).await;

        assert_eq!(captured, b"abcde");
        assert!(saw_more_bytes);
    }

    #[tokio::test]
    async fn capture_bounded_with_zero_bound_drains_input_and_reports_overflow() {
        let (captured, saw_more_bytes) = capture_chunks(&[b"abc", b"def"], 3, 0).await;

        assert!(captured.is_empty());
        assert!(saw_more_bytes);
    }
}
