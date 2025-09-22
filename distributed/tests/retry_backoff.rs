#[test]
fn retry_backoff_sequence_and_deadline_budget() {
    // 指数退避序列：base=5ms, 次数=5，应单调非减
    let base = 5u64;
    let retries = 5u32;
    let mut last = 0u64;
    for i in 0..retries {
        let delay = base * (1u64 << i);
        assert!(delay >= last);
        last = delay;
    }
    // 截止时间预算：总预算 50ms，三次尝试分别消耗 10/20/25，应在第三次之前用尽
    let mut budget = 50i64;
    let costs = [10i64, 20, 25];
    let mut attempts = 0;
    for c in costs {
        if budget - c > 0 {
            budget -= c;
            attempts += 1;
        }
    }
    assert_eq!(attempts, 2);
}
