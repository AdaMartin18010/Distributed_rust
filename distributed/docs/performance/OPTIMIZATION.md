# 性能优化技巧

本文档提供了分布式系统性能优化的详细指南，包括理论分析、实践技巧和具体实现。

## 🎯 性能优化目标

### 关键指标

- **延迟 (Latency)**: P50, P95, P99 延迟
- **吞吐量 (Throughput)**: 每秒操作数 (OPS)
- **资源利用率**: CPU, 内存, 网络, 磁盘
- **错误率**: 失败请求比例
- **可用性**: 系统正常运行时间

### 性能基准

```rust
// 性能基准测试
use criterion::{criterion_group, criterion_main, Criterion};

fn benchmark_replication(c: &mut Criterion) {
    let mut group = c.benchmark_group("replication");
    
    group.bench_function("quorum_write", |b| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let replicator = LocalReplicator::new(5, 3, 3);
        
        b.iter(|| {
            rt.block_on(async {
                replicator.replicate("key", "value", ConsistencyLevel::Quorum).await
            })
        });
    });
    
    group.bench_function("strong_consistency_read", |b| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let replicator = LocalReplicator::new(5, 3, 3);
        
        b.iter(|| {
            rt.block_on(async {
                replicator.read("key", ConsistencyLevel::Strong).await
            })
        });
    });
    
    group.finish();
}

criterion_group!(benches, benchmark_replication);
criterion_main!(benches);
```

## 🔄 复制优化

### 1. 批量复制

减少网络往返次数，提高吞吐量。

```rust
struct BatchReplicator {
    batch_size: usize,
    batch_timeout: Duration,
    pending_operations: Arc<Mutex<Vec<ReplicationOperation>>>,
    replicator: LocalReplicator,
}

impl BatchReplicator {
    async fn replicate_batch(&self, operations: Vec<ReplicationOperation>) -> Result<(), Error> {
        // 按目标节点分组
        let mut grouped_operations = HashMap::new();
        
        for op in operations {
            for target_node in &op.target_nodes {
                grouped_operations
                    .entry(target_node.clone())
                    .or_insert_with(Vec::new)
                    .push(op.clone());
            }
        }
        
        // 并行发送到各个节点
        let mut handles = Vec::new();
        
        for (target_node, ops) in grouped_operations {
            let replicator = self.replicator.clone();
            let handle = tokio::spawn(async move {
                replicator.send_batch(target_node, ops).await
            });
            handles.push(handle);
        }
        
        // 等待所有批次完成
        for handle in handles {
            handle.await??;
        }
        
        Ok(())
    }
    
    async fn start_batch_processor(&self) -> Result<(), Error> {
        let mut interval = tokio::time::interval(self.batch_timeout);
        
        loop {
            interval.tick().await;
            
            let mut pending = self.pending_operations.lock().await;
            if !pending.is_empty() {
                let operations = pending.drain(..).collect();
                drop(pending);
                
                self.replicate_batch(operations).await?;
            }
        }
    }
}
```

### 2. 异步复制

使用异步操作提高并发性。

```rust
struct AsyncReplicator {
    replicator: LocalReplicator,
    completion_queue: Arc<Mutex<Vec<CompletionCallback>>>,
}

impl AsyncReplicator {
    async fn replicate_async(&self, key: &str, value: &[u8], level: ConsistencyLevel) -> Result<(), Error> {
        // 启动异步复制
        let replicator = self.replicator.clone();
        let key = key.to_string();
        let value = value.to_vec();
        
        tokio::spawn(async move {
            match replicator.replicate(&key, &value, level).await {
                Ok(_) => {
                    // 复制成功，调用完成回调
                    if let Some(callback) = self.completion_queue.lock().await.pop() {
                        callback.on_success();
                    }
                }
                Err(e) => {
                    // 复制失败，调用错误回调
                    if let Some(callback) = self.completion_queue.lock().await.pop() {
                        callback.on_error(e);
                    }
                }
            }
        });
        
        Ok(())
    }
}
```

### 3. 压缩优化

减少网络传输数据量。

