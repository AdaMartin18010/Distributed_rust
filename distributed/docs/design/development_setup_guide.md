# 开发环境配置指南

**版本**: v1.0  
**日期**: 2025年10月17日  
**目标**: Q1 2025开发团队

---

## 📋 前置要求

### 硬件要求

| 组件 | 最低配置 | 推荐配置 |
|------|---------|---------|
| CPU | 4核心 | 8核心+ |
| 内存 | 8GB | 16GB+ |
| 磁盘 | 50GB SSD | 200GB NVMe SSD |
| 网络 | 100Mbps | 1Gbps |

### 操作系统

支持的操作系统：

- ✅ Linux (Ubuntu 22.04+, Fedora 38+, Arch)
- ✅ macOS (12.0+ Monterey)
- ✅ Windows 11 (WSL2推荐)

---

## 🛠️ 基础工具安装

### 1. Rust工具链

#### 安装Rustup

```bash
# Linux/macOS
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Windows (使用WSL2)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

#### 配置Rust版本

```bash
# 安装最新稳定版
rustup default stable

# 更新到最新版本
rustup update

# 验证安装
rustc --version  # 应该显示 1.90.0 或更高

# 安装必要组件
rustup component add clippy rustfmt rust-analyzer
```

#### 配置国内镜像（可选）

编辑 `~/.cargo/config.toml`:

```toml
[source.crates-io]
replace-with = 'ustc'

[source.ustc]
registry = "sparse+https://mirrors.ustc.edu.cn/crates.io-index/"

[source.tuna]
registry = "https://mirrors.tuna.tsinghua.edu.cn/git/crates.io-index.git"

[net]
git-fetch-with-cli = true
```

### 2. 开发工具

#### Git

```bash
# Ubuntu/Debian
sudo apt-get update
sudo apt-get install git

# macOS
brew install git

# 配置Git
git config --global user.name "Your Name"
git config --global user.email "your.email@example.com"
```

#### 代码编辑器

**推荐: VS Code**:

```bash
# 下载安装: https://code.visualstudio.com/

# 安装必要扩展
code --install-extension rust-lang.rust-analyzer
code --install-extension vadimcn.vscode-lldb
code --install-extension tamasfe.even-better-toml
code --install-extension serayuzgur.crates
```

**或者: IntelliJ IDEA + Rust插件**:

```bash
# 下载安装: https://www.jetbrains.com/idea/
# 安装Rust插件: Settings -> Plugins -> Marketplace -> "Rust"
```

#### 构建工具

```bash
# 安装cargo-watch（自动编译）
cargo install cargo-watch

# 安装cargo-edit（依赖管理）
cargo install cargo-edit

# 安装cargo-expand（宏展开）
cargo install cargo-expand

# 安装cargo-flamegraph（性能分析）
cargo install flamegraph

# 安装cargo-deny（依赖审计）
cargo install cargo-deny
```

### 3. 系统依赖

#### Ubuntu/Debian

```bash
sudo apt-get update
sudo apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    cmake \
    clang \
    lldb \
    protobuf-compiler
```

#### macOS

```bash
# 安装Homebrew
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# 安装依赖
brew install cmake openssl protobuf
```

#### Windows (WSL2)

```powershell
# 启用WSL2
wsl --install

# 在WSL2中安装Ubuntu
# 然后按Ubuntu的步骤安装依赖
```

---

## 📦 项目设置

### 1. 克隆项目

```bash
# 克隆代码库
git clone https://github.com/your-org/distributed-rust.git
cd distributed-rust

# 创建开发分支
git checkout -b feature/your-feature-name
```

### 2. 构建项目

```bash
# 检查项目结构
tree -L 2 distributed/

# 首次完整构建（可能需要10-20分钟）
cd distributed
cargo build

# 快速检查（不生成二进制文件）
cargo check

# 发布构建（优化，用于测试性能）
cargo build --release
```

### 3. 运行测试

```bash
# 运行所有测试
cargo test

# 运行特定模块测试
cargo test --package distributed --lib consensus

# 运行单个测试
cargo test test_raft_leader_election -- --exact

# 显示测试输出
cargo test -- --nocapture

# 并行运行测试（提高速度）
cargo test -- --test-threads=4
```

### 4. 代码质量检查

```bash
# 运行Clippy（Rust linter）
cargo clippy --all-targets --all-features -- -D warnings

# 格式化代码
cargo fmt --all

# 检查格式是否正确（CI用）
cargo fmt --all -- --check

# 检查依赖
cargo deny check

# 安全审计
cargo audit
```

---

## 🧪 测试环境配置

### 1. 单元测试配置

创建 `distributed/tests/.env`:

```bash
# 日志级别
RUST_LOG=distributed=debug,raft=trace

