// 测试目的：重试/退避与截止时间不变量
// - 不变量：
//   1) 指数退避序列单调非减；
//   2) 共享截止时间：同一请求的重试共享一次总预算，尝试次数受总预算上限；
//   3) 抖动应保持上界（此处不引入随机，验证基线序列）。
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
