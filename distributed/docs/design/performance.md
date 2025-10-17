# 3.10.5 性能优化 (Performance Optimization)

## 目录

- [3.10.5 性能优化 (Performance Optimization)](#3105-性能优化-performance-optimization)
  - [目录](#目录)
  - [核心概念](#核心概念)
    - [性能目标](#性能目标)
    - [性能权衡](#性能权衡)
  - [性能度量](#性能度量)
    - [基准测试](#基准测试)
    - [性能分析](#性能分析)
  - [系统性能优化](#系统性能优化)
    - [CPU优化](#cpu优化)
    - [内存优化](#内存优化)
  - [网络性能优化](#网络性能优化)
    - [连接复用](#连接复用)
    - [批处理](#批处理)
    - [压缩](#压缩)
  - [存储性能优化](#存储性能优化)
    - [读写优化](#读写优化)
    - [LSM-Tree优化](#lsm-tree优化)
    - [布隆过滤器](#布隆过滤器)
  - [并发与并行](#并发与并行)
    - [异步并发](#异步并发)
    - [数据并行](#数据并行)
    - [无锁数据结构](#无锁数据结构)
  - [缓存策略](#缓存策略)
    - [多级缓存](#多级缓存)
    - [缓存策略1](#缓存策略1)
    - [缓存一致性](#缓存一致性)
  - [负载均衡](#负载均衡)
    - [负载均衡算法](#负载均衡算法)
  - [性能分析工具](#性能分析工具)
    - [自定义性能监控](#自定义性能监控)
  - [最佳实践](#最佳实践)
    - [性能优化清单](#性能优化清单)
    - [性能目标设定](#性能目标设定)
    - [性能回归检测](#性能回归检测)
  - [相关文档](#相关文档)
  - [参考资料](#参考资料)
    - [工具](#工具)
    - [最佳实践1](#最佳实践1)

---

## 核心概念

性能优化是分布式系统设计中的关键考虑因素，涉及延迟、吞吐量、资源利用率等多个维度。

### 性能目标

**关键指标**：

```text
1. 延迟 (Latency)
   - P50, P95, P99延迟
   - 端到端延迟
   - 网络延迟
   
2. 吞吐量 (Throughput)
   - 每秒请求数 (RPS)
   - 每秒事务数 (TPS)
   - 数据传输速率
   
3. 资源利用率
   - CPU利用率
   - 内存使用
   - 网络带宽
   - 磁盘I/O
   
4. 可扩展性 (Scalability)
   - 垂直扩展能力
   - 水平扩展能力
   - 线性扩展性
```

### 性能权衡

**CAP与性能**：

```text
一致性 vs 延迟：
- 强一致性 → 更高延迟
- 最终一致性 → 更低延迟

可用性 vs 吞吐量：
- 高可用 → 需要冗余 → 降低吞吐
- 高吞吐 → 可能牺牲某些可用性保证

分区容错 vs 性能：
- 跨分区操作 → 网络开销 → 延迟增加
```

**Amdahl定律**：

```text
加速比 = 1 / ((1 - P) + P/N)

其中：
- P: 可并行化部分
- N: 处理器数量

示例：
如果90%的代码可并行，使用10个核心：
加速比 = 1 / ((1 - 0.9) + 0.9/10) = 1 / 0.19 ≈ 5.3x
```

---

## 性能度量

### 基准测试

**Criterion.rs基准测试**：

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};

fn fibonacci_recursive(n: u64) -> u64 {
    match n {
        0 => 0,
        1 => 1,
        n => fibonacci_recursive(n - 1) + fibonacci_recursive(n - 2),
    }
}

fn fibonacci_iterative(n: u64) -> u64 {
    let mut a = 0;
    let mut b = 1;
    for _ in 0..n {
        let tmp = a;
        a = b;
        b = tmp + b;
    }
    a
}

fn benchmark_fibonacci(c: &mut Criterion) {
    let mut group = c.benchmark_group("fibonacci");
    
    for n in [10, 20, 25].iter() {
        group.bench_with_input(BenchmarkId::new("recursive", n), n, |b, &n| {
            b.iter(|| fibonacci_recursive(black_box(n)));
        });
        
        group.bench_with_input(BenchmarkId::new("iterative", n), n, |b, &n| {
            b.iter(|| fibonacci_iterative(black_box(n)));
        });
    }
    
    group.finish();
}

criterion_group!(benches, benchmark_fibonacci);
criterion_main!(benches);
```

### 性能分析

**Profiling工具**：

```text
1. CPU Profiling
   - perf (Linux)
   - Instruments (macOS)
   - cargo-flamegraph
   
2. 内存分析
   - Valgrind
   - heaptrack
   - cargo-profdata
   
3. 异步运行时
   - tokio-console
   - async-profiler
```

**火焰图生成**：

```bash
# 安装cargo-flamegraph
cargo install flamegraph

# 生成火焰图
cargo flamegraph --bench my_benchmark

# 生成调用图
perf record --call-graph dwarf ./target/release/my_app
perf report
```

---

## 系统性能优化

### CPU优化

**算法优化**：

```rust
// 低效：O(n²)
fn has_duplicates_naive(nums: &[i32]) -> bool {
    for i in 0..nums.len() {
        for j in (i + 1)..nums.len() {
            if nums[i] == nums[j] {
                return true;
            }
        }
    }
    false
}

// 高效：O(n)
fn has_duplicates_optimized(nums: &[i32]) -> bool {
    use std::collections::HashSet;
    let mut seen = HashSet::new();
    for &num in nums {
        if !seen.insert(num) {
            return true;
        }
    }
    false
}
```

**SIMD优化**：

```rust
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

// 使用SIMD加速向量加法
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn add_vectors_simd(a: &[f32], b: &[f32], result: &mut [f32]) {
    assert_eq!(a.len(), b.len());
    assert_eq!(a.len(), result.len());
    assert_eq!(a.len() % 8, 0); // AVX2处理8个f32
    
    for i in (0..a.len()).step_by(8) {
        let va = _mm256_loadu_ps(&a[i]);
        let vb = _mm256_loadu_ps(&b[i]);
        let vr = _mm256_add_ps(va, vb);
        _mm256_storeu_ps(&mut result[i], vr);
    }
}

// 标量版本
fn add_vectors_scalar(a: &[f32], b: &[f32], result: &mut [f32]) {
    for i in 0..a.len() {
        result[i] = a[i] + b[i];
    }
}
```

**分支预测优化**：

```rust
// 使用likely/unlikely宏（需要nightly）
#[cfg(feature = "unstable")]
use core::intrinsics::{likely, unlikely};

fn process_value(value: i32) -> i32 {
    // 告诉编译器这个分支更可能执行
    if likely(value > 0) {
        value * 2
    } else {
        value
    }
}

// 使用branchless技术
fn abs_branchless(x: i32) -> i32 {
    let mask = x >> 31;
    (x ^ mask) - mask
}
```

### 内存优化

**内存布局优化**：

```rust
// 缓存不友好：结构体过大，字段对齐问题
#[repr(C)]
struct BadLayout {
    flag1: bool,    // 1 byte + 7 bytes padding
    value: u64,     // 8 bytes
    flag2: bool,    // 1 byte + 7 bytes padding
    data: [u8; 64], // 64 bytes
}
// 总大小: 88 bytes (含padding)

// 缓存友好：重新排列字段
#[repr(C)]
struct GoodLayout {
    value: u64,     // 8 bytes
    data: [u8; 64], // 64 bytes
    flag1: bool,    // 1 byte
    flag2: bool,    // 1 byte
    // 6 bytes padding
}
// 总大小: 80 bytes (含padding)

// 紧凑布局
#[repr(packed)]
struct CompactLayout {
    flag1: bool,
    value: u64,
    flag2: bool,
    data: [u8; 64],
}
// 总大小: 74 bytes (无padding，但访问可能较慢)
```

**内存池**：

```rust
use std::alloc::{alloc, dealloc, Layout};
use std::ptr::NonNull;

pub struct MemoryPool<T> {
    free_list: Vec<NonNull<T>>,
    layout: Layout,
    capacity: usize,
}

impl<T> MemoryPool<T> {
    pub fn new(capacity: usize) -> Self {
        let layout = Layout::new::<T>();
        let mut free_list = Vec::with_capacity(capacity);
        
        // 预分配内存块
        unsafe {
            for _ in 0..capacity {
                let ptr = alloc(layout) as *mut T;
                if let Some(nn) = NonNull::new(ptr) {
                    free_list.push(nn);
                }
            }
        }
        
        Self {
            free_list,
            layout,
            capacity,
        }
    }
    
    pub fn allocate(&mut self) -> Option<NonNull<T>> {
        self.free_list.pop()
    }
    
    pub fn deallocate(&mut self, ptr: NonNull<T>) {
        if self.free_list.len() < self.capacity {
            self.free_list.push(ptr);
        } else {
            unsafe {
                dealloc(ptr.as_ptr() as *mut u8, self.layout);
            }
        }
    }
}

impl<T> Drop for MemoryPool<T> {
    fn drop(&mut self) {
        unsafe {
            for ptr in &self.free_list {
                dealloc(ptr.as_ptr() as *mut u8, self.layout);
            }
        }
    }
}
```

**零拷贝技术**：

```rust
use bytes::{Bytes, BytesMut, Buf, BufMut};

// 使用Bytes实现零拷贝
pub fn process_message(data: Bytes) -> Bytes {
    // Bytes支持cheap clone，不会复制底层数据
    let cloned = data.clone();
    
    // 可以slice数据而不复制
    let header = data.slice(0..8);
    let body = data.slice(8..);
    
    // 返回数据不需要复制
    body
}

// 使用共享内存映射
use memmap2::Mmap;
use std::fs::File;

pub fn read_large_file_zero_copy(path: &str) -> Result<Mmap, std::io::Error> {
    let file = File::open(path)?;
    // 内存映射，操作系统按需加载
    unsafe { Mmap::map(&file) }
}
```

---

## 网络性能优化

### 连接复用

**HTTP/2多路复用**：

```rust
use hyper::{Client, Body};
use hyper::client::HttpConnector;
use hyper_tls::HttpsConnector;

pub struct HttpClient {
    client: Client<HttpsConnector<HttpConnector>>,
}

impl HttpClient {
    pub fn new() -> Self {
        let https = HttpsConnector::new();
        
        // HTTP/2支持，连接池，连接复用
        let client = Client::builder()
            .http2_only(true)
            .pool_max_idle_per_host(10)
            .build::<_, Body>(https);
        
        Self { client }
    }
    
    // 多个请求复用同一连接
    pub async fn batch_requests(&self, urls: Vec<String>) -> Vec<String> {
        use futures::future::join_all;
        
        let futures = urls.into_iter().map(|url| {
            let client = self.client.clone();
            async move {
                let response = client.get(url.parse().unwrap()).await.ok()?;
                let bytes = hyper::body::to_bytes(response.into_body()).await.ok()?;
                String::from_utf8(bytes.to_vec()).ok()
            }
        });
        
        join_all(futures).await.into_iter().flatten().collect()
    }
}
```

### 批处理

**请求批处理**：

```rust
use tokio::sync::mpsc;
use tokio::time::{interval, Duration};

pub struct BatchProcessor<T> {
    batch: Vec<T>,
    batch_size: usize,
    flush_interval: Duration,
}

impl<T> BatchProcessor<T> {
    pub fn new(batch_size: usize, flush_interval: Duration) -> Self {
        Self {
            batch: Vec::with_capacity(batch_size),
            batch_size,
            flush_interval,
        }
    }
    
    pub async fn process(
        &mut self,
        mut rx: mpsc::Receiver<T>,
        processor: impl Fn(Vec<T>),
    ) {
        let mut timer = interval(self.flush_interval);
        
        loop {
            tokio::select! {
                Some(item) = rx.recv() => {
                    self.batch.push(item);
                    
                    // 达到批次大小，立即处理
                    if self.batch.len() >= self.batch_size {
                        let batch = std::mem::take(&mut self.batch);
                        processor(batch);
                        self.batch = Vec::with_capacity(self.batch_size);
                    }
                }
                _ = timer.tick() => {
                    // 定时刷新
                    if !self.batch.is_empty() {
                        let batch = std::mem::take(&mut self.batch);
                        processor(batch);
                        self.batch = Vec::with_capacity(self.batch_size);
                    }
                }
            }
        }
    }
}
```

### 压缩

**数据压缩**：

```rust
use flate2::Compression;
use flate2::write::{GzEncoder, GzDecoder};
use std::io::Write;

pub struct CompressionService {
    compression_level: Compression,
    min_size: usize, // 最小压缩大小阈值
}

impl CompressionService {
    pub fn new(level: u32, min_size: usize) -> Self {
        Self {
            compression_level: Compression::new(level),
            min_size,
        }
    }
    
    pub fn compress(&self, data: &[u8]) -> Result<Vec<u8>, std::io::Error> {
        // 小数据不压缩（压缩开销大于收益）
        if data.len() < self.min_size {
            return Ok(data.to_vec());
        }
        
        let mut encoder = GzEncoder::new(Vec::new(), self.compression_level);
        encoder.write_all(data)?;
        encoder.finish()
    }
    
    pub fn decompress(&self, data: &[u8]) -> Result<Vec<u8>, std::io::Error> {
        if data.len() < self.min_size {
            return Ok(data.to_vec());
        }
        
        let mut decoder = GzDecoder::new(Vec::new());
        decoder.write_all(data)?;
        decoder.finish()
    }
    
    // 自适应压缩：根据压缩比决定是否使用
    pub fn adaptive_compress(&self, data: &[u8]) -> Result<Vec<u8>, std::io::Error> {
        let compressed = self.compress(data)?;
        
        // 压缩比小于0.9才使用压缩数据
        if compressed.len() as f64 / data.len() as f64 < 0.9 {
            Ok(compressed)
        } else {
            Ok(data.to_vec())
        }
    }
}
```

---

## 存储性能优化

### 读写优化

**顺序写入**：

```rust
use std::io::{Write, BufWriter};
use std::fs::File;

pub struct SequentialWriter {
    writer: BufWriter<File>,
    buffer_size: usize,
}

impl SequentialWriter {
    pub fn new(path: &str, buffer_size: usize) -> std::io::Result<Self> {
        let file = File::create(path)?;
        let writer = BufWriter::with_capacity(buffer_size, file);
        
        Ok(Self {
            writer,
            buffer_size,
        })
    }
    
    pub fn append(&mut self, data: &[u8]) -> std::io::Result<()> {
        // 顺序写入，利用OS页缓存
        self.writer.write_all(data)?;
        Ok(())
    }
    
    pub fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }
}
```

**预读取**：

```rust
use std::io::{Read, BufReader};
use std::fs::File;

pub struct PrefetchReader {
    reader: BufReader<File>,
    prefetch_size: usize,
}

impl PrefetchReader {
    pub fn new(path: &str, prefetch_size: usize) -> std::io::Result<Self> {
        let file = File::open(path)?;
        let reader = BufReader::with_capacity(prefetch_size, file);
        
        Ok(Self {
            reader,
            prefetch_size,
        })
    }
    
    pub fn read_chunk(&mut self) -> std::io::Result<Vec<u8>> {
        let mut buffer = vec![0u8; self.prefetch_size];
        let n = self.reader.read(&mut buffer)?;
        buffer.truncate(n);
        Ok(buffer)
    }
}
```

### LSM-Tree优化

**分层存储**：

```rust
use std::collections::BTreeMap;

pub struct LSMTree {
    memtable: BTreeMap<String, String>,
    memtable_size: usize,
    max_memtable_size: usize,
    sstables: Vec<SSTable>,
}

pub struct SSTable {
    level: usize,
    data: BTreeMap<String, String>,
}

impl LSMTree {
    pub fn new(max_memtable_size: usize) -> Self {
        Self {
            memtable: BTreeMap::new(),
            memtable_size: 0,
            max_memtable_size,
            sstables: Vec::new(),
        }
    }
    
    pub fn put(&mut self, key: String, value: String) -> Result<(), std::io::Error> {
        self.memtable_size += key.len() + value.len();
        self.memtable.insert(key, value);
        
        // Memtable满了，flush到SSTable
        if self.memtable_size >= self.max_memtable_size {
            self.flush()?;
        }
        
        Ok(())
    }
    
    pub fn get(&self, key: &str) -> Option<String> {
        // 先查memtable
        if let Some(value) = self.memtable.get(key) {
            return Some(value.clone());
        }
        
        // 再查SSTables（从新到旧）
        for sstable in self.sstables.iter().rev() {
            if let Some(value) = sstable.data.get(key) {
                return Some(value.clone());
            }
        }
        
        None
    }
    
    fn flush(&mut self) -> Result<(), std::io::Error> {
        // 将memtable写入SSTable
        let sstable = SSTable {
            level: 0,
            data: std::mem::take(&mut self.memtable),
        };
        
        self.sstables.push(sstable);
        self.memtable_size = 0;
        
        // 触发compaction
        if self.sstables.len() > 4 {
            self.compact()?;
        }
        
        Ok(())
    }
    
    fn compact(&mut self) -> Result<(), std::io::Error> {
        // 合并多个SSTable
        let mut merged = BTreeMap::new();
        
        for sstable in &self.sstables {
            for (k, v) in &sstable.data {
                merged.insert(k.clone(), v.clone());
            }
        }
        
        self.sstables.clear();
        self.sstables.push(SSTable {
            level: 1,
            data: merged,
        });
        
        Ok(())
    }
}
```

### 布隆过滤器

```rust
use bit_vec::BitVec;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

pub struct BloomFilter {
    bits: BitVec,
    num_hashes: usize,
}

impl BloomFilter {
    pub fn new(size: usize, num_hashes: usize) -> Self {
        Self {
            bits: BitVec::from_elem(size, false),
            num_hashes,
        }
    }
    
    pub fn insert<T: Hash>(&mut self, item: &T) {
        for i in 0..self.num_hashes {
            let hash = self.hash(item, i);
            let index = (hash as usize) % self.bits.len();
            self.bits.set(index, true);
        }
    }
    
    pub fn contains<T: Hash>(&self, item: &T) -> bool {
        for i in 0..self.num_hashes {
            let hash = self.hash(item, i);
            let index = (hash as usize) % self.bits.len();
            if !self.bits.get(index).unwrap() {
                return false;
            }
        }
        true
    }
    
    fn hash<T: Hash>(&self, item: &T, seed: usize) -> u64 {
        let mut hasher = DefaultHasher::new();
        item.hash(&mut hasher);
        seed.hash(&mut hasher);
        hasher.finish()
    }
}

// 在LSM-Tree中使用布隆过滤器
impl SSTable {
    pub fn with_bloom_filter(mut self) -> Self {
        let mut bloom = BloomFilter::new(self.data.len() * 10, 3);
        for key in self.data.keys() {
            bloom.insert(key);
        }
        // 在实际实现中，bloom filter应该是SSTable的一个字段
        self
    }
    
    pub fn might_contain(&self, key: &str, bloom: &BloomFilter) -> bool {
        bloom.contains(&key)
    }
}
```

---

## 并发与并行

### 异步并发

**Tokio任务调度**：

```rust
use tokio::task;
use std::time::Duration;

pub async fn parallel_processing() {
    // 并发执行多个任务
    let handles = (0..10).map(|i| {
        task::spawn(async move {
            // 模拟异步工作
            tokio::time::sleep(Duration::from_millis(100)).await;
            i * 2
        })
    }).collect::<Vec<_>>();
    
    // 等待所有任务完成
    let results = futures::future::join_all(handles).await;
    
    for result in results {
        match result {
            Ok(value) => println!("Result: {}", value),
            Err(e) => eprintln!("Task failed: {}", e),
        }
    }
}

// 使用semaphore限制并发度
use tokio::sync::Semaphore;
use std::sync::Arc;

pub async fn limited_concurrency(tasks: Vec<String>, max_concurrent: usize) {
    let semaphore = Arc::new(Semaphore::new(max_concurrent));
    let mut handles = Vec::new();
    
    for task in tasks {
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        
        let handle = task::spawn(async move {
            let result = process_task(&task).await;
            drop(permit); // 释放许可
            result
        });
        
        handles.push(handle);
    }
    
    futures::future::join_all(handles).await;
}

async fn process_task(task: &str) -> String {
    // 任务处理逻辑
    tokio::time::sleep(Duration::from_millis(100)).await;
    task.to_string()
}
```

### 数据并行

**Rayon并行迭代器**：

```rust
use rayon::prelude::*;

// 并行map
pub fn parallel_transform(data: Vec<i32>) -> Vec<i32> {
    data.par_iter()
        .map(|&x| x * x)
        .collect()
}

// 并行reduce
pub fn parallel_sum(data: &[i32]) -> i32 {
    data.par_iter()
        .sum()
}

// 并行sort
pub fn parallel_sort(data: &mut [i32]) {
    data.par_sort_unstable();
}

// 自定义并行处理
pub fn parallel_process_chunks(data: Vec<u8>, chunk_size: usize) -> Vec<Vec<u8>> {
    data.par_chunks(chunk_size)
        .map(|chunk| {
            // 对每个chunk进行处理
            chunk.iter().map(|&b| b.wrapping_add(1)).collect()
        })
        .collect()
}
```

### 无锁数据结构

**无锁队列**：

```rust
use crossbeam::queue::SegQueue;

pub struct LockFreeQueue<T> {
    queue: SegQueue<T>,
}

impl<T> LockFreeQueue<T> {
    pub fn new() -> Self {
        Self {
            queue: SegQueue::new(),
        }
    }
    
    pub fn push(&self, item: T) {
        self.queue.push(item);
    }
    
    pub fn pop(&self) -> Option<T> {
        self.queue.pop()
    }
}

// 使用原子操作
use std::sync::atomic::{AtomicU64, Ordering};

pub struct AtomicCounter {
    value: AtomicU64,
}

impl AtomicCounter {
    pub fn new() -> Self {
        Self {
            value: AtomicU64::new(0),
        }
    }
    
    pub fn increment(&self) -> u64 {
        self.value.fetch_add(1, Ordering::SeqCst)
    }
    
    pub fn get(&self) -> u64 {
        self.value.load(Ordering::SeqCst)
    }
    
    // Compare-and-swap
    pub fn compare_and_swap(&self, expected: u64, new: u64) -> Result<u64, u64> {
        self.value
            .compare_exchange(expected, new, Ordering::SeqCst, Ordering::SeqCst)
    }
}
```

---

## 缓存策略

### 多级缓存

**L1/L2缓存架构**：

```rust
use moka::sync::Cache;
use std::time::Duration;
use std::sync::Arc;

pub struct MultiLevelCache<K, V>
where
    K: Eq + std::hash::Hash + Clone + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    l1_cache: Cache<K, V>,        // 热数据，小容量，快速访问
    l2_cache: Cache<K, V>,        // 温数据，大容量，较快访问
    backend: Arc<dyn CacheBackend<K, V>>,
}

pub trait CacheBackend<K, V>: Send + Sync {
    fn load(&self, key: &K) -> Option<V>;
}

impl<K, V> MultiLevelCache<K, V>
where
    K: Eq + std::hash::Hash + Clone + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    pub fn new(
        l1_capacity: u64,
        l2_capacity: u64,
        ttl: Duration,
        backend: Arc<dyn CacheBackend<K, V>>,
    ) -> Self {
        let l1_cache = Cache::builder()
            .max_capacity(l1_capacity)
            .time_to_live(ttl)
            .build();
        
        let l2_cache = Cache::builder()
            .max_capacity(l2_capacity)
            .time_to_live(ttl * 2)
            .build();
        
        Self {
            l1_cache,
            l2_cache,
            backend,
        }
    }
    
    pub fn get(&self, key: &K) -> Option<V> {
        // L1 cache
        if let Some(value) = self.l1_cache.get(key) {
            return Some(value);
        }
        
        // L2 cache
        if let Some(value) = self.l2_cache.get(key) {
            // 提升到L1
            self.l1_cache.insert(key.clone(), value.clone());
            return Some(value);
        }
        
        // Backend
        if let Some(value) = self.backend.load(key) {
            self.l2_cache.insert(key.clone(), value.clone());
            self.l1_cache.insert(key.clone(), value.clone());
            return Some(value);
        }
        
        None
    }
    
    pub fn put(&self, key: K, value: V) {
        self.l1_cache.insert(key.clone(), value.clone());
        self.l2_cache.insert(key, value);
    }
    
    pub fn invalidate(&self, key: &K) {
        self.l1_cache.invalidate(key);
        self.l2_cache.invalidate(key);
    }
}
```

### 缓存策略1

**LRU缓存**：

```rust
use lru::LruCache;
use std::num::NonZeroUsize;

pub struct LRUCache<K, V> {
    cache: LruCache<K, V>,
}

impl<K: Eq + std::hash::Hash, V> LRUCache<K, V> {
    pub fn new(capacity: usize) -> Self {
        Self {
            cache: LruCache::new(NonZeroUsize::new(capacity).unwrap()),
        }
    }
    
    pub fn get(&mut self, key: &K) -> Option<&V> {
        self.cache.get(key)
    }
    
    pub fn put(&mut self, key: K, value: V) -> Option<V> {
        self.cache.put(key, value)
    }
}
```

**Write-through vs Write-back**：

```rust
pub trait CacheWriteStrategy<K, V> {
    async fn write(&mut self, key: K, value: V) -> Result<(), anyhow::Error>;
}

// Write-through：同时写缓存和后端
pub struct WriteThroughCache<K, V> {
    cache: Cache<K, V>,
    backend: Arc<dyn CacheBackend<K, V>>,
}

impl<K, V> CacheWriteStrategy<K, V> for WriteThroughCache<K, V>
where
    K: Eq + std::hash::Hash + Clone + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    async fn write(&mut self, key: K, value: V) -> Result<(), anyhow::Error> {
        // 同时写入缓存和后端
        self.cache.insert(key.clone(), value.clone());
        // self.backend.write(key, value).await?;
        Ok(())
    }
}

// Write-back：先写缓存，异步写后端
pub struct WriteBackCache<K, V> {
    cache: Cache<K, V>,
    dirty_keys: Arc<tokio::sync::Mutex<Vec<K>>>,
    backend: Arc<dyn CacheBackend<K, V>>,
}

impl<K, V> WriteBackCache<K, V>
where
    K: Eq + std::hash::Hash + Clone + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    async fn write(&self, key: K, value: V) -> Result<(), anyhow::Error> {
        // 写入缓存
        self.cache.insert(key.clone(), value.clone());
        
        // 标记为脏数据
        self.dirty_keys.lock().await.push(key);
        
        Ok(())
    }
    
    // 后台flush线程
    pub async fn flush_worker(&self) {
        let mut interval = tokio::time::interval(Duration::from_secs(5));
        
        loop {
            interval.tick().await;
            
            let keys = {
                let mut dirty = self.dirty_keys.lock().await;
                std::mem::take(&mut *dirty)
            };
            
            for key in keys {
                if let Some(value) = self.cache.get(&key) {
                    // 写入后端
                    // self.backend.write(key, value).await;
                }
            }
        }
    }
}
```

### 缓存一致性

**缓存失效策略**：

```rust
pub enum InvalidationStrategy {
    TimeToLive(Duration),
    ExplicitInvalidation,
    RefreshAhead,
}

pub struct ConsistentCache<K, V> {
    cache: Cache<K, V>,
    strategy: InvalidationStrategy,
}

impl<K, V> ConsistentCache<K, V>
where
    K: Eq + std::hash::Hash + Clone + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    // Refresh-ahead：主动刷新即将过期的数据
    pub async fn refresh_ahead_worker(&self, backend: Arc<dyn CacheBackend<K, V>>) {
        // 定期检查即将过期的key
        let mut interval = tokio::time::interval(Duration::from_secs(10));
        
        loop {
            interval.tick().await;
            
            // 在实际实现中，需要跟踪key的过期时间
            // 这里简化处理
        }
    }
}
```

---

## 负载均衡

### 负载均衡算法

```rust
use std::sync::atomic::{AtomicUsize, Ordering};

pub trait LoadBalancer {
    fn select_node(&self, nodes: &[String]) -> Option<String>;
}

// 轮询
pub struct RoundRobinBalancer {
    counter: AtomicUsize,
}

impl RoundRobinBalancer {
    pub fn new() -> Self {
        Self {
            counter: AtomicUsize::new(0),
        }
    }
}

impl LoadBalancer for RoundRobinBalancer {
    fn select_node(&self, nodes: &[String]) -> Option<String> {
        if nodes.is_empty() {
            return None;
        }
        
        let index = self.counter.fetch_add(1, Ordering::SeqCst) % nodes.len();
        Some(nodes[index].clone())
    }
}

// 加权轮询
pub struct WeightedRoundRobinBalancer {
    counter: AtomicUsize,
    weights: Vec<usize>,
}

impl WeightedRoundRobinBalancer {
    pub fn new(weights: Vec<usize>) -> Self {
        Self {
            counter: AtomicUsize::new(0),
            weights,
        }
    }
}

impl LoadBalancer for WeightedRoundRobinBalancer {
    fn select_node(&self, nodes: &[String]) -> Option<String> {
        if nodes.is_empty() || self.weights.is_empty() {
            return None;
        }
        
        let total_weight: usize = self.weights.iter().sum();
        let pos = self.counter.fetch_add(1, Ordering::SeqCst) % total_weight;
        
        let mut cumulative = 0;
        for (i, &weight) in self.weights.iter().enumerate() {
            cumulative += weight;
            if pos < cumulative {
                return Some(nodes[i].clone());
            }
        }
        
        None
    }
}

// 最少连接
pub struct LeastConnectionBalancer {
    connections: Arc<tokio::sync::Mutex<Vec<usize>>>,
}

impl LeastConnectionBalancer {
    pub fn new(num_nodes: usize) -> Self {
        Self {
            connections: Arc::new(tokio::sync::Mutex::new(vec![0; num_nodes])),
        }
    }
    
    pub async fn select_node(&self, nodes: &[String]) -> Option<String> {
        let mut conns = self.connections.lock().await;
        
        let min_index = conns
            .iter()
            .enumerate()
            .min_by_key(|(_, &count)| count)
            .map(|(i, _)| i)?;
        
        conns[min_index] += 1;
        Some(nodes[min_index].clone())
    }
    
    pub async fn release_connection(&self, node_index: usize) {
        let mut conns = self.connections.lock().await;
        if node_index < conns.len() && conns[node_index] > 0 {
            conns[node_index] -= 1;
        }
    }
}

// 一致性哈希
use std::collections::BTreeMap;

pub struct ConsistentHashBalancer {
    ring: BTreeMap<u64, String>,
    virtual_nodes: usize,
}

impl ConsistentHashBalancer {
    pub fn new(nodes: Vec<String>, virtual_nodes: usize) -> Self {
        let mut ring = BTreeMap::new();
        
        for node in nodes {
            for i in 0..virtual_nodes {
                let key = format!("{}:{}", node, i);
                let hash = Self::hash(&key);
                ring.insert(hash, node.clone());
            }
        }
        
        Self {
            ring,
            virtual_nodes,
        }
    }
    
    fn hash(key: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        hasher.finish()
    }
    
    pub fn get_node(&self, key: &str) -> Option<String> {
        if self.ring.is_empty() {
            return None;
        }
        
        let hash = Self::hash(key);
        
        // 找到第一个大于等于hash的节点
        self.ring
            .range(hash..)
            .next()
            .or_else(|| self.ring.iter().next())
            .map(|(_, node)| node.clone())
    }
}
```

---

## 性能分析工具

### 自定义性能监控

```rust
use std::time::Instant;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub count: u64,
    pub total_duration: Duration,
    pub min_duration: Duration,
    pub max_duration: Duration,
}

impl PerformanceMetrics {
    fn new() -> Self {
        Self {
            count: 0,
            total_duration: Duration::ZERO,
            min_duration: Duration::MAX,
            max_duration: Duration::ZERO,
        }
    }
    
    fn record(&mut self, duration: Duration) {
        self.count += 1;
        self.total_duration += duration;
        self.min_duration = self.min_duration.min(duration);
        self.max_duration = self.max_duration.max(duration);
    }
    
    pub fn avg_duration(&self) -> Duration {
        if self.count == 0 {
            Duration::ZERO
        } else {
            self.total_duration / self.count as u32
        }
    }
}

pub struct PerformanceMonitor {
    metrics: Arc<RwLock<HashMap<String, PerformanceMetrics>>>,
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    pub async fn record(&self, operation: &str, duration: Duration) {
        let mut metrics = self.metrics.write().await;
        metrics
            .entry(operation.to_string())
            .or_insert_with(PerformanceMetrics::new)
            .record(duration);
    }
    
    pub async fn get_metrics(&self, operation: &str) -> Option<PerformanceMetrics> {
        let metrics = self.metrics.read().await;
        metrics.get(operation).cloned()
    }
    
    pub async fn print_summary(&self) {
        let metrics = self.metrics.read().await;
        
        println!("\n=== Performance Summary ===");
        for (operation, stats) in metrics.iter() {
            println!("\n{}", operation);
            println!("  Count: {}", stats.count);
            println!("  Avg:   {:?}", stats.avg_duration());
            println!("  Min:   {:?}", stats.min_duration);
            println!("  Max:   {:?}", stats.max_duration);
        }
    }
}

// 使用宏简化性能测量
#[macro_export]
macro_rules! measure {
    ($monitor:expr, $operation:expr, $block:expr) => {{
        let start = std::time::Instant::now();
        let result = $block;
        let duration = start.elapsed();
        $monitor.record($operation, duration).await;
        result
    }};
}

// 使用示例
#[tokio::test]
async fn test_performance_monitoring() {
    let monitor = PerformanceMonitor::new();
    
    for _ in 0..10 {
        measure!(monitor, "database_query", {
            tokio::time::sleep(Duration::from_millis(50)).await;
            42
        });
    }
    
    monitor.print_summary().await;
}
```

---

## 最佳实践

### 性能优化清单

```text
□ 算法优化
  □ 选择合适的数据结构
  □ 优化算法复杂度
  □ 避免不必要的计算

□ 内存优化
  □ 减少内存分配
  □ 使用内存池
  □ 优化数据布局
  □ 实现零拷贝

□ 并发优化
  □ 使用异步I/O
  □ 并行处理
  □ 无锁数据结构
  □ 限制并发度

□ 网络优化
  □ 连接复用
  □ 批处理请求
  □ 数据压缩
  □ 使用HTTP/2

□ 存储优化
  □ 顺序写入
  □ 预读取
  □ 使用索引
  □ 数据分片

□ 缓存策略
  □ 多级缓存
  □ 选择合适的淘汰策略
  □ 缓存预热
  □ 监控缓存命中率

□ 监控与调优
  □ 性能基准测试
  □ Profiling分析
  □ 持续监控
  □ A/B测试
```

### 性能目标设定

```text
SLA示例：

1. 延迟目标
   - P50 < 10ms
   - P95 < 50ms
   - P99 < 100ms
   - P99.9 < 500ms

2. 吞吐量目标
   - 读: 100,000 RPS
   - 写: 10,000 RPS

3. 可用性目标
   - 99.99% uptime
   - < 1小时/年停机时间

4. 扩展性目标
   - 支持10倍流量增长
   - 线性水平扩展
```

### 性能回归检测

```rust
use criterion::{Criterion, criterion_group, criterion_main};

fn performance_regression_test(c: &mut Criterion) {
    let mut group = c.benchmark_group("regression");
    
    // 设置baseline
    group.bench_function("current", |b| {
        b.iter(|| {
            // 当前实现
        });
    });
    
    // 对比新实现
    group.bench_function("optimized", |b| {
        b.iter(|| {
            // 优化后的实现
        });
    });
    
    group.finish();
}

criterion_group!(benches, performance_regression_test);
criterion_main!(benches);
```

---

## 相关文档

- [3.10.1 架构模式](architecture_patterns.md)
- [3.10.2 错误处理](error_handling.md)
- [3.10.4 安全设计](security.md)
- [3.8 可观测性](../observability/README.md)
- [3.3 存储系统](../storage/README.md)

## 参考资料

### 工具

- **基准测试**: `criterion`, `bencher`
- **Profiling**: `perf`, `flamegraph`, `tokio-console`
- **并发**: `tokio`, `rayon`, `crossbeam`
- **缓存**: `moka`, `lru`

### 最佳实践1

- The Rust Performance Book
- Linux Performance Tools
- Systems Performance (Brendan Gregg)
