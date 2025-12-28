#![cfg(target_os = "linux")]

mod fixtures;

use std::time::Duration;

use anyhow::Result;
use crossbeam_channel::unbounded;
use jdb::options::{Aslr, Options};
use jdb::process::Process;
use jdb::process::register_info::Register;

/// Wrapper around the `Process` instance. The key insight is implementing the
/// `Drop` trait which will guarantee the proper shutdown of the `Process`.
struct ProcessGuard {
    process: Option<Process>,
    shutdown_tx: Option<crossbeam_channel::Sender<()>>,
}

impl ProcessGuard {
    fn new(process: Process, shutdown_tx: crossbeam_channel::Sender<()>) -> Self {
        Self {
            process: Some(process),
            shutdown_tx: Some(shutdown_tx),
        }
    }

    fn get_mut(&mut self) -> &mut Process {
        self.process
            .as_mut()
            .expect("process should still be available")
    }
}

impl Drop for ProcessGuard {
    fn drop(&mut self) {
        if let Some(mut process) = self.process.take() {
            let _ = process.destroy();
        }
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
    }
}

/// End-to-end smoke test: spawn the fixture, stop on SIGSTOP, read registers,
/// and resume until exit.
#[test]
fn attach_read_and_resume_inferior() -> Result<()> {
    let fixture = fixtures::hello_fixture_path();

    let (inferior_tx, inferior_rx) = unbounded();
    let (shutdown_tx, shutdown_rx) = unbounded();
    let options = Options {
        executable: fixture.clone(),
    };
    let mut process_guard =
        ProcessGuard::new(Process::new(options, inferior_tx, shutdown_rx), shutdown_tx);
    let process = process_guard.get_mut();

    // Attach (launch) the inferior; it stops on SIGSTOP right away.
    process.attach(Vec::new(), Aslr::Disabled).expect("attach should succeed");
    let pid = process.pid().expect("pid should be available after attach");
    process.resume()?;

    // Registers should be readable while stopped.
    assert!(process.read_register(Register::RIP).is_some());
    assert!(process.read_register(Register::RSP).is_some());

    // The inferior prints once before stopping; ensure we captured it.
    let msg = inferior_rx.recv_timeout(Duration::from_secs(5))?;
    process.receive_inferior_logging(msg);
    let logs = process.last_n_log_lines(4);
    assert!(
        logs.iter().any(|l| l.contains("HELLO_FROM_INFERIOR")),
        "log lines did not include expected greeting: {logs:?}"
    );

    // Resume execution and wait for exit.
    process.resume().expect("resume should succeed");
    let wait_status = process.wait_on_signal().expect("wait should succeed");
    assert!(
        matches!(wait_status, nix::sys::wait::WaitStatus::Exited(p, _) if p == pid),
        "expected inferior to exit after resume, got {wait_status:?}"
    );

    // Clean teardown.
    Ok(())
}
