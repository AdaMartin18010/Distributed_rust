# Wikipedia 概念映射到代码模块

本表将常见分布式系统 Wikipedia 主题映射到本 crate 的模块、类型与 API，并为每个主题补充“定义/性质/可观测行为/典型陷阱/相关公式或判据/代码映射”，便于从知识点跳转到实现与实验。

---

## Failure detector（故障检测）

- 定义：在不可靠网络下，给出“怀疑/存活”判断的机制（强/弱完备性与准确性刻画）。
- 性质：概率式（如 SWIM）在有限时间内以高概率收敛；误报率与探测周期/超时相关。
- 可观测行为：心跳超时、间接探测成功事件、成员状态从 Alive→Suspect→Dead 的转移。
- 典型陷阱：超时阈值过小导致误报、过大导致收敛慢；广播风暴；视图抖动。
- 相关公式/判据：误报概率与超时 Δ 的函数；流行病传播期望轮数 ≈ O(log N)。
- 代码映射：`swim::{SwimMemberState, SwimEvent, SwimTransport}`，`membership` 视图维护。

## Gossip / Anti-entropy（流言/反熵）

- 定义：随机对等体选择进行状态交换，最终以高概率收敛到一致视图或数据副本。
- 性质：去中心化、健壮性高；收敛时间与 fanout、轮次相关。
- 可观测行为：批次传播、版本向量或摘要比对、谣言终止时间分布。
- 典型陷阱：大对象全量传播开销；热点键重复传播；无背压导致拥塞。
- 相关公式/判据：推式/拉式谣言模型的收敛上界；fanout=√N 的经验选择。
- 代码映射：`swim` 与 `membership` 中的传播与反熵接口（若提供）。

## Consistent hashing（一致性哈希）

- 定义：将节点映射到哈希环，键根据哈希落点选择顺时针的 K 个节点。
- 性质：节点增删时迁移比例期望约为 1/|nodes|（虚拟节点可平衡负载）。
- 可观测行为：键的放置稳定性、扩缩容后的键迁移比例、热点倾斜度。
- 典型陷阱：虚拟节点数过小导致倾斜；哈希函数质量不足；再均衡与副本地理亲和冲突。
- 相关公式/判据：E[迁移比例] ≈ 1/N；副本位置 = next_k(clockwise(hash(key)))。
- 代码映射：`topology::ConsistentHashRing`，`partitioning` 路由策略。

## Quorum / Replication（法定人数 / 复制）

- 定义：通过读/写副本数门限实现一致性语义（如 Majority、R/W quorum）。
- 性质：两个多数派必相交；写入多数派后可视为提交；读多数派可见最新提交。
- 可观测行为：不同 `ConsistencyLevel` 下 ACK 阈值与延迟分布；失败注入后的可见性。
- 典型陷阱：将 N/2 误当多数派；读写策略不匹配导致脏读；超时导致“幽灵失败”。
- 相关公式/判据：`required_acks(N, Strong|Quorum)=floor(N/2)+1`，`Eventual=1`；`R+W>N`。
- 代码映射：`replication::{QuorumPolicy, MajorityQuorum, Replicator}`。

## Consensus（共识：Raft/Paxos/VR）

- 定义：在可能宕机/重试/乱序网络下，多个副本就值序列达成一致的算法族。
- 性质：安全性（不产生冲突决定）、活性（条件下最终决定）；领导者选举与任期。
- 可观测行为：日志前缀匹配、领导者切换、心跳维持；只读指数/租约读。
- 典型陷阱：任期不单调导致脑裂；日志压缩/快照时破坏前缀；重试造成重复应用。
- 相关公式/判据：领导者任期单调递增；提交索引单调；选举超时随机化避免活锁。
- 代码映射：`consensus` 抽象；`consensus_raft` 可选特性（若启用）。

## Consistency models（一致性模型）

- 定义：对读写可见性的语义约束（线性一致、顺序一致、因果一致、最终一致）。
- 性质：线性一致尊重实时先后；顺序一致对每进程程序顺序可见；因果一致保因果前后；最终一致在无更新后收敛。
- 可观测行为：并发读写历史的线化检查；会话读单调性；“读到旧值”的预期与非预期区分。
- 典型陷阱：混淆线性一致与顺序一致；跨分区事务下的读屏障缺失；缓存导致可见性错觉。
- 相关公式/判据：线化存在性；会话保证（RMW/Monotonic Reads/Writes）。
- 代码映射：`consistency::ConsistencyLevel` 与 `replication` 中的 ACK 规则。

## State machine replication（状态机复制）

- 定义：将操作日志按序应用到确定性状态机，以副本冗余实现容错。
- 性质：确定性要求；日志截断/快照；幂等与去重。
- 可观测行为：应用索引单调、快照点前缀一致；恢复后状态一致。
- 典型陷阱：非确定性副作用（时间/随机/IO 顺序）破坏一致；重复应用。
- 相关公式/判据：同序同初始条件 ⇒ 同终态；日志前缀一致性约束。
- 代码映射：`storage::{StateMachineStorage, LogStorage}`。

## Idempotency（幂等）

- 定义：同一操作在副本或重试下多次执行，结果与一次执行等价。
- 性质：需要请求去重键与可重复执行实现；与补偿事务协同。
- 可观测行为：重试下无重复副作用；去重命中率与 TTL。
- 典型陷阱：副作用写外部系统无法回滚；去重键粒度错误导致误判。
- 相关公式/判据：f(f(x))=f(x) 的工程化等价；去重表一致性与过期策略。
- 代码映射：`storage::IdempotencyStore` + `replication::LocalReplicator::replicate_idempotent`。

## Logical clock / Timeouts（逻辑时钟 / 超时）

- 定义：以 Lamport/Vector clock 等描述因果关系；以超时驱动重试/选举等事件。
- 性质：Lamport 保偏序到全序映射；Vector 记录因果并发；超时影响可达性与误报。
- 可观测行为：事件戳单调、并发检测；重试/选举的超时触发与抖动。
- 典型陷阱：将墙钟当因果依据；未考虑时钟漂移；超时固定导致同步抖动。
- 相关公式/判据：`Happens-Before` 关系；抖动随机化避免同步放大。
- 代码映射：`scheduling::{LogicalClock, TimerService}`。

## Saga / Compensation（长事务与补偿）

- 定义：将长事务拆解为可补偿步骤的序列，以补偿操作替代全局锁持有。
- 性质：每步 execute/compensate 均需幂等；失败触发逆序补偿；可与本地事务结合。
- 可观测行为：部分失败后系统不变式保持；重复补偿不产生额外副作用。
- 典型陷阱：补偿不可逆/非幂等；跨域副作用不可回收；超时与重试交互不当。
- 相关公式/判据：补偿闭包性；幂等等价类；隔离级别下的可见性约束。
- 代码映射：`transactions::{Saga, SagaStep}`。

---

## 参考：联动与命令

- 文档联动：`CONCEPT_MODEL.md`、`COURSE_ALIGNMENT.md`、`EXPERIMENT_GUIDE.md`、`ROADMAP.md`
- 快速命令：
  - 测试：`cargo test -p c20_distributed -- --nocapture`
  - 示例：`cargo run -p c20_distributed --example e2e_saga`
  - 基准：`cargo bench -p c20_distributed`