```rust
struct CompressedReplicator {
    replicator: LocalReplicator,
    compression_algorithm: CompressionAlgorithm,
    compression_threshold: usize,
}

impl CompressedReplicator {
    async fn replicate_compressed(&self, key: &str, value: &[u8], level: ConsistencyLevel) -> Result<(), Error> {
        // 检查是否需要压缩
        if value.len() > self.compression_threshold {
            // 压缩数据
            let compressed_value = self.compress(value)?;
            
            // 添加压缩标记
            let mut compressed_data = Vec::new();
            compressed_data.push(0x01); // 压缩标记
            compressed_data.extend_from_slice(&compressed_value);
            
            // 复制压缩数据
            self.replicator.replicate(key, &compressed_data, level).await?;
        } else {
            // 直接复制原始数据
            let mut uncompressed_data = Vec::new();
            uncompressed_data.push(0x00); // 未压缩标记
            uncompressed_data.extend_from_slice(value);
            
            self.replicator.replicate(key, &uncompressed_data, level).await?;
        }
        
        Ok(())
    }
    
    fn compress(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        match self.compression_algorithm {
            CompressionAlgorithm::Gzip => {
                use flate2::write::GzEncoder;
                use flate2::Compression;
                
                let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
                encoder.write_all(data)?;
                encoder.finish().map_err(|e| Error::CompressionFailed(e))
            }
            CompressionAlgorithm::Lz4 => {
                use lz4_flex::{compress, decompress};
                
                compress(data).map_err(|e| Error::CompressionFailed(e.into()))
            }
        }
    }
}
```

## 🗳️ 共识优化

### 1. 批量日志条目

减少网络往返次数。

```rust
struct BatchLogReplicator {
    raft_node: RaftNode,
    batch_size: usize,
    batch_timeout: Duration,
    pending_entries: Arc<Mutex<Vec<LogEntry>>>,
}

impl BatchLogReplicator {
    async fn append_entries_batch(&self, entries: Vec<LogEntry>) -> Result<(), Error> {
        // 按目标节点分组
        let mut grouped_entries = HashMap::new();
        
        for entry in entries {
            for follower in &self.raft_node.get_followers() {
                grouped_entries
                    .entry(follower.clone())
                    .or_insert_with(Vec::new)
                    .push(entry.clone());
            }
        }
        
        // 并行发送到各个跟随者
        let mut handles = Vec::new();
        
        for (follower, entries) in grouped_entries {
            let raft_node = self.raft_node.clone();
            let handle = tokio::spawn(async move {
                raft_node.send_append_entries(follower, entries).await
            });
            handles.push(handle);
        }
        
        // 等待所有发送完成
        for handle in handles {
            handle.await??;
        }
        
        Ok(())
    }
    
    async fn start_batch_processor(&self) -> Result<(), Error> {
        let mut interval = tokio::time::interval(self.batch_timeout);
        
        loop {
            interval.tick().await;
            
            let mut pending = self.pending_entries.lock().await;
            if !pending.is_empty() {
                let entries = pending.drain(..).collect();
                drop(pending);
                
                self.append_entries_batch(entries).await?;
            }
        }
    }
}
```

### 2. 流水线复制

使用流水线技术提高吞吐量。

```rust
struct PipelineReplicator {
    raft_node: RaftNode,
    pipeline_depth: usize,
    in_flight_requests: Arc<Mutex<HashMap<u64, InFlightRequest>>>,
    next_request_id: AtomicU64,
}

impl PipelineReplicator {
    async fn replicate_pipeline(&self, entries: Vec<LogEntry>) -> Result<(), Error> {
        let mut handles = Vec::new();
        
        for entry in entries {
            // 检查流水线深度
            while self.in_flight_requests.lock().await.len() >= self.pipeline_depth {
                // 等待一些请求完成
                self.wait_for_completion().await?;
            }
            
            // 发送请求
            let request_id = self.next_request_id.fetch_add(1, Ordering::SeqCst);
            let handle = self.send_entry_pipeline(entry, request_id).await?;
            handles.push(handle);
        }
        
        // 等待所有请求完成
        for handle in handles {
            handle.await??;
        }
        
        Ok(())
    }
    
    async fn send_entry_pipeline(&self, entry: LogEntry, request_id: u64) -> Result<JoinHandle<Result<(), Error>>, Error> {
        let raft_node = self.raft_node.clone();
        let in_flight_requests = self.in_flight_requests.clone();
        
        let handle = tokio::spawn(async move {
            // 发送请求
            let result = raft_node.send_append_entries_single(entry).await;
            
            // 从飞行中请求中移除
            in_flight_requests.lock().await.remove(&request_id);
            
            result
        });
        
        // 记录飞行中请求
        self.in_flight_requests.lock().await.insert(request_id, InFlightRequest {
            start_time: Instant::now(),
            entry_count: 1,
        });
        
        Ok(handle)
    }
}
```

