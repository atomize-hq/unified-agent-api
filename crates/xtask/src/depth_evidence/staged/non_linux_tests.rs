//! Without the Linux peer anchor every channel refuses, a live host's own channel included, and
//! the candidate is never written.
use super::*;

#[test]
fn the_channel_refuses_without_the_linux_peer_anchor() {
    let root = tempfile::tempdir().unwrap();
    let target = Target {
        agent_id: "depth_fixture".into(),
        version: "0.2.0".into(),
        manifest_root: "cli_manifests/depth_fixture".into(),
    };
    let staged = stage_answer(root.path(), Err("unused".into()), &target).unwrap();
    let channel = staged.channel();
    channel.admission.admit(std::process::id());
    let nonce = channel.admission.nonce().to_owned();
    let error = assemble(root.path(), Some(channel.locator), &nonce).unwrap_err();
    assert!(error.contains("only on Linux"), "{error}");
    assert_eq!(fs::read_dir(root.path()).unwrap().count(), 0);
}
