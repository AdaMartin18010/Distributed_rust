use crate::core::errors::DistributedError;

pub trait SagaStep {
    fn execute(&mut self) -> Result<(), DistributedError>;
    fn compensate(&mut self) -> Result<(), DistributedError>;
}

pub struct Saga {
    steps: Vec<Box<dyn SagaStep + Send>>,
}

impl Default for Saga {
    fn default() -> Self {
        Self::new()
    }
}

impl Saga {
    pub fn new() -> Self {
        Self { steps: Vec::new() }
    }
    pub fn then(mut self, step: Box<dyn SagaStep + Send>) -> Self {
        self.steps.push(step);
        self
    }

    pub fn run(self) -> Result<(), DistributedError> {
        let mut done: Vec<Box<dyn SagaStep + Send>> = Vec::new();
        for mut s in self.steps.into_iter() {
            match s.execute() {
                Ok(_) => done.push(s),
                Err(e) => {
                    // rollback in reverse
                    while let Some(mut step) = done.pop() {
                        let _ = step.compensate();
                    }
                    return Err(e);
                }
            }
        }
        Ok(())
    }
}
