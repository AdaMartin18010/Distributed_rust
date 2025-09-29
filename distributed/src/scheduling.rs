//! 调度与定时
//!
//! 范围与目标：
//! - 提供逻辑时钟与抽象定时服务接口，便于在不同运行时（如 Tokio）下注入实现。
//! - 作为重试、心跳、选举与健康检查的时间基础设施。
//!
//! 注意点（草图）：
//! - 定时回调应避免长阻塞；在异步运行时中使用 `spawn` 与定时器。
//! - 逻辑时钟可用于单调事件编号与幂等键生成。
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
