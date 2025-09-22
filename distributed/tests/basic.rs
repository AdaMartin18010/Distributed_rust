use c20_distributed::*;

#[test]
fn smoke_exports_exist() {
    let _cfg = DistributedConfig::default();
    let _ = ConsistencyLevel::Quorum;
    let _ = LogicalClock::default();
}