### 3. 快照优化

优化快照创建和传输。

```rust
struct OptimizedSnapshotManager {
    storage: StorageBackend,
    snapshot_threshold: usize,
    compression_enabled: bool,
    incremental_snapshots: bool,
}

impl OptimizedSnapshotManager {
    async fn create_snapshot(&self) -> Result<Snapshot, Error> {
        if self.incremental_snapshots {
            self.create_incremental_snapshot().await
        } else {
            self.create_full_snapshot().await
        }
    }
    
    async fn create_incremental_snapshot(&self) -> Result<Snapshot, Error> {
        // 获取上次快照的元数据
        let last_snapshot = self.storage.get_last_snapshot().await?;
        
        // 计算增量数据
        let incremental_data = self.storage.get_incremental_data(
            last_snapshot.last_applied_index
        ).await?;
        
        // 创建增量快照
        let snapshot = Snapshot {
            last_applied_index: self.storage.get_last_applied_index().await?,
            data: incremental_data,
            is_incremental: true,
            base_snapshot_id: last_snapshot.id,
        };
        
        // 压缩快照数据
        if self.compression_enabled {
            snapshot.data = self.compress_snapshot_data(&snapshot.data)?;
        }
        
        Ok(snapshot)
    }
    
    async fn create_full_snapshot(&self) -> Result<Snapshot, Error> {
        // 创建完整快照
        let snapshot = Snapshot {
            last_applied_index: self.storage.get_last_applied_index().await?,
            data: self.storage.get_all_data().await?,
            is_incremental: false,
            base_snapshot_id: None,
        };
        
        // 压缩快照数据
        if self.compression_enabled {
            snapshot.data = self.compress_snapshot_data(&snapshot.data)?;
        }
        
        Ok(snapshot)
    }
}
```

## 💰 事务优化

### 1. 乐观并发控制

减少锁竞争，提高并发性。

```rust
struct OptimisticConcurrencyControl {
    version_store: VersionStore,
    conflict_resolver: ConflictResolver,
}

impl OptimisticConcurrencyControl {
    async fn execute_transaction<T>(&self, transaction: T) -> Result<(), Error>
    where
        T: Transaction,
    {
        loop {
            // 1. 读取数据版本
            let versions = self.version_store.get_versions(&transaction.get_keys()).await?;
            
            // 2. 执行事务逻辑
            let result = transaction.execute().await?;
            
            // 3. 尝试提交
            match self.version_store.try_commit(&transaction.get_keys(), &versions, &result).await {
                Ok(_) => return Ok(()),
                Err(Error::VersionConflict) => {
                    // 4. 解决冲突
                    self.conflict_resolver.resolve_conflict(&transaction, &versions).await?;
                    
                    // 5. 重试事务
                    continue;
                }
                Err(e) => return Err(e),
            }
        }
    }
}
```

### 2. 事务批处理

批量处理事务，提高吞吐量。

