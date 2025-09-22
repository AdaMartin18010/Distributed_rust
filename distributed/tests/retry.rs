use c20_distributed::transport::{
    InMemoryRpcClient, InMemoryRpcServer, RetryClient, RetryPolicy, RpcClient, RpcServer,
};

#[test]
fn retry_succeeds_after_initial_failure() {
    let mut srv = InMemoryRpcServer::new();
    let attempts = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let attempts_clone = attempts.clone();
    RpcServer::register(
        &mut srv,
        "flaky",
        Box::new(move |_b| {
            let n = attempts_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            if n == 0 {
                return Vec::new();
            }
            b"ok".to_vec()
        }),
    );
    let cli = InMemoryRpcClient::new(srv);
    let retry = RetryClient {
        inner: cli,
        policy: RetryPolicy {
            max_retries: 2,
            retry_on_empty: true,
            backoff_base_ms: None,
        },
    };
    let resp = RpcClient::call(&retry, "flaky", b"").unwrap();
    assert_eq!(resp, b"ok");
}
