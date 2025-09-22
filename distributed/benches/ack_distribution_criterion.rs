use c20_distributed::consistency::ConsistencyLevel;
use c20_distributed::replication::MajorityQuorum;
use c20_distributed::replication::QuorumPolicy;
use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;

fn bench_ack(c: &mut Criterion) {
    c.bench_function("ack_quorum_table", |b| {
        b.iter(|| {
            let total = 5usize;
            for ok in 0..=total {
                let need = MajorityQuorum::required_acks(total, ConsistencyLevel::Quorum);
                black_box((ok, need, ok >= need));
            }
        })
    });
}

criterion_group!(benches, bench_ack);
criterion_main!(benches);