# 测试超时
TEST_TIMEOUT=30

# 临时目录
TEST_TMP_DIR=/tmp/distributed-test
```

### 2. 集成测试配置

```bash
# 创建测试集群配置
mkdir -p distributed/tests/fixtures

cat > distributed/tests/fixtures/test_cluster.toml <<EOF
[cluster]
size = 3

[node.0]
id = "node0"
addr = "127.0.0.1:5000"

[node.1]
id = "node1"
addr = "127.0.0.1:5001"

[node.2]
id = "node2"
addr = "127.0.0.1:5002"
EOF
```

### 3. 性能测试配置

```bash
# 安装基准测试工具
cargo install cargo-criterion

# 运行基准测试
cargo bench

# 查看报告
open distributed/target/criterion/report/index.html
```

---

## 🐛 调试配置

### VS Code调试配置

创建 `.vscode/launch.json`:

```json
{
    "version": "0.2.0",
    "configurations": [
        {
            "type": "lldb",
            "request": "launch",
            "name": "Debug distributed",
            "cargo": {
                "args": [
                    "build",
                    "--bin=distributed-node",
                    "--package=distributed"
                ],
                "filter": {
                    "name": "distributed-node",
                    "kind": "bin"
                }
            },
            "args": [],
            "cwd": "${workspaceFolder}",
            "env": {
                "RUST_LOG": "distributed=debug",
                "RUST_BACKTRACE": "1"
            }
        },
        {
            "type": "lldb",
            "request": "launch",
            "name": "Debug tests",
            "cargo": {
                "args": [
                    "test",
                    "--no-run",
                    "--package=distributed",
                    "--lib"
                ],
                "filter": {
                    "name": "distributed",
                    "kind": "lib"
                }
            },
            "args": ["--nocapture"],
            "cwd": "${workspaceFolder}"
        }
    ]
}
```

### 日志配置

创建 `distributed/.cargo/config.toml`:

```toml
[build]
rustflags = ["-C", "link-arg=-fuse-ld=lld"]  # 使用lld加速链接

[target.x86_64-unknown-linux-gnu]
linker = "clang"

[env]
RUST_LOG = "distributed=debug,raft=trace"
RUST_BACKTRACE = "1"
```

### GDB/LLDB使用

```bash
# 使用LLDB调试
rust-lldb distributed/target/debug/distributed-node

# 常用命令
(lldb) b main.rs:100        # 设置断点
(lldb) r                    # 运行
(lldb) n                    # 单步执行
(lldb) s                    # 进入函数
(lldb) bt                   # 查看调用栈
(lldb) p variable_name      # 打印变量
(lldb) c                    # 继续运行
```

---

## 📊 性能分析工具

### 1. Flamegraph（CPU火焰图）

```bash
# 生成火焰图
cargo flamegraph --bin distributed-node

# 在浏览器中查看
firefox flamegraph.svg
```

### 2. Valgrind（内存分析）

```bash
# 检查内存泄漏
cargo build
valgrind --leak-check=full --show-leak-kinds=all \
    ./target/debug/distributed-node

# 性能分析
valgrind --tool=callgrind ./target/debug/distributed-node
kcachegrind callgrind.out.*
```

### 3. Perf（Linux性能分析）

```bash
# 记录性能数据
cargo build --release
perf record -g ./target/release/distributed-node

# 分析报告
perf report
```

---

## 🔧 开发工作流

### 日常开发流程

```bash
# 1. 更新代码
git pull origin main

# 2. 创建特性分支
git checkout -b feature/read-index

# 3. 开发
# 自动重新编译和运行测试
cargo watch -x check -x test -x run

# 4. 代码质量检查
cargo fmt --all
cargo clippy --fix --allow-dirty --allow-staged

# 5. 运行完整测试
cargo test --all-features

# 6. 提交代码
git add -A
git commit -m "feat: implement read index"

# 7. 推送到远程
git push origin feature/read-index

# 8. 创建Pull Request
# 在GitHub/GitLab上创建PR
```

### 代码审查检查清单

提交PR前的自检：

- [ ] 代码通过`cargo fmt`格式化
- [ ] 没有`cargo clippy`警告
- [ ] 所有测试通过（`cargo test`）
- [ ] 添加了必要的单元测试
- [ ] 更新了相关文档
- [ ] 提交信息清晰明了
- [ ] 代码有适当的注释
- [ ] 没有TODO或FIXME标记

---

## 🌍 CI/CD本地验证

### GitHub Actions本地运行

```bash
# 安装act
brew install act  # macOS
# 或
curl https://raw.githubusercontent.com/nektos/act/master/install.sh | sudo bash

# 运行CI流程
act -j test
act -j lint
```

### Docker开发环境

```dockerfile
# Dockerfile.dev
FROM rust:1.90

