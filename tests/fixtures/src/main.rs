use std::thread;
use std::time::Duration;

fn main() {
    // Emit a line we can assert on.
    println!("HELLO_FROM_INFERIOR");

    // Stop immediately so ptrace sees SIGSTOP on attach.
    unsafe {
        libc::raise(libc::SIGSTOP);
    }

    // After continuing, run briefly and exit.
    for _ in 0..3 {
        thread::sleep(Duration::from_millis(10));
    }
}
