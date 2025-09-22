//! 存储模块
//! 
//! 提供分布式存储相关的抽象和实现

pub mod replication;

use crate::codec::BinaryCodec;
use crate::core::errors::DistributedError;

pub trait LogStorage<E> {
    fn append(&mut self, entry: E) -> Result<u64, DistributedError>;
}

pub trait StateMachineStorage<S, C> {
    fn apply(&mut self, state: &mut S, command: C) -> Result<(), DistributedError>;
}

use std::collections::HashSet;

pub trait IdempotencyStore<ID> {
    fn seen(&self, id: &ID) -> bool;
    fn record(&mut self, id: ID);
}

#[derive(Default)]
pub struct InMemoryIdempotency<ID: std::hash::Hash + Eq> {
    set: HashSet<ID>,
}

impl<ID: std::hash::Hash + Eq + Clone> IdempotencyStore<ID> for InMemoryIdempotency<ID> {
    fn seen(&self, id: &ID) -> bool {
        self.set.contains(id)
    }
    fn record(&mut self, id: ID) {
        self.set.insert(id);
    }
}

pub trait SnapshotStorage<S> {
    fn save_snapshot(&mut self, state: &S) -> Result<(), DistributedError>;
    fn load_snapshot(&self) -> Result<Option<S>, DistributedError>
    where
        S: Clone;
}

// ---------------- File-based persistence (minimal) ----------------

pub struct FileLogStorage<C: BinaryCodec<E>, E> {
    path: std::path::PathBuf,
    codec: C,
    _marker: std::marker::PhantomData<E>,
}

impl<C: BinaryCodec<E>, E> FileLogStorage<C, E> {
    pub fn new(path: std::path::PathBuf, codec: C) -> Self {
        Self {
            path,
            codec,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<C: BinaryCodec<E>, E> LogStorage<E> for FileLogStorage<C, E> {
    fn append(&mut self, entry: E) -> Result<u64, DistributedError> {
        use std::io::{Seek, SeekFrom, Write};
        let bytes = self.codec.encode(&entry);
        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .read(true)
            .open(&self.path)
            .map_err(|e| DistributedError::Storage(e.to_string()))?;
        let len = bytes.len() as u64;
        f.write_all(&len.to_le_bytes())
            .map_err(|e| DistributedError::Storage(e.to_string()))?;
        f.write_all(&bytes)
            .map_err(|e| DistributedError::Storage(e.to_string()))?;
        let pos = f
            .seek(SeekFrom::End(0))
            .map_err(|e| DistributedError::Storage(e.to_string()))?;
        Ok(pos)
    }
}

pub struct FileSnapshot<C: BinaryCodec<S>, S: Clone> {
    path: std::path::PathBuf,
    codec: C,
    _marker: std::marker::PhantomData<S>,
}

impl<C: BinaryCodec<S>, S: Clone> FileSnapshot<C, S> {
    pub fn new(path: std::path::PathBuf, codec: C) -> Self {
        Self {
            path,
            codec,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<C: BinaryCodec<S>, S: Clone> SnapshotStorage<S> for FileSnapshot<C, S> {
    fn save_snapshot(&mut self, state: &S) -> Result<(), DistributedError> {
        let bytes = self.codec.encode(state);
        std::fs::write(&self.path, bytes).map_err(|e| DistributedError::Storage(e.to_string()))
    }
    fn load_snapshot(&self) -> Result<Option<S>, DistributedError> {
        match std::fs::read(&self.path) {
            Ok(b) => Ok(self.codec.decode(&b)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(DistributedError::Storage(e.to_string())),
        }
    }
}

#[derive(Default)]
pub struct InMemorySnapshot<S: Clone> {
    pub snapshot: Option<S>,
}

impl<S: Clone> SnapshotStorage<S> for InMemorySnapshot<S> {
    fn save_snapshot(&mut self, state: &S) -> Result<(), DistributedError> {
        self.snapshot = Some(state.clone());
        Ok(())
    }
    fn load_snapshot(&self) -> Result<Option<S>, DistributedError> {
        Ok(self.snapshot.clone())
    }
}

#[derive(Default)]
pub struct InMemoryLogStorage<E> {
    entries: Vec<E>,
}

impl<E> InMemoryLogStorage<E> {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl<E> LogStorage<E> for InMemoryLogStorage<E> {
    fn append(&mut self, entry: E) -> Result<u64, DistributedError> {
        self.entries.push(entry);
        Ok(self.entries.len() as u64)
    }
}
