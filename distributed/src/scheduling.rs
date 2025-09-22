#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct LogicalClock {
    pub tick: u64,
}

pub trait TimerService {
    fn after_ms(&self, ms: u64, f: impl FnOnce() + Send + 'static);
}

#[cfg(feature = "runtime-tokio")]
#[derive(Debug, Default, Clone)]
pub struct TokioTimer;

#[cfg(feature = "runtime-tokio")]
impl TimerService for TokioTimer {
    fn after_ms(&self, ms: u64, f: impl FnOnce() + Send + 'static) {
        let duration = std::time::Duration::from_millis(ms);
        tokio::spawn(async move {
            tokio::time::sleep(duration).await;
            f();
        });
    }
}