```rust
struct TransactionBatchProcessor {
    batch_size: usize,
    batch_timeout: Duration,
    pending_transactions: Arc<Mutex<Vec<Transaction>>>,
    transaction_executor: TransactionExecutor,
}

impl TransactionBatchProcessor {
    async fn process_transaction_batch(&self, transactions: Vec<Transaction>) -> Result<(), Error> {
        // 分析事务依赖
        let dependency_graph = self.analyze_dependencies(&transactions);
        
        // 按依赖顺序执行事务
        let execution_order = self.topological_sort(&dependency_graph);
        
        // 并行执行无依赖的事务
        let mut handles = Vec::new();
        
        for level in execution_order {
            let mut level_handles = Vec::new();
            
            for transaction_id in level {
                let transaction = transactions[transaction_id].clone();
                let executor = self.transaction_executor.clone();
                
                let handle = tokio::spawn(async move {
                    executor.execute(transaction).await
                });
                
                level_handles.push(handle);
            }
            
            // 等待当前级别的事务完成
            for handle in level_handles {
                handle.await??;
            }
        }
        
        Ok(())
    }
    
    fn analyze_dependencies(&self, transactions: &[Transaction]) -> DependencyGraph {
        let mut graph = DependencyGraph::new();
        
        for (i, transaction) in transactions.iter().enumerate() {
            for (j, other_transaction) in transactions.iter().enumerate() {
                if i != j && transaction.has_conflict_with(other_transaction) {
                    graph.add_edge(i, j);
                }
            }
        }
        
        graph
    }
}
```

## 🔍 故障检测优化

### 1. 自适应故障检测

根据网络条件调整检测参数。

```rust
struct AdaptiveFailureDetector {
    base_timeout: Duration,
    network_quality: NetworkQualityMonitor,
    timeout_multiplier: f64,
}

impl AdaptiveFailureDetector {
    async fn detect_failure(&self, target: NodeId) -> Result<FailureStatus, Error> {
        // 根据网络质量调整超时
        let adjusted_timeout = self.calculate_adjusted_timeout().await;
        
        // 执行故障检测
        match tokio::time::timeout(adjusted_timeout, self.ping_target(target)).await {
            Ok(result) => result,
            Err(_) => {
                // 超时，标记为可疑
                Ok(FailureStatus::Suspect)
            }
        }
    }
    
    async fn calculate_adjusted_timeout(&self) -> Duration {
        let network_quality = self.network_quality.get_current_quality().await;
        
        let multiplier = match network_quality {
            NetworkQuality::Excellent => 1.0,
            NetworkQuality::Good => 1.2,
            NetworkQuality::Fair => 1.5,
            NetworkQuality::Poor => 2.0,
            NetworkQuality::VeryPoor => 3.0,
        };
        
        let adjusted_timeout = self.base_timeout.mul_f64(multiplier);
        adjusted_timeout
    }
}
```

### 2. 预测性故障检测

使用机器学习预测故障。

```rust
struct PredictiveFailureDetector {
    ml_model: FailurePredictionModel,
    metrics_collector: MetricsCollector,
    prediction_threshold: f64,
}

impl PredictiveFailureDetector {
    async fn predict_failure(&self, target: NodeId) -> Result<FailureProbability, Error> {
        // 收集历史指标
        let metrics = self.metrics_collector.get_metrics(target).await?;
        
        // 提取特征
        let features = self.extract_features(&metrics);
        
        // 使用 ML 模型预测
        let probability = self.ml_model.predict(&features)?;
        
        Ok(FailureProbability {
            node_id: target,
            probability,
            confidence: self.ml_model.get_confidence(&features),
        })
    }
    
    fn extract_features(&self, metrics: &NodeMetrics) -> Vec<f64> {
        vec![
            metrics.cpu_usage,
            metrics.memory_usage,
            metrics.network_latency,
            metrics.response_time_p99,
            metrics.error_rate,
            metrics.request_rate,
        ]
    }
}
```

## ⚖️ 负载均衡优化

### 1. 动态权重调整

根据节点性能动态调整权重。

