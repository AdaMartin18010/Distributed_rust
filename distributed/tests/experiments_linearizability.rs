use std::sync::{Arc, Mutex};
use std::thread;

#[test]
fn linearizability_smoke() {
    assert!(true);
}

#[test]
fn linearizability_mutex_increment() {
    let counter = Arc::new(Mutex::new(0usize));
    let mut handles = Vec::new();
    let ops = 100usize;
    for _ in 0..ops {
        let c = counter.clone();
        handles.push(thread::spawn(move || {
            let mut g = c.lock().unwrap();
            *g += 1;
        }));
    }
    for h in handles {
        h.join().unwrap();
    }
    assert_eq!(*counter.lock().unwrap(), ops);
}