WORKDIR /workspace

# 安装依赖
RUN apt-get update && apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    cmake

# 复制项目
COPY . .

# 构建项目
RUN cargo build

CMD ["/bin/bash"]
```

```bash
# 构建开发镜像
docker build -f Dockerfile.dev -t distributed-rust-dev .

# 运行容器
docker run -it --rm \
    -v $(pwd):/workspace \
    -w /workspace \
    distributed-rust-dev
```

---

## 📚 学习资源

### 必读文档

1. **项目文档**
   - [README.md](../../README.md)
   - [CONTRIBUTING.md](../../CONTRIBUTING.md)
   - [技术设计文档](./README.md)

2. **Rust学习**
   - [The Rust Book](https://doc.rust-lang.org/book/)
   - [Rust by Example](https://doc.rust-lang.org/rust-by-example/)
   - [Async Book](https://rust-lang.github.io/async-book/)

3. **分布式系统**
   - [MIT 6.824](https://pdos.csail.mit.edu/6.824/)
   - [Raft论文](https://raft.github.io/raft.pdf)
   - [Designing Data-Intensive Applications](https://dataintensive.net/)

### 开发技巧

#### 快速命令别名

添加到 `~/.bashrc` 或 `~/.zshrc`:

```bash
# Rust开发别名
alias cb='cargo build'
alias cr='cargo run'
alias ct='cargo test'
alias cc='cargo check'
alias cw='cargo watch'
alias cf='cargo fmt --all'
alias cl='cargo clippy --all-targets --all-features'

# 快速测试
alias ctn='cargo test -- --nocapture'
alias ctf='cargo test --features'

# 性能测试
alias cbench='cargo bench'
alias crelease='cargo build --release'
```

#### Vim/Neovim配置（可选）

对于Vim用户，安装rust.vim和coc.nvim:

```vim
" ~/.vimrc 或 ~/.config/nvim/init.vim
Plug 'rust-lang/rust.vim'
Plug 'neoclide/coc.nvim', {'branch': 'release'}

" Rust配置
let g:rustfmt_autosave = 1
let g:rust_clip_command = 'pbcopy'  " macOS
```

---

## ❓ 常见问题

### Q: 编译很慢怎么办？

A: 可以尝试以下优化：

```bash
# 1. 使用更快的链接器
cargo install -f cargo-binutils
rustup component add llvm-tools-preview

# 2. 启用增量编译（debug模式默认启用）
export CARGO_INCREMENTAL=1

# 3. 使用sccache缓存
cargo install sccache
export RUSTC_WRAPPER=sccache

# 4. 增加并行编译jobs
export CARGO_BUILD_JOBS=8
```

### Q: 测试失败怎么办？

A: 调试步骤：

```bash
# 1. 运行单个测试查看详细输出
cargo test test_name -- --nocapture --exact

# 2. 启用日志
RUST_LOG=debug cargo test test_name -- --nocapture

# 3. 使用调试器
rust-lldb target/debug/deps/distributed-* test_name
```

### Q: RocksDB编译失败？

A: 确保安装了必要的系统依赖：

```bash
# Ubuntu/Debian
sudo apt-get install libclang-dev

# macOS
brew install llvm
export LIBCLANG_PATH=/usr/local/opt/llvm/lib
```

---

## ✅ 验收标准

开发环境配置完成后，运行以下命令验证：

```bash
#!/bin/bash
echo "验证开发环境..."

# 检查Rust版本
echo "检查Rust版本..."
rustc --version | grep "1.90" || echo "❌ Rust版本不正确"

# 检查工具
echo "检查开发工具..."
command -v cargo-watch || echo "❌ cargo-watch未安装"
command -v cargo-clippy || echo "❌ clippy未安装"
command -v cargo-fmt || echo "❌ rustfmt未安装"

# 构建项目
echo "构建项目..."
cargo build || echo "❌ 构建失败"

# 运行测试
echo "运行测试..."
cargo test --lib --bins || echo "❌ 测试失败"

# 代码质量检查
echo "代码质量检查..."
cargo clippy -- -D warnings || echo "❌ Clippy检查失败"
cargo fmt --all -- --check || echo "❌ 格式检查失败"

echo "✅ 开发环境配置完成！"
```

---

## 📞 获取帮助

遇到问题？

1. **查阅文档**: 先查看[FAQ](../FAQ.md)
2. **搜索Issues**: GitHub Issues中搜索类似问题
3. **提问**: 在GitHub Discussions中提问
4. **联系导师**: 联系项目导师或团队负责人

---

**文档维护者**: DevOps Team  
**最后更新**: 2025年10月17日  
**下次更新**: 每月1日