```rust
struct DynamicWeightBalancer {
    services: Arc<Mutex<Vec<WeightedService>>>,
    performance_monitor: PerformanceMonitor,
    weight_update_interval: Duration,
}

impl DynamicWeightBalancer {
    async fn start_weight_updater(&self) -> Result<(), Error> {
        let mut interval = tokio::time::interval(self.weight_update_interval);
        
        loop {
            interval.tick().await;
            
            // 更新服务权重
            self.update_service_weights().await?;
        }
    }
    
    async fn update_service_weights(&self) -> Result<(), Error> {
        let mut services = self.services.lock().await;
        
        for service in services.iter_mut() {
            // 获取性能指标
            let metrics = self.performance_monitor.get_metrics(&service.id).await?;
            
            // 计算新权重
            let new_weight = self.calculate_weight(&metrics);
            
            // 平滑更新权重
            service.weight = (service.weight * 0.7) + (new_weight * 0.3);
        }
        
        Ok(())
    }
    
    fn calculate_weight(&self, metrics: &ServiceMetrics) -> u32 {
        let mut weight = 100; // 基础权重
        
        // 根据响应时间调整
        if metrics.avg_response_time < Duration::from_millis(100) {
            weight += 20;
        } else if metrics.avg_response_time > Duration::from_millis(500) {
            weight -= 30;
        }
        
        // 根据错误率调整
        if metrics.error_rate < 0.01 {
            weight += 10;
        } else if metrics.error_rate > 0.05 {
            weight -= 20;
        }
        
        // 根据 CPU 使用率调整
        if metrics.cpu_usage < 50.0 {
            weight += 15;
        } else if metrics.cpu_usage > 80.0 {
            weight -= 25;
        }
        
        weight.max(10).min(200) // 限制权重范围
    }
}
```

### 2. 智能路由

根据请求特征选择最佳节点。

```rust
struct IntelligentRouter {
    routing_rules: Vec<RoutingRule>,
    performance_predictor: PerformancePredictor,
    cost_calculator: CostCalculator,
}

impl IntelligentRouter {
    async fn route_request(&self, request: &Request) -> Result<NodeId, Error> {
        let mut candidates = Vec::new();
        
        // 应用路由规则
        for rule in &self.routing_rules {
            if rule.matches(request) {
                let nodes = rule.get_candidate_nodes();
                candidates.extend(nodes);
            }
        }
        
        if candidates.is_empty() {
            return Err(Error::NoAvailableNodes);
        }
        
        // 预测性能
        let mut scored_candidates = Vec::new();
        
        for node_id in candidates {
            let predicted_performance = self.performance_predictor.predict(node_id, request).await?;
            let cost = self.cost_calculator.calculate_cost(node_id, request).await?;
            
            let score = self.calculate_score(&predicted_performance, cost);
            scored_candidates.push((node_id, score));
        }
        
        // 选择最佳节点
        scored_candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        
        Ok(scored_candidates[0].0)
    }
    
    fn calculate_score(&self, performance: &PerformancePrediction, cost: f64) -> f64 {
        let latency_score = 1.0 / (performance.estimated_latency.as_secs_f64() + 0.001);
        let throughput_score = performance.estimated_throughput;
        let reliability_score = performance.estimated_reliability;
        let cost_score = 1.0 / (cost + 0.001);
        
        // 加权计算总分
        (latency_score * 0.3) + (throughput_score * 0.2) + (reliability_score * 0.3) + (cost_score * 0.2)
    }
}
```

## 🛡️ 安全优化

### 1. 连接池安全

安全地管理连接池。

```rust
struct SecureConnectionPool {
    connections: Arc<Mutex<Vec<SecureConnection>>>,
    max_size: usize,
    min_size: usize,
    connection_timeout: Duration,
    idle_timeout: Duration,
    encryption_manager: EncryptionManager,
}

impl SecureConnectionPool {
    async fn get_secure_connection(&self) -> Result<SecureConnection, Error> {
        let mut connections = self.connections.lock().await;
        
        // 尝试从池中获取连接
        if let Some(connection) = connections.pop() {
            if connection.is_healthy() && !connection.is_expired() {
                return Ok(connection);
            }
        }
        
        // 创建新连接
        let connection = self.create_secure_connection().await?;
        Ok(connection)
    }
    
    async fn create_secure_connection(&self) -> Result<SecureConnection, Error> {
        // 建立安全连接
        let mut connection = SecureConnection::new().await?;
        
        // 执行 TLS 握手
        connection.perform_tls_handshake().await?;
        
        // 验证证书
        connection.verify_certificate().await?;
        
        // 设置加密
        connection.setup_encryption(&self.encryption_manager).await?;
        
        Ok(connection)
    }
}
```

### 2. 令牌桶优化

高效实现令牌桶算法。

