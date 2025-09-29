# 风格规范（Style Guide）

统一注释、Markdown 与代码风格，提升可读性与一致性。

## 注释规范（Rust）

- 模块级注释使用 `//!` 描述目标、性质/不变量、形式化线索与参考。
- 公共 API 使用 `///` 说明用途、参数/返回与边界条件；必要时提供最小示例（doctest）。
- 注释聚焦“为什么”和边界条件，避免赘述显然实现细节。

## 代码风格

- 提交前运行 `cargo fmt` 与 `clippy -D warnings`。
- 命名清晰可读，避免缩写；错误统一使用 `DistributedError`。
- 控制流偏好早返回与浅层嵌套；除测试外避免 `unwrap/expect`。

## 一致性与不变量

- 在核心路径标注不变量（提交单调、前缀匹配、版本单调、幂等约束）。
- 每个测试文件顶部列出“测试目的/关键不变量”，与实现文档互证。

## Markdown 规范

- 列表前后留空行；统一缩进；通过文档 lint。
- 代码块标注语言；长命令/输出分步展示。
- 文档互引：在 `observability`、`performance`、`testing`、`time`、`transactions` 等专题间相互链接。

## 提交前检查

- [ ] 通过 `cargo fmt` 与 `clippy -D warnings`
- [ ] 新增/更新的测试覆盖关键不变量
- [ ] 文档与注释更新且通过 Markdown lint
