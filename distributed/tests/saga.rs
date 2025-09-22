use c20_distributed::transactions::{Saga, SagaStep};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

struct OkStep(Arc<AtomicUsize>);
impl SagaStep for OkStep {
    fn execute(&mut self) -> Result<(), c20_distributed::DistributedError> {
        self.0.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
    fn compensate(&mut self) -> Result<(), c20_distributed::DistributedError> {
        Ok(())
    }
}

struct FailStep(Arc<AtomicUsize>);
impl SagaStep for FailStep {
    fn execute(&mut self) -> Result<(), c20_distributed::DistributedError> {
        Err(c20_distributed::DistributedError::Configuration(
            "fail".into(),
        ))
    }
    fn compensate(&mut self) -> Result<(), c20_distributed::DistributedError> {
        self.0.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

#[test]
fn saga_rollback_on_failure() {
    let c = Arc::new(AtomicUsize::new(0));
    let rc = c.clone();
    let saga = Saga::new()
        .then(Box::new(OkStep(rc.clone())))
        .then(Box::new(FailStep(rc.clone())));
    let _ = saga.run();
    assert_eq!(c.load(Ordering::SeqCst), 1);
}