```rust
struct OptimizedTokenBucket {
    capacity: u64,
    tokens: AtomicU64,
    last_refill: AtomicU64,
    refill_rate: u64,
    refill_interval: Duration,
}

impl OptimizedTokenBucket {
    fn new(capacity: u64, refill_rate: u64, refill_interval: Duration) -> Self {
        Self {
            capacity,
            tokens: AtomicU64::new(capacity),
            last_refill: AtomicU64::new(now()),
            refill_rate,
            refill_interval,
        }
    }
    
    fn try_consume(&self, tokens: u64) -> bool {
        let now = now();
        let last_refill = self.last_refill.load(Ordering::SeqCst);
        
        // 计算需要补充的令牌数
        let elapsed = now - last_refill;
        let refill_tokens = (elapsed * self.refill_rate) / self.refill_interval.as_nanos() as u64;
        
        if refill_tokens > 0 {
            // 更新最后补充时间
            self.last_refill.store(now, Ordering::SeqCst);
            
            // 补充令牌
            let current_tokens = self.tokens.load(Ordering::SeqCst);
            let new_tokens = (current_tokens + refill_tokens).min(self.capacity);
            self.tokens.store(new_tokens, Ordering::SeqCst);
        }
        
        // 尝试消费令牌
        let current_tokens = self.tokens.load(Ordering::SeqCst);
        if current_tokens >= tokens {
            self.tokens.fetch_sub(tokens, Ordering::SeqCst);
            true
        } else {
            false
        }
    }
}

fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64
}
```

## 📊 监控优化

### 1. 高效指标收集

优化指标收集性能。

```rust
struct OptimizedMetricsCollector {
    counters: DashMap<String, AtomicU64>,
    histograms: DashMap<String, AtomicHistogram>,
    gauges: DashMap<String, AtomicU64>,
    collection_interval: Duration,
}

impl OptimizedMetricsCollector {
    fn new(collection_interval: Duration) -> Self {
        Self {
            counters: DashMap::new(),
            histograms: DashMap::new(),
            gauges: DashMap::new(),
            collection_interval,
        }
    }
    
    fn increment_counter(&self, name: &str, value: u64) {
        self.counters
            .entry(name.to_string())
            .or_insert_with(|| AtomicU64::new(0))
            .fetch_add(value, Ordering::SeqCst);
    }
    
    fn record_histogram(&self, name: &str, value: f64) {
        self.histograms
            .entry(name.to_string())
            .or_insert_with(|| AtomicHistogram::new())
            .record(value);
    }
    
    fn set_gauge(&self, name: &str, value: u64) {
        self.gauges
            .entry(name.to_string())
            .or_insert_with(|| AtomicU64::new(0))
            .store(value, Ordering::SeqCst);
    }
    
    async fn start_collection(&self) -> Result<(), Error> {
        let mut interval = tokio::time::interval(self.collection_interval);
        
        loop {
            interval.tick().await;
            
            // 收集指标
            let metrics = self.collect_metrics().await;
            
            // 导出指标
            self.export_metrics(metrics).await?;
        }
    }
    
    async fn collect_metrics(&self) -> MetricsSnapshot {
        let mut snapshot = MetricsSnapshot::new();
        
        // 收集计数器
        for entry in self.counters.iter() {
            let name = entry.key();
            let value = entry.value().load(Ordering::SeqCst);
            snapshot.add_counter(name, value);
        }
        
        // 收集直方图
        for entry in self.histograms.iter() {
            let name = entry.key();
            let histogram = entry.value();
            snapshot.add_histogram(name, histogram.get_stats());
        }
        
        // 收集仪表
        for entry in self.gauges.iter() {
            let name = entry.key();
            let value = entry.value().load(Ordering::SeqCst);
            snapshot.add_gauge(name, value);
        }
        
        snapshot
    }
}
```

### 2. 智能告警

减少告警噪音，提高告警质量。

