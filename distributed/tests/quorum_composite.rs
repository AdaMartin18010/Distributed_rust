use c20_distributed::consistency::ConsistencyLevel;
use c20_distributed::replication::{CompositeQuorum, MajorityRead, MajorityWrite};

#[test]
fn composite_quorum_read_write_requirements() {
    let n = 5usize;
    let r_need =
        CompositeQuorum::<MajorityRead, MajorityWrite>::required_read(n, ConsistencyLevel::Quorum);
    let w_need =
        CompositeQuorum::<MajorityRead, MajorityWrite>::required_write(n, ConsistencyLevel::Quorum);
    assert_eq!(r_need, 3);
    assert_eq!(w_need, 3);
    // Eventual 下读/写都可退化为 1
    let r_ev = CompositeQuorum::<MajorityRead, MajorityWrite>::required_read(
        n,
        ConsistencyLevel::Eventual,
    );
    let w_ev = CompositeQuorum::<MajorityRead, MajorityWrite>::required_write(
        n,
        ConsistencyLevel::Eventual,
    );
    assert_eq!(r_ev, 1);
    assert_eq!(w_ev, 1);
}
