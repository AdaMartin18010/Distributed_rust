//! Raft 接口骨架（受 feature `consensus-raft` 门控）
//! 
//! 增强功能：
//! - 日志压缩和快照
//! - 批量操作支持
//! - 性能优化

use crate::core::errors::DistributedError;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RaftState {
    Follower,
    Candidate,
    Leader,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Term(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LogIndex(pub u64);

#[derive(Debug, Clone)]
pub struct Snapshot {
    pub last_included_index: LogIndex,
    pub last_included_term: Term,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct InstallSnapshotReq {
    pub term: Term,
    pub leader_id: String,
    pub last_included_index: LogIndex,
    pub last_included_term: Term,
    pub offset: u64,
    pub data: Vec<u8>,
    pub done: bool,
}

#[derive(Debug, Clone)]
pub struct InstallSnapshotResp {
    pub term: Term,
}

#[derive(Debug, Clone)]
pub struct AppendEntriesReq<E> {
    pub term: Term,
    pub leader_id: String,
    pub prev_log_index: LogIndex,
    pub prev_log_term: Term,
    pub entries: Vec<E>,
    pub leader_commit: LogIndex,
}

#[derive(Debug, Clone)]
pub struct AppendEntriesResp {
    pub term: Term,
    pub success: bool,
}

#[derive(Debug, Clone)]
pub struct RequestVoteReq {
    pub term: Term,
    pub candidate_id: String,
    pub last_log_index: LogIndex,
    pub last_log_term: Term,
}

#[derive(Debug, Clone)]
pub struct RequestVoteResp {
    pub term: Term,
    pub vote_granted: bool,
}

pub trait RaftNode<E> {
    fn state(&self) -> RaftState;
    fn current_term(&self) -> Term;
    fn handle_append_entries(
        &mut self,
        req: AppendEntriesReq<E>,
    ) -> Result<AppendEntriesResp, DistributedError>;
    fn handle_request_vote(
        &mut self,
        req: RequestVoteReq,
    ) -> Result<RequestVoteResp, DistributedError>;
    fn handle_install_snapshot(
        &mut self,
        req: InstallSnapshotReq,
    ) -> Result<InstallSnapshotResp, DistributedError>;
    fn create_snapshot(&self) -> Result<Snapshot, DistributedError>;
    fn should_compact(&self, threshold: LogIndex) -> bool;
}

#[allow(dead_code)]
pub struct MinimalRaft<E> {
    state: RaftState,
    term: Term,
    log: Vec<(Term, E)>,
    commit_index: usize,
    last_applied: usize,
    apply: Option<Box<dyn FnMut(&E) + Send>>,
    // 快照相关字段
    snapshot: Option<Snapshot>,
    // 性能优化字段
    next_index: HashMap<String, usize>,
    match_index: HashMap<String, usize>,
    // 批量操作支持
    batch_size: usize,
}

impl<E> MinimalRaft<E> {
    pub fn new() -> Self {
        Self {
            state: RaftState::Follower,
            term: Term(0),
            log: Vec::new(),
            commit_index: 0,
            last_applied: 0,
            apply: None,
            snapshot: None,
            next_index: HashMap::new(),
            match_index: HashMap::new(),
            batch_size: 100, // 默认批量大小
        }
    }

    pub fn with_batch_size(mut self, batch_size: usize) -> Self {
        self.batch_size = batch_size;
        self
    }

    pub fn install_snapshot(&mut self, snapshot: Snapshot) {
        // 安装快照，截断日志
        let last_included_index = snapshot.last_included_index.0 as usize;
        if last_included_index > 0 && last_included_index <= self.log.len() {
            self.log.drain(0..last_included_index);
        }
        self.commit_index = last_included_index;
        self.last_applied = last_included_index;
        self.snapshot = Some(snapshot);
    }

    /// 创建快照
    pub fn create_snapshot_internal(&self, last_included_index: LogIndex) -> Result<Snapshot, DistributedError> {
        let last_included_term = if last_included_index.0 > 0 {
            let idx = (last_included_index.0 - 1) as usize;
            if let Some((term, _)) = self.log.get(idx) {
                *term
            } else {
                return Err(DistributedError::InvalidState("Log index out of bounds".to_string()));
            }
        } else {
            Term(0)
        };

        // 简化的快照数据（实际应用中应该序列化状态机状态）
        let data = format!("snapshot_at_{}", last_included_index.0).into_bytes();

        Ok(Snapshot {
            last_included_index,
            last_included_term,
            data,
        })
    }

    /// 检查是否需要压缩日志
    pub fn should_compact_internal(&self, threshold: LogIndex) -> bool {
        self.log.len() > threshold.0 as usize
    }

    pub fn set_apply(&mut self, f: Box<dyn FnMut(&E) + Send>) {
        self.apply = Some(f);
    }

    /// 提供作用域内生效的 apply 回调，而不要求 'static。
    /// 使用方法：
    /// let mut guard = raft.set_apply_scoped(&mut apply_fn);
    /// guard.handle_append_entries(req)?;
    pub fn set_apply_scoped<'a>(
        &'a mut self,
        f: &'a mut (dyn FnMut(&E) + Send),
    ) -> ScopedApply<'a, E> {
        ScopedApply {
            raft: self,
            apply: f,
        }
    }

    /// 内部核心实现：可传入临时回调用于应用 entries
    fn handle_append_entries_core(
        &mut self,
        req: AppendEntriesReq<E>,
        mut apply: Option<&mut (dyn FnMut(&E) + Send)>,
    ) -> Result<AppendEntriesResp, DistributedError>
    where
        E: Clone,
    {
        if req.term.0 < self.term.0 {
            return Ok(AppendEntriesResp {
                term: self.term,
                success: false,
            });
        }
        if req.term.0 > self.term.0 {
            self.term = req.term;
        }
        self.state = RaftState::Follower;

        // prev check
        let prev_idx = req.prev_log_index.0 as usize;
        if prev_idx > 0 {
            let i = prev_idx - 1;
            if let Some((t, _)) = self.log.get(i) {
                if t.0 != req.prev_log_term.0 {
                    return Ok(AppendEntriesResp {
                        term: self.term,
                        success: false,
                    });
                }
            } else {
                return Ok(AppendEntriesResp {
                    term: self.term,
                    success: false,
                });
            }
        }

        // overwrite from prev_log_index
        let mut insert_at = prev_idx;
        if insert_at > self.log.len() {
            insert_at = self.log.len();
        }
        self.log.truncate(insert_at);
        for e in req.entries.into_iter() {
            self.log.push((self.term, e));
        }

        // commit & apply
        let leader_commit = req.leader_commit.0 as usize;
        let log_len = self.log.len();
        self.commit_index = std::cmp::min(leader_commit, log_len);
        while self.last_applied < self.commit_index {
            let idx = self.last_applied; // 0-based
            if let Some((_, entry)) = self.log.get(idx) {
                if let Some(ref mut cb) = apply {
                    (cb)(entry);
                }
            }
            self.last_applied += 1;
        }

        Ok(AppendEntriesResp {
            term: self.term,
            success: true,
        })
    }
}

impl<E: Clone> RaftNode<E> for MinimalRaft<E> {
    fn state(&self) -> RaftState {
        self.state
    }
    fn current_term(&self) -> Term {
        self.term
    }

    fn handle_append_entries(
        &mut self,
        req: AppendEntriesReq<E>,
    ) -> Result<AppendEntriesResp, DistributedError> {
        // 将已有的 'static 回调透传给核心实现（通过临时 take 避免可变别名）
        let mut taken = self.apply.take();
        let res = match taken.as_mut() {
            Some(cb) => self
                .handle_append_entries_core(req, Some(cb.as_mut() as &mut (dyn FnMut(&E) + Send))),
            None => self.handle_append_entries_core(req, None),
        };
        self.apply = taken;
        res
    }

    fn handle_request_vote(
        &mut self,
        req: RequestVoteReq,
    ) -> Result<RequestVoteResp, DistributedError> {
        if req.term.0 > self.term.0 {
            self.term = req.term;
            self.state = RaftState::Follower;
            return Ok(RequestVoteResp {
                term: self.term,
                vote_granted: true,
            });
        }
        Ok(RequestVoteResp {
            term: self.term,
            vote_granted: false,
        })
    }

    fn handle_install_snapshot(
        &mut self,
        req: InstallSnapshotReq,
    ) -> Result<InstallSnapshotResp, DistributedError> {
        if req.term.0 < self.term.0 {
            return Ok(InstallSnapshotResp { term: self.term });
        }

        if req.term.0 > self.term.0 {
            self.term = req.term;
        }

        // 安装快照
        let snapshot = Snapshot {
            last_included_index: req.last_included_index,
            last_included_term: req.last_included_term,
            data: req.data,
        };
        self.install_snapshot(snapshot);

        Ok(InstallSnapshotResp { term: self.term })
    }

    fn create_snapshot(&self) -> Result<Snapshot, DistributedError> {
        let last_included_index = LogIndex(self.commit_index as u64);
        self.create_snapshot_internal(last_included_index)
    }

    fn should_compact(&self, threshold: LogIndex) -> bool {
        self.should_compact_internal(threshold)
    }
}

/// 守卫式作用域对象：对 `MinimalRaft` 的操作将使用提供的非 'static 回调
pub struct ScopedApply<'a, E> {
    raft: &'a mut MinimalRaft<E>,
    apply: &'a mut (dyn FnMut(&E) + Send),
}

impl<'a, E> ScopedApply<'a, E> {
    pub fn inner(&mut self) -> &mut MinimalRaft<E> {
        self.raft
    }
}

impl<'a, E: Clone> RaftNode<E> for ScopedApply<'a, E> {
    fn state(&self) -> RaftState {
        self.raft.state()
    }
    fn current_term(&self) -> Term {
        self.raft.current_term()
    }
    fn handle_append_entries(
        &mut self,
        req: AppendEntriesReq<E>,
    ) -> Result<AppendEntriesResp, DistributedError> {
        self.raft.handle_append_entries_core(req, Some(self.apply))
    }
    fn handle_request_vote(
        &mut self,
        req: RequestVoteReq,
    ) -> Result<RequestVoteResp, DistributedError> {
        self.raft.handle_request_vote(req)
    }
    fn handle_install_snapshot(
        &mut self,
        req: InstallSnapshotReq,
    ) -> Result<InstallSnapshotResp, DistributedError> {
        self.raft.handle_install_snapshot(req)
    }
    fn create_snapshot(&self) -> Result<Snapshot, DistributedError> {
        self.raft.create_snapshot()
    }
    fn should_compact(&self, threshold: LogIndex) -> bool {
        self.raft.should_compact(threshold)
    }
}
