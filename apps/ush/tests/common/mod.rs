use std::{
    thread,
    time::{Duration, Instant},
};

#[allow(dead_code)]
pub fn wait_until<F>(timeout: Duration, mut check: F) -> bool
where
    F: FnMut() -> bool,
{
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if check() {
            return true;
        }
        thread::sleep(Duration::from_millis(5));
    }
    check()
}
