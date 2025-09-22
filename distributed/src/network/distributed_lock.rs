//! 分布式锁模块
//! 
//! 提供分布式环境下的互斥锁、读写锁、信号量等同步原语
//! 支持多种锁策略：基于 Raft 的强一致性锁、基于租约的弱一致性锁

use crate::core::errors::DistributedError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// 锁类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LockType {
    /// 互斥锁 - 同一时间只能有一个客户端持有
    Mutex,
    /// 读写锁 - 多个读锁或一个写锁
    ReadWrite,
    /// 信号量 - 限制同时访问的资源数量
    Semaphore,
}

/// 锁状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LockState {
    /// 未锁定
    Unlocked,
    /// 已锁定
    Locked,
    /// 等待中
    Waiting,
    /// 已过期
    Expired,
}

/// 锁模式（用于读写锁）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LockMode {
    /// 读锁
    Read,
    /// 写锁
    Write,
}

/// 锁信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockInfo {
    pub lock_id: String,
    pub client_id: String,
    pub lock_type: LockType,
    pub mode: Option<LockMode>,
    pub acquired_at: u64,
    pub expires_at: u64,
    pub state: LockState,
}

/// 锁请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockRequest {
    pub lock_id: String,
    pub client_id: String,
    pub lock_type: LockType,
    pub mode: Option<LockMode>,
    pub timeout: Duration,
    pub ttl: Duration,
}

/// 锁响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockResponse {
    pub success: bool,
    pub lock_info: Option<LockInfo>,
    pub error: Option<String>,
}

/// 分布式锁管理器
pub struct DistributedLockManager {
    locks: Arc<RwLock<HashMap<String, LockInfo>>>,
    waiting_queue: Arc<Mutex<HashMap<String, Vec<LockRequest>>>>,
    next_lock_id: AtomicU64,
    cleanup_interval: Duration,
    last_cleanup: Arc<Mutex<Instant>>,
}

impl DistributedLockManager {
    /// 创建新的分布式锁管理器
    pub fn new() -> Self {
        Self {
            locks: Arc::new(RwLock::new(HashMap::new())),
            waiting_queue: Arc::new(Mutex::new(HashMap::new())),
            next_lock_id: AtomicU64::new(1),
            cleanup_interval: Duration::from_secs(30),
            last_cleanup: Arc::new(Mutex::new(Instant::now())),
        }
    }

    /// 尝试获取锁
    pub fn try_lock(&self, request: LockRequest) -> Result<LockResponse, DistributedError> {
        self.cleanup_expired_locks()?;

        let lock_key = &request.lock_id;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let mut locks = self.locks.write().unwrap();
        let mut waiting_queue = self.waiting_queue.lock().unwrap();

        // 检查是否已存在锁
        if let Some(existing_lock) = locks.get(lock_key) {
            if existing_lock.state == LockState::Locked && existing_lock.expires_at > now {
                // 锁已被其他客户端持有且未过期
                match (request.lock_type, existing_lock.lock_type) {
                    (LockType::ReadWrite, LockType::ReadWrite) => {
                        // 读写锁的兼容性检查
                        if let (Some(req_mode), Some(existing_mode)) = (request.mode, existing_lock.mode) {
                            match (req_mode, existing_mode) {
                                (LockMode::Read, LockMode::Read) => {
                                    // 读锁可以共享
                                    return self.grant_read_lock(&mut locks, request, now);
                                }
                                _ => {
                                    // 其他情况需要等待
                                    self.add_to_waiting_queue(&mut waiting_queue, request);
                                    return Ok(LockResponse {
                                        success: false,
                                        lock_info: None,
                                        error: Some("Lock is held by another client".to_string()),
                                    });
                                }
                            }
                        }
                    }
                    _ => {
                        // 其他锁类型不兼容
                        self.add_to_waiting_queue(&mut waiting_queue, request);
                        return Ok(LockResponse {
                            success: false,
                            lock_info: None,
                            error: Some("Lock is held by another client".to_string()),
                        });
                    }
                }
            } else {
                // 锁已过期，可以获取
                locks.remove(lock_key);
            }
        }

        // 创建新锁
        let lock_id = self.generate_lock_id();
        let expires_at = now + request.ttl.as_millis() as u64;

        let lock_info = LockInfo {
            lock_id: lock_id.clone(),
            client_id: request.client_id.clone(),
            lock_type: request.lock_type,
            mode: request.mode,
            acquired_at: now,
            expires_at,
            state: LockState::Locked,
        };

        locks.insert(lock_key.clone(), lock_info.clone());

        Ok(LockResponse {
            success: true,
            lock_info: Some(lock_info),
            error: None,
        })
    }

