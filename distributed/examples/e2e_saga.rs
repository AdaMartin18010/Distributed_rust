use c20_distributed::transactions::{Saga, SagaStep};
//use c20_distributed::storage::{InMemoryIdempotency, IdempotencyStore};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

struct Debit(Arc<AtomicUsize>);
impl SagaStep for Debit {
    fn execute(&mut self) -> Result<(), c20_distributed::DistributedError> {
        self.0.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
    fn compensate(&mut self) -> Result<(), c20_distributed::DistributedError> {
        self.0.fetch_sub(1, Ordering::SeqCst);
        Ok(())
    }
}

struct Credit(Arc<AtomicUsize>);
impl SagaStep for Credit {
    fn execute(&mut self) -> Result<(), c20_distributed::DistributedError> {
        self.0.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
    fn compensate(&mut self) -> Result<(), c20_distributed::DistributedError> {
        self.0.fetch_sub(1, Ordering::SeqCst);
        Ok(())
    }
}

fn main() {
    let a = Arc::new(AtomicUsize::new(0));
    let b = Arc::new(AtomicUsize::new(0));
    let saga = Saga::new()
        .then(Box::new(Debit(a.clone())))
        .then(Box::new(Credit(b.clone())));
    let _ = saga.run();
    println!(
        "a={}, b={}",
        a.load(Ordering::SeqCst),
        b.load(Ordering::SeqCst)
    );
}
