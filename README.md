# Distributed Rust 🦀

[![Rust](https://img.shields.io/badge/rust-1.90+-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Build Status](https://github.com/your-org/distributed-rust/workflows/CI/badge.svg)](https://github.com/your-org/distributed-rust/actions)
[![Coverage](https://codecov.io/gh/your-org/distributed-rust/branch/main/graph/badge.svg)](https://codecov.io/gh/your-org/distributed-rust)

> 🚀 **现代化的分布式系统教学与实践项目** - 通过 Rust 语言展示分布式系统的核心概念、算法实现和工程实践

## 📋 目录

- [Distributed Rust 🦀](#distributed-rust-)
  - [📋 目录](#-目录)
  - [🎯 项目介绍](#-项目介绍)
    - [✨ 项目特色](#-项目特色)
    - [🎯 适用场景](#-适用场景)
  - [🚀 快速开始](#-快速开始)
    - [环境要求](#环境要求)
    - [安装步骤](#安装步骤)
    - [验证安装](#验证安装)
  - [🏗️ 核心功能](#️-核心功能)
    - [1. 分布式系统核心组件](#1-分布式系统核心组件)
    - [2. 工程实践](#2-工程实践)
    - [3. 性能优化](#3-性能优化)
  - [📁 项目结构](#-项目结构)
  - [🎓 学习路径](#-学习路径)
    - [初学者路径 (1-2 周)](#初学者路径-1-2-周)
    - [进阶路径 (3-4 周)](#进阶路径-3-4-周)
    - [专家路径 (1-2 月)](#专家路径-1-2-月)
  - [🎮 运行示例](#-运行示例)
    - [基础示例](#基础示例)
    - [端到端示例](#端到端示例)
    - [期望输出示例](#期望输出示例)
  - [🧪 测试验证](#-测试验证)
    - [运行测试](#运行测试)
    - [代码质量检查](#代码质量检查)
    - [性能基准](#性能基准)
  - [📚 文档资源](#-文档资源)
    - [核心文档](#核心文档)
    - [理论文档](#理论文档)
    - [实践文档](#实践文档)
    - [实验指南](#实验指南)
    - [学习资源](#学习资源)
      - [推荐课程](#推荐课程)
      - [相关书籍](#相关书籍)
      - [重要论文](#重要论文)
  - [🤝 贡献指南](#-贡献指南)
    - [如何贡献](#如何贡献)
    - [代码规范](#代码规范)
    - [贡献类型](#贡献类型)
    - [问题报告](#问题报告)
  - [📄 许可证](#-许可证)
  - [🆘 获取帮助](#-获取帮助)

## 🎯 项目介绍

Distributed Rust 是一个现代化的分布式系统教学与实践项目，旨在通过 Rust 语言展示分布式系统的核心概念、算法实现和工程实践。

### ✨ 项目特色

- **🎓 教学导向**: 完整的分布式系统概念覆盖，适合学习和研究
- **🦀 Rust 原生**: 利用 Rust 的安全性和性能优势构建分布式系统
- **🔧 生产就绪**: 提供可部署的解决方案和最佳实践
- **📊 可观测性**: 内置监控、日志和追踪能力
- **🧪 测试完备**: 单元测试、集成测试、混沌测试全覆盖

### 🎯 适用场景

- **分布式系统学习**: 通过可运行代码理解分布式算法
- **原型开发**: 快速搭建分布式系统原型
- **生产部署**: 提供可部署的解决方案和最佳实践
- **性能测试**: 基准测试和性能分析
- **故障模拟**: 混沌工程和故障注入测试

## 🚀 快速开始

### 环境要求

- Rust 1.90+
- Windows 10/11, Linux (x86_64/aarch64), macOS (Apple/Intel)

### 安装步骤

```bash
# 1. 安装 Rust (如果未安装)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 2. 克隆项目
git clone https://github.com/your-org/distributed-rust.git
cd distributed-rust

# 3. 构建项目
cargo build --workspace

# 4. 运行基础示例
cargo run -p distributed --example network_demo
```

### 验证安装

```bash
# 检查版本
rustc --version
cargo --version

# 运行测试
cargo test --workspace
```

## 🏗️ 核心功能

### 1. 分布式系统核心组件

| 组件 | 功能 | 实现 |
|------|------|------|
| **共识算法** | 分布式一致性 | Raft, Paxos, 拜占庭容错 |
| **一致性模型** | 数据一致性保证 | 线性一致, 顺序一致, 因果一致 |
| **复制策略** | 数据冗余和可用性 | 主从复制, 链式复制, Quorum 读写 |
| **成员管理** | 集群节点管理 | SWIM 故障检测, Gossip 协议 |
| **负载均衡** | 请求分发 | 一致性哈希, 加权轮询, 最少连接 |

### 2. 工程实践

| 实践 | 功能 | 工具 |
|------|------|------|
| **可观测性** | 系统监控和调试 | 分布式追踪, 指标收集, 结构化日志 |
| **安全治理** | 系统安全和稳定 | 访问控制, 限流熔断, 审计日志 |
| **故障注入** | 系统韧性测试 | 混沌工程, 网络分区, 延迟注入 |
| **配置管理** | 动态配置 | 热更新, 多环境配置, 版本控制 |

### 3. 性能优化

| 优化 | 技术 | 效果 |
|------|------|------|
| **异步编程** | Tokio 运行时 | 高并发, 低延迟 |
| **内存管理** | 零拷贝序列化 | 减少内存分配和复制 |
| **网络优化** | 连接池, 批量请求 | 提升网络吞吐量 |
| **基准测试** | Criterion 驱动 | 性能回归检测 |

## 📁 项目结构

```text
distributed-rust/
├── distributed/                    # 核心分布式系统库
│   ├── src/
│   │   ├── consensus/             # 共识算法 (Raft, Paxos)
│   │   ├── consistency/           # 一致性模型
│   │   ├── network/               # 网络通信 (RPC, 连接池)
│   │   ├── storage/               # 存储抽象 (日志存储, 复制)
│   │   ├── monitoring/            # 监控系统 (指标收集, 健康检查)
│   │   └── security/              # 安全模块 (访问控制, 限流熔断)
│   ├── examples/                  # 示例代码
│   ├── tests/                     # 测试代码
│   ├── benches/                   # 基准测试
│   └── docs/                      # 文档
├── solutions/                     # 生产级解决方案
│   ├── foundations-datafusion/    # DataFusion + Foundations 集成
│   ├── vector-topology/           # Vector 分布式可观测性
│   └── end-to-end-stack/          # 端到端参考架构
└── README.md                      # 项目说明
```

## 🎓 学习路径

### 初学者路径 (1-2 周)

1. **基础概念**: 阅读 `distributed/docs/` 目录下的概念文档
2. **简单示例**: 运行 `examples/` 中的基础示例
3. **动手实践**: 修改示例代码，观察行为变化

**推荐文档**:

- [快速开始指南](distributed/docs/QUICKSTART.md)
- [安装指南](distributed/docs/INSTALL.md)
- [示例代码中心](distributed/docs/examples/README.md)

### 进阶路径 (3-4 周)

1. **算法实现**: 深入研究 `consensus/` 和 `consistency/` 模块
2. **性能调优**: 运行基准测试，分析性能瓶颈
3. **端到端实践**: 使用 `solutions/` 中的完整解决方案

**推荐文档**:

- [一致性模型详解](distributed/docs/consistency/README.md)
- [共识算法实现](distributed/docs/consensus/README.md)
- [系统设计最佳实践](distributed/docs/design/BEST_PRACTICES.md)

### 专家路径 (1-2 月)

1. **源码贡献**: 阅读核心代码，提交改进建议
2. **新特性开发**: 实现新的分布式算法或优化
3. **生产部署**: 使用部署策略在生产环境中验证

**推荐文档**:

- [关键论证与公式](distributed/docs/FORMAL_ARGUMENTS.md)
- [性能优化技巧](distributed/docs/performance/OPTIMIZATION.md)
- [测试策略](distributed/docs/testing/README.md)

## 🎮 运行示例

### 基础示例

```bash
# 网络通信演示
cargo run -p distributed --example network_demo

# 一致性模型演示
cargo run -p distributed --example consistency_demo

# Raft 共识算法演示
cargo run -p distributed --example raft_demo
```

### 端到端示例

```bash
# 分布式复制示例
cargo run -p distributed --example e2e_replication

# Saga 分布式事务示例
cargo run -p distributed --example e2e_saga

# 负载均衡示例
cargo run -p distributed --example e2e_load_balancer_min

# 混沌工程示例
cargo run -p distributed --example e2e_chaos_min
```

### 期望输出示例

**e2e_replication 正常输出**:

```text
[INFO] Starting replication demo with 3 nodes
[INFO] Node 1: Listening on 127.0.0.1:8001
[INFO] Node 2: Listening on 127.0.0.1:8002
[INFO] Node 3: Listening on 127.0.0.1:8003
[INFO] Replication test: Writing key=test, value=hello
[INFO] Replication test: Reading from all nodes - SUCCESS
```

**e2e_saga 正常输出**:

```text
[INFO] Starting Saga transaction demo
[INFO] Step 1: Reserve inventory - SUCCESS
[INFO] Step 2: Process payment - SUCCESS
[INFO] Step 3: Update order status - SUCCESS
[INFO] Saga transaction completed successfully
```

## 🧪 测试验证

### 运行测试

```bash
# 运行所有测试
cargo test --workspace

# 运行特定模块测试
cargo test -p distributed

# 运行基准测试
cargo bench -p distributed

# 运行混沌测试
cargo test --features chaos --test chaos_tests
```

### 代码质量检查

```bash
# 代码格式检查
cargo fmt --all --check

# 静态分析
cargo clippy --all-targets -- -D warnings

# 安全审计
cargo audit
```

### 性能基准

```bash
# 运行网络性能基准
cargo bench -p distributed network_performance

# 运行锁性能基准
cargo bench -p distributed lock_performance

# 运行共识性能基准
cargo bench -p distributed consensus_performance
```

## 📚 文档资源

### 核心文档

- **[快速开始指南](distributed/docs/QUICKSTART.md)** - 5分钟快速上手
- **[安装指南](distributed/docs/INSTALL.md)** - 详细的安装步骤
- **[示例代码中心](distributed/docs/examples/README.md)** - 丰富的使用示例

### 理论文档

- **[一致性模型详解](distributed/docs/consistency/README.md)** - 强一致性、最终一致性等
- **[共识算法实现](distributed/docs/consensus/README.md)** - Raft、Paxos 等算法
- **[关键论证与公式](distributed/docs/FORMAL_ARGUMENTS.md)** - 数学证明和形式化论证

### 实践文档

- **[系统设计最佳实践](distributed/docs/design/BEST_PRACTICES.md)** - 架构设计原则
- **[性能优化技巧](distributed/docs/performance/OPTIMIZATION.md)** - 性能调优指南
- **[测试策略](distributed/docs/testing/README.md)** - 单元测试、集成测试、混沌测试

### 实验指南

- **[实验检查清单](distributed/docs/experiments/CHECKLIST.md)** - 系统性验证实验
- **[常见陷阱与调试](distributed/docs/PITFALLS.md)** - 开发中的常见问题和解决方案

### 学习资源

#### 推荐课程

- [MIT 6.824 分布式系统](https://pdos.csail.mit.edu/6.824/)
- [Stanford CS244B 分布式系统](https://web.stanford.edu/class/cs244b/)
- [CMU 15-440 分布式系统](https://www.cs.cmu.edu/~dga/15-440/S14/)

#### 相关书籍

- 《分布式系统概念与设计》
- 《数据密集型应用系统设计》
- 《Rust 程序设计语言》

#### 重要论文

- [Raft: In Search of an Understandable Consensus Algorithm](https://raft.github.io/raft.pdf)
- [The Part-Time Parliament](https://lamport.azurewebsites.net/pubs/lamport-paxos.pdf)
- [Dynamo: Amazon's Highly Available Key-value Store](https://www.allthingsdistributed.com/files/amazon-dynamo-sosp2007.pdf)

## 🤝 贡献指南

### 如何贡献

1. **Fork 项目**: 在 GitHub 上 Fork 本项目
2. **创建分支**: 创建新的功能分支 `git checkout -b feature/your-feature`
3. **提交代码**: 提交前运行质量检查
4. **创建 PR**: 提交 Pull Request

### 代码规范

```bash
# 提交前本地校验
cargo fmt --all && cargo clippy --all-targets -- -D warnings && cargo test --workspace
```

### 贡献类型

- **Bug 修复**: 修复现有问题
- **新功能**: 实现新的分布式算法或组件
- **文档改进**: 完善文档和示例
- **性能优化**: 提升系统性能
- **测试增强**: 增加测试覆盖

### 问题报告

- **Bug 报告**: 使用 GitHub Issues 报告问题
- **功能请求**: 提出新功能建议
- **讨论交流**: 参与项目讨论

## 📄 许可证

本项目遵循 `MIT` 许可证，详见根目录 `LICENSE` 文件。

## 🆘 获取帮助

- **GitHub Issues**: [报告问题](https://github.com/your-org/c20_distributed/issues)
- **Discussions**: [讨论交流](https://github.com/your-org/c20_distributed/discussions)
- **Stack Overflow**: [技术问答](https://stackoverflow.com/questions/tagged/c20-distributed)

---

**开始您的分布式系统之旅！** 🚀

选择适合您的学习路径，深入理解分布式系统的核心概念，掌握实践技能，构建可靠、高性能的分布式应用。
