# Tasks: 通用 EDID 序列化能力

## Phase 1: 严格序列化基础

- [x] Task 1: 固化 Base Block roundtrip 契约
  - Acceptance: 合法 EDID 原样 roundtrip；只改目标字段时未触及 bytes 保持；失败无部分输出。
  - Verify: `cargo test --all-targets`
  - Dependencies: None

- [x] Task 2: 增加统一 checked EDID 输出边界
  - Acceptance: `validate_for_serialization` 与 `to_bytes_checked`；校验 header/version/count/checksum；修复 extension count 溢出。
  - Verify: `cargo test --all-targets`; `cargo clippy --all-targets -- -D warnings`
  - Dependencies: Task 1

### Checkpoint: 严格基础能力

- [x] Base Block parse-edit-serialize 契约通过
- [x] checked 输出不会产生 count/checksum 不一致


## Phase 2: Base Block 生成与策略

- [x] Task 3: 建立 Base Block builder 与序列化策略
  - Acceptance: 从零构造合法 Base Block；metadata/chromaticity/established/standard/descriptor/detailed timings 支持；非法字段结构化失败；单一 placement policy 明确为先复用 timing、再使用 free slot、永不覆盖 descriptor。
  - Scope boundary: flags/canonicalization 多策略尚未实现，不在本 checkpoint 声称支持。
  - Verify: builder tests; `cargo test --all-targets`; `cargo clippy --all-targets -- -D warnings`; `cargo doc --no-deps`
  - Dependencies: Task 2
### Checkpoint: Base Block builder
- [x] builder 能力与结构化错误边界通过测试。
- [x] 单一 timing placement/preservation policy 已公开并文档化。
- [ ] flags/canonicalization 多策略（后续任务）。

## Phase 3: CTA-861 写入

- [x] Task 4: 定义 CTA raw/typed 混合写入模型
  - Acceptance: raw CTA blocks 可编码；unknown blocks 原样保留；非法 payload 结构化失败；typed view encoder 已覆盖可表示的已知 CTA views。
  - Verify: CTA golden bytes and roundtrip tests
  - Dependencies: Task 2

- [x] Task 5: 实现 CTA 自动布局与 DTD 写入
  - Acceptance: 自动维护 DTD offset/native count/checksum；Data Block、DTD 不重叠；空间不足不截断。
  - Verify: CTA boundary tests; `cargo test --all-targets`
  - Dependencies: Task 4

- [x] Task 6: 提供 CTA raw mutation 与完整 roundtrip API
  - Acceptance: CTA raw blocks/DTD/header 可构造、替换；unknown raw blocks 和其他 extensions 保持；输出可重新解析。
  - Verify: CTA integration tests
  - Dependencies: Task 5

### Checkpoint: CTA 写入

- [x] CTA typed blocks 可编码。
- [x] CTA 自动布局通过所有边界测试。
- [x] CTA 构造与替换可完成 parse-edit-serialize。

## Phase 4: DisplayID 写入

- [x] Task 7: 定义 DisplayID typed/raw block 编码器
  - Acceptance: raw 与 Product/Parameters/Type I/Type VII timing/embedded CTA typed data blocks 可编码；unknown blocks 保留；不可表示字段结构化失败。
  - Verify: DisplayID golden bytes and roundtrip tests
  - Dependencies: Task 2, Task 4

- [x] Task 8: 实现 DisplayID raw payload 布局与 mutation
  - Acceptance: payload <= 121；header length 正确；block 可增删改；失败不产生截断输出。
  - Verify: 121-byte boundary and integration tests
  - Dependencies: Task 7

## Phase 5: 统一生命周期与发布质量

- [x] Task 9: 统一扩展生命周期 API
  - Acceptance: extensions 可插入、替换、删除；count/checksum 由生命周期 API 维护；checked output 验证完整对象。
  - Verify: multi-extension integration tests; test/clippy/doc
  - Dependencies: Task 6, Task 8

- [ ] Task 10: fuzz、真实 corpus、文档和发布验证
  - Acceptance: fuzz、README、CHANGELOG、CI 已补齐；真实 EDID corpus 仍待补齐，cargo-fuzz 本地工具未安装。
  - Verify: fmt, clippy, test, doc, package, MSRV, cross-platform checks
  - Dependencies: Task 1, Task 2, Task 6, Task 8, Task 9

### Checkpoint: Complete

- [ ] Base、CTA、DisplayID 均具备 typed write path
- [ ] 完整 EDID 支持 parse-edit-validate-serialize
- [ ] unknown/reserved 保留契约有测试证明
- [ ] 所有质量门禁和 MSRV 检查通过
- [ ] 发布说明和 API 文档完成