```rust
struct IntelligentAlerting {
    alert_rules: Vec<AlertRule>,
    alert_history: AlertHistory,
    alert_cooldown: Duration,
    alert_aggregation: AlertAggregation,
}

impl IntelligentAlerting {
    async fn evaluate_alerts(&self, metrics: &MetricsSnapshot) -> Result<(), Error> {
        for rule in &self.alert_rules {
            if rule.evaluate(metrics) {
                // 检查告警冷却期
                if self.is_in_cooldown(rule) {
                    continue;
                }
                
                // 检查告警聚合
                if self.should_aggregate_alert(rule) {
                    self.aggregate_alert(rule).await?;
                } else {
                    self.send_alert(rule).await?;
                }
                
                // 记录告警历史
                self.alert_history.record_alert(rule);
            }
        }
        
        Ok(())
    }
    
    fn is_in_cooldown(&self, rule: &AlertRule) -> bool {
        if let Some(last_alert) = self.alert_history.get_last_alert(rule) {
            last_alert.timestamp + self.alert_cooldown > Instant::now()
        } else {
            false
        }
    }
    
    fn should_aggregate_alert(&self, rule: &AlertRule) -> bool {
        match self.alert_aggregation {
            AlertAggregation::None => false,
            AlertAggregation::TimeWindow(window) => {
                let recent_alerts = self.alert_history.get_recent_alerts(rule, window);
                recent_alerts.len() > 1
            }
            AlertAggregation::CountThreshold(threshold) => {
                let recent_alerts = self.alert_history.get_recent_alerts(rule, Duration::from_secs(60));
                recent_alerts.len() >= threshold
            }
        }
    }
}
```

## 🚀 性能测试

### 1. 压力测试

```rust
#[tokio::test]
async fn test_replication_performance() {
    let replicator = LocalReplicator::new(5, 3, 3);
    let mut handles = Vec::new();
    
    // 启动多个并发客户端
    for i in 0..100 {
        let replicator = replicator.clone();
        let handle = tokio::spawn(async move {
            let start = Instant::now();
            
            for j in 0..1000 {
                let key = format!("key_{}_{}", i, j);
                let value = format!("value_{}_{}", i, j);
                
                replicator.replicate(&key, &value, ConsistencyLevel::Quorum).await.unwrap();
            }
            
            start.elapsed()
        });
        
        handles.push(handle);
    }
    
    // 等待所有客户端完成
    let mut total_duration = Duration::from_secs(0);
    for handle in handles {
        let duration = handle.await.unwrap();
        total_duration += duration;
    }
    
    // 计算性能指标
    let avg_duration = total_duration / 100;
    let throughput = 100000 / avg_duration.as_secs();
    
    println!("平均延迟: {:?}", avg_duration);
    println!("吞吐量: {} OPS", throughput);
    
    // 验证性能要求
    assert!(avg_duration < Duration::from_secs(10));
    assert!(throughput > 10000);
}
```

### 2. 基准测试

```rust
use criterion::{criterion_group, criterion_main, Criterion};

fn benchmark_consensus(c: &mut Criterion) {
    let mut group = c.benchmark_group("consensus");
    
    group.bench_function("leader_election", |b| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        
        b.iter(|| {
            rt.block_on(async {
                let mut cluster = create_raft_cluster(5).await;
                cluster.kill_leader().await;
                cluster.wait_for_new_leader().await;
            })
        });
    });
    
    group.bench_function("log_replication", |b| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        
        b.iter(|| {
            rt.block_on(async {
                let mut cluster = create_raft_cluster(5).await;
                
                for i in 0..100 {
                    cluster.propose(format!("entry_{}", i)).await;
                }
                
                cluster.wait_for_consensus().await;
            })
        });
    });
    
    group.finish();
}

criterion_group!(benches, benchmark_consensus);
criterion_main!(benches);
```

## 🔗 相关资源

- [快速开始指南](../QUICKSTART.md)
- [系统设计最佳实践](../design/BEST_PRACTICES.md)
- [测试策略](../testing/README.md)
- [监控与可观测性](../observability/README.md)
- [性能基准测试](../benchmarks/README.md)

## 🆘 获取帮助

- **GitHub Issues**: [报告问题](https://github.com/your-org/c20_distributed/issues)
- **Discussions**: [讨论交流](https://github.com/your-org/c20_distributed/discussions)
- **Stack Overflow**: [技术问答](https://stackoverflow.com/questions/tagged/c20-distributed)

---

**优化性能！** 🚀 应用这些优化技巧，提升您的分布式系统性能，实现更高的吞吐量和更低的延迟。
