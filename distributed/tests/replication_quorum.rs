#[test]
fn quorum_rw_smoke() {
    let n = 3;
    let r = 2;
    let w = 2;
    assert!(r + w > n);
}

#[test]
fn quorum_rw_matrix_properties() {
    // 基本多数派性质：若 R+W>N 则线性化读可达；W>N/2 确保提交唯一性
    let cases = vec![
        (3, 2, 2, true),  // 2+2>3
        (5, 3, 3, true),  // 3+3>5
        (4, 2, 2, false), // 2+2==4 非严格大于
        (5, 2, 3, false), // 2+3==5 非严格大于
        (5, 1, 3, false),
    ];
    for (n, r, w, expect_linearizable_read) in cases {
        let linearizable_read = r + w > n;
        assert_eq!(
            linearizable_read, expect_linearizable_read,
            "case n={n}, r={r}, w={w}"
        );
        let unique_commit = w > n / 2; // 严格大于半数
        // 边界检查：n=偶数时 w==n/2 不是多数派
        if n % 2 == 0 && w == n / 2 {
            assert!(!unique_commit);
        }
        if w == (n / 2) + 1 {
            assert!(unique_commit);
        }
    }
}