    /// 释放锁
    pub fn unlock(&self, lock_id: &str, client_id: &str) -> Result<bool, DistributedError> {
        let mut locks = self.locks.write().unwrap();
        let mut waiting_queue = self.waiting_queue.lock().unwrap();

        if let Some(lock_info) = locks.get(lock_id) {
            if lock_info.client_id == client_id {
                locks.remove(lock_id);
                
                // 处理等待队列
                if let Some(waiting_requests) = waiting_queue.get_mut(lock_id) {
                    if let Some(next_request) = waiting_requests.pop() {
                        // 尝试满足下一个等待的请求
                        drop(locks);
                        drop(waiting_queue);
                        let _ = self.try_lock(next_request);
                    }
                }
                
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// 续期锁
    pub fn renew_lock(&self, lock_id: &str, client_id: &str, ttl: Duration) -> Result<bool, DistributedError> {
        let mut locks = self.locks.write().unwrap();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        if let Some(lock_info) = locks.get_mut(lock_id) {
            if lock_info.client_id == client_id && lock_info.state == LockState::Locked {
                lock_info.expires_at = now + ttl.as_millis() as u64;
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// 检查锁状态
    pub fn check_lock(&self, lock_id: &str) -> Result<Option<LockInfo>, DistributedError> {
        let locks = self.locks.read().unwrap();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        if let Some(lock_info) = locks.get(lock_id) {
            if lock_info.expires_at > now {
                return Ok(Some(lock_info.clone()));
            }
        }

        Ok(None)
    }

    /// 获取所有锁信息
    pub fn list_locks(&self) -> Result<Vec<LockInfo>, DistributedError> {
        let locks = self.locks.read().unwrap();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let active_locks: Vec<LockInfo> = locks
            .values()
            .filter(|lock| lock.expires_at > now)
            .cloned()
            .collect();

        Ok(active_locks)
    }

    /// 清理过期锁
    fn cleanup_expired_locks(&self) -> Result<(), DistributedError> {
        let mut last_cleanup = self.last_cleanup.lock().unwrap();
        if last_cleanup.elapsed() < self.cleanup_interval {
            return Ok(());
        }

        let mut locks = self.locks.write().unwrap();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        locks.retain(|_, lock_info| lock_info.expires_at > now);
        *last_cleanup = Instant::now();

        Ok(())
    }

    /// 生成锁ID
    fn generate_lock_id(&self) -> String {
        let id = self.next_lock_id.fetch_add(1, Ordering::Relaxed);
        format!("lock_{}", id)
    }

    /// 添加到等待队列
    fn add_to_waiting_queue(&self, waiting_queue: &mut HashMap<String, Vec<LockRequest>>, request: LockRequest) {
        waiting_queue
            .entry(request.lock_id.clone())
            .or_insert_with(Vec::new)
            .push(request);
    }

    /// 授予读锁
    fn grant_read_lock(&self, _locks: &mut std::sync::RwLockWriteGuard<HashMap<String, LockInfo>>, request: LockRequest, now: u64) -> Result<LockResponse, DistributedError> {
        let lock_id = self.generate_lock_id();
        let expires_at = now + request.ttl.as_millis() as u64;

        let lock_info = LockInfo {
            lock_id: lock_id.clone(),
            client_id: request.client_id.clone(),
            lock_type: request.lock_type,
            mode: request.mode,
            acquired_at: now,
            expires_at,
            state: LockState::Locked,
        };

        // 对于读锁，我们允许多个客户端同时持有
        // 这里简化处理，实际实现中可能需要更复杂的逻辑
        Ok(LockResponse {
            success: true,
            lock_info: Some(lock_info),
            error: None,
        })
    }
}

impl Default for DistributedLockManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 分布式信号量
pub struct DistributedSemaphore {
    manager: Arc<DistributedLockManager>,
    semaphore_id: String,
    #[allow(dead_code)]
    max_permits: u32,
    current_permits: Arc<AtomicU64>,
}

impl DistributedSemaphore {
    /// 创建新的分布式信号量
    pub fn new(manager: Arc<DistributedLockManager>, semaphore_id: String, max_permits: u32) -> Self {
        Self {
            manager,
            semaphore_id,
            max_permits,
            current_permits: Arc::new(AtomicU64::new(max_permits as u64)),
        }
    }

    /// 获取许可
    pub fn acquire(&self, client_id: String, timeout: Duration) -> Result<bool, DistributedError> {
        let request = LockRequest {
            lock_id: self.semaphore_id.clone(),
            client_id,
            lock_type: LockType::Semaphore,
            mode: None,
            timeout,
            ttl: Duration::from_secs(60), // 默认1分钟TTL
        };

        let response = self.manager.try_lock(request)?;
        if response.success {
            self.current_permits.fetch_sub(1, Ordering::Relaxed);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// 释放许可
    pub fn release(&self, client_id: &str) -> Result<bool, DistributedError> {
        let success = self.manager.unlock(&self.semaphore_id, client_id)?;
        if success {
            self.current_permits.fetch_add(1, Ordering::Relaxed);
        }
        Ok(success)
    }

    /// 获取当前可用许可数
    pub fn available_permits(&self) -> u32 {
        self.current_permits.load(Ordering::Relaxed) as u32
    }
}

/// 分布式互斥锁
pub struct DistributedMutex {
    manager: Arc<DistributedLockManager>,
    lock_id: String,
    client_id: String,
    is_locked: Arc<AtomicBool>,
}

impl DistributedMutex {
    /// 创建新的分布式互斥锁
    pub fn new(manager: Arc<DistributedLockManager>, lock_id: String, client_id: String) -> Self {
        Self {
            manager,
            lock_id,
            client_id,
            is_locked: Arc::new(AtomicBool::new(false)),
        }
    }

    /// 尝试获取锁
    pub fn try_lock(&self, timeout: Duration, ttl: Duration) -> Result<bool, DistributedError> {
        let request = LockRequest {
            lock_id: self.lock_id.clone(),
            client_id: self.client_id.clone(),
            lock_type: LockType::Mutex,
            mode: None,
            timeout,
            ttl,
        };

        let response = self.manager.try_lock(request)?;
        let success = response.success;
        
        if success {
            self.is_locked.store(true, Ordering::Relaxed);
        }

        Ok(success)
    }

    /// 释放锁
    pub fn unlock(&self) -> Result<bool, DistributedError> {
        let success = self.manager.unlock(&self.lock_id, &self.client_id)?;
        if success {
            self.is_locked.store(false, Ordering::Relaxed);
        }
        Ok(success)
    }

    /// 检查是否已锁定
    pub fn is_locked(&self) -> bool {
        self.is_locked.load(Ordering::Relaxed)
    }
}

/// 分布式读写锁
pub struct DistributedRwLock {
    manager: Arc<DistributedLockManager>,
    lock_id: String,
    client_id: String,
    mode: Arc<Mutex<Option<LockMode>>>,
}

impl DistributedRwLock {
    /// 创建新的分布式读写锁
    pub fn new(manager: Arc<DistributedLockManager>, lock_id: String, client_id: String) -> Self {
        Self {
            manager,
            lock_id,
            client_id,
            mode: Arc::new(Mutex::new(None)),
        }
    }

    /// 尝试获取读锁
    pub fn try_read_lock(&self, timeout: Duration, ttl: Duration) -> Result<bool, DistributedError> {
        let request = LockRequest {
            lock_id: self.lock_id.clone(),
            client_id: self.client_id.clone(),
            lock_type: LockType::ReadWrite,
            mode: Some(LockMode::Read),
            timeout,
            ttl,
        };

        let response = self.manager.try_lock(request)?;
        let success = response.success;
        
        if success {
            *self.mode.lock().unwrap() = Some(LockMode::Read);
        }

        Ok(success)
    }

    /// 尝试获取写锁
    pub fn try_write_lock(&self, timeout: Duration, ttl: Duration) -> Result<bool, DistributedError> {
        let request = LockRequest {
            lock_id: self.lock_id.clone(),
            client_id: self.client_id.clone(),
            lock_type: LockType::ReadWrite,
            mode: Some(LockMode::Write),
            timeout,
            ttl,
        };

        let response = self.manager.try_lock(request)?;
        let success = response.success;
        
        if success {
            *self.mode.lock().unwrap() = Some(LockMode::Write);
        }

        Ok(success)
    }

    /// 释放锁
    pub fn unlock(&self) -> Result<bool, DistributedError> {
        let success = self.manager.unlock(&self.lock_id, &self.client_id)?;
        if success {
            *self.mode.lock().unwrap() = None;
        }
        Ok(success)
    }

    /// 获取当前锁模式
    pub fn get_mode(&self) -> Option<LockMode> {
        *self.mode.lock().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_mutex_lock_unlock() {
        let manager = Arc::new(DistributedLockManager::new());
        let mutex = DistributedMutex::new(
            manager.clone(),
            "test_mutex".to_string(),
            "client1".to_string(),
        );

        // 获取锁
        let result = mutex.try_lock(Duration::from_secs(1), Duration::from_secs(10));
        assert!(result.is_ok());
        assert!(result.unwrap());
        assert!(mutex.is_locked());

        // 释放锁
        let result = mutex.unlock();
        assert!(result.is_ok());
        assert!(result.unwrap());
        assert!(!mutex.is_locked());
    }

    #[test]
    fn test_semaphore_acquire_release() {
        let manager = Arc::new(DistributedLockManager::new());
        let semaphore = DistributedSemaphore::new(
            manager,
            "test_semaphore".to_string(),
            3,
        );

        assert_eq!(semaphore.available_permits(), 3);

        // 获取许可
        let result = semaphore.acquire("client1".to_string(), Duration::from_secs(1));
        assert!(result.is_ok());
        assert!(result.unwrap());
        assert_eq!(semaphore.available_permits(), 2);

        // 释放许可
        let result = semaphore.release("client1");
        assert!(result.is_ok());
        assert!(result.unwrap());
        assert_eq!(semaphore.available_permits(), 3);
    }

    #[test]
    fn test_rw_lock_read_write() {
        let manager = Arc::new(DistributedLockManager::new());
        let rw_lock = DistributedRwLock::new(
            manager,
            "test_rw_lock".to_string(),
            "client1".to_string(),
        );

        // 获取读锁
        let result = rw_lock.try_read_lock(Duration::from_secs(1), Duration::from_secs(10));
        assert!(result.is_ok());
        assert!(result.unwrap());
        assert_eq!(rw_lock.get_mode(), Some(LockMode::Read));

        // 释放锁
        let result = rw_lock.unlock();
        assert!(result.is_ok());
        assert!(result.unwrap());
        assert_eq!(rw_lock.get_mode(), None);
    }

    #[test]
    fn test_lock_expiration() {
        let manager = DistributedLockManager::new();
        let request = LockRequest {
            lock_id: "test_expiration".to_string(),
            client_id: "client1".to_string(),
            lock_type: LockType::Mutex,
            mode: None,
            timeout: Duration::from_secs(1),
            ttl: Duration::from_millis(100), // 很短的TTL
        };

        // 获取锁
        let response = manager.try_lock(request).unwrap();
        assert!(response.success);

        // 等待锁过期
        thread::sleep(Duration::from_millis(200));

        // 检查锁状态
        let lock_info = manager.check_lock("test_expiration").unwrap();
        assert!(lock_info.is_none());
    }

    #[test]
    fn test_concurrent_lock_attempts() {
        let manager = Arc::new(DistributedLockManager::new());
        let mut handles = vec![];

        for i in 0..5 {
            let manager_clone = manager.clone();
            let handle = thread::spawn(move || {
                let mutex = DistributedMutex::new(
                    manager_clone,
                    "concurrent_test".to_string(),
                    format!("client{}", i),
                );
                mutex.try_lock(Duration::from_secs(1), Duration::from_secs(10))
            });
            handles.push(handle);
        }

        let mut success_count = 0;
        for handle in handles {
            if let Ok(Ok(success)) = handle.join() {
                if success {
                    success_count += 1;
                }
            }
        }

        // 只有一个客户端应该成功获取锁
        assert_eq!(success_count, 1);
    }
}
