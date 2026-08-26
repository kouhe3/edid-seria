# Implementation Plan: 通用 EDID 序列化能力

## Overview

将 `edid-seria` 从“Base Block DTD 序列化、CTA/DisplayID 只读保留”扩展为具备稳定编辑生命周期的通用 EDID 序列化库。目标是支持 `parse -> typed edit -> validate -> serialize`，优先建立严格事务边界与 Base Block 回归契约，再实现 CTA-861 写入，最后实现 DisplayID 写入与高层 builder。

本计划不包含操作系统显示器枚举、注册表访问、权限申请或驱动 override 应用；这些属于平台层，不进入零依赖核心库。

## Architecture Decisions

- **严格序列化优先：** 新增统一 checked 验证/输出入口；已有兼容 API 保留，但不作为新功能的基础接口。
- **编辑与最终编码分离：** setter 可以修改内存模型；最终 `to_bytes_checked` 负责完整验证、布局、extension count 和 checksum。
- **原始保留优先：** 未修改字段、reserved bits、unknown payload、未知块顺序默认保持；canonicalization 作为显式策略，不作为默认行为。
- **扩展按块建模：** CTA 与 DisplayID 使用 typed known block + raw unknown block 的混合模型，避免已知类型编码时丢失未知数据。
- **布局器负责边界：** CTA Data Block/DTD 区域和 DisplayID payload 长度不得由调用者手工维护。
- **失败不产生半成品：** checked serialize 在验证或布局失败时返回结构化错误，不返回部分字节；必要时先在临时副本上完成编码。
- **公共 API 兼容：** 继续支持现有 `EdidBlock` 与 `serialize_*` API；新能力通过 additive API 提供，避免无必要的破坏性修改。

## Dependency Graph

```text
Task 1: roundtrip contract and error inventory
    |
    +--> Task 2: Edid checked serialization boundary
    |       |
    |       +--> Task 3: Base Block builder and policy model
    |
    +--> Task 4: CTA block model and encoders
            |
            +--> Task 5: CTA layout and mutation API
                    |
                    +--> Task 6: CTA roundtrip integration

Task 2 + Task 4 --> Task 7: DisplayID block encoders
Task 7 --> Task 8: DisplayID layout and mutation API
Task 5 + Task 8 --> Task 9: unified extension lifecycle
Task 6 + Task 8 --> Task 10: fuzz/corpus/docs/release verification
```

## Task List

### Phase 1: 严格序列化基础

#### Task 1: 固化 Base Block roundtrip 契约

**Description:** 明确并测试解析后原样输出、只修改目标字段、保留未触及字节、reserved bits 与 unknown descriptor 的行为；纠正当前“corpus roundtrip”测试未比较输出字节的问题。

**Acceptance criteria:**
- [x] 合法 EDID 的 `from_bytes -> to_bytes` 有明确的字节级测试。
- [x] 修改 Base DTD 后，未触及的 descriptors、metadata、reserved bits 与 extension bytes 保持。
- [x] 无效输入、无效 timing、槽位不足时没有可观察的部分序列化结果。

**Verification:** `cargo test --all-targets`；新增测试覆盖 exact roundtrip 与 edit isolation。

**Dependencies:** None

**Files likely touched:** `tests/core_regression.rs`, `src/edid.rs`, `src/serialize.rs`

**Estimated scope:** Medium

#### Task 2: 增加统一 checked EDID 输出边界

**Description:** 为完整 `Edid` 聚合对象增加验证与 checked serialization 入口，统一校验 Base Block、extension count、每块 checksum、扩展边界，并明确 `to_bytes` 的兼容语义。

**Acceptance criteria:**
- [x] 提供 `Edid::validate_for_serialization` 或等价公开 API。
- [x] 提供 `Edid::to_bytes_checked` 或等价 checked API，失败返回结构化错误。
- [x] checked API 验证 extension count、checksum、Base Block version/header 和块数量。
- [x] 修复兼容 serializer 的 extension count 溢出路径，不产生截断后的非法输出。

**Verification:** `cargo test --all-targets`、`cargo clippy --all-targets -- -D warnings`。

**Dependencies:** Task 1

**Files likely touched:** `src/edid.rs`, `src/error.rs`, `src/serialize.rs`, `src/lib.rs`

**Estimated scope:** Medium

#### Checkpoint: 严格基础能力

- [x] Base Block parse-edit-serialize 契约通过。
- [x] checked 输出不会产生 count/checksum 不一致的结果。
- [x] 108 个测试、clippy、rustdoc 通过。

### Phase 2: Base Block 生成与策略

#### Task 3: 建立 Base Block builder 与序列化策略

**Description:** 在不破坏现有 setter 的前提下，提供从 typed metadata、timing 和 descriptors 构造合法 Base Block 的高层 builder；当前实现公开一个明确的 timing placement/preservation policy。flags 与 canonicalization 多策略不在本任务范围，不能宣称已实现。

**Acceptance criteria:**
- [x] builder 能从零构造带合法 header、metadata、descriptor、DTD 和 checksum 的 Base Block。
- [x] 非法 manufacturer、gamma、尺寸、descriptor 槽位和 DTD 字段返回结构化错误。
- [x] timing placement、旧 DTD 保留策略通过公开的单一 `TimingPlacement` policy 明确规定：先复用已有 timing，再使用 free slot，永不覆盖 descriptor；flags/canonicalization 多策略留待后续任务。

**Verification:** builder 单元测试、`cargo test --all-targets`、`cargo doc --no-deps`。

**Dependencies:** Task 2

**Files likely touched:** `src/builder.rs`, `src/lib.rs`, `tasks/plan.md`, `tasks/todo.md`

**Estimated scope:** Medium

#### Checkpoint: Base Block builder

- [x] builder metadata/chromaticity/established/standard/descriptor/detailed timing 能力通过测试。
- [x] 单一 placement policy、descriptor preservation 与结构化错误边界明确。
- [ ] flags/canonicalization 多策略（未在 Task 3 实现）。


### Phase 3: CTA-861 写入

#### Task 4: 定义 CTA typed/raw 混合写入模型

**Description:** 为现有 CTA read views 建立可编码的数据模型，覆盖 Header、Video、Audio、Speaker、HDR、Colorimetry、Video Capability、VSDB 和 unknown blocks；已知模型必须能保留必要 raw/reserved 字段。

- [x] 原始 CTA Data Block、集合和 progressive DTD 均有编码路径。
- [x] 未知 CTA payload 可通过 raw block 保留。
- [x] 短 payload、非法 tag、非法长度和不可表示字段返回结构化错误。
- [x] 所有当前 typed CTA view 均有专用 encoder。

**Verification:** CTA golden byte tests、encode/decode roundtrip tests、clippy。

**Dependencies:** Task 2

**Files likely touched:** `src/extensions.rs`, `src/error.rs`, `src/lib.rs`, `tests/core_regression.rs`

**Estimated scope:** Large；按 CTA block 家族继续拆分实现。

#### Task 5: 实现 CTA 自动布局与 DTD 写入

**Description:** 实现 CTA Data Block Collection、CTA DTD、padding、Header 和 checksum 的统一布局器，自动维护 DTD offset 与 native DTD count。

- [x] Data Block 与 DTD 不重叠，DTD offset/native count/checksum 自动维护。
- [x] 空间不足、DTD 不可表示、非法输入时返回结构化错误。
- [x] CTA padding 清零，不发生静默截断。
- [x] 未修改 raw block 通过原始模型保留。

**Verification:** 边界测试覆盖 0/1/最大长度 block、满 DTD 区、无空间和 CTA byte 127 边界；`cargo test --all-targets`。

**Dependencies:** Task 4

**Files likely touched:** `src/extensions.rs`, `src/edid.rs`, `src/error.rs`, `tests/core_regression.rs`

**Estimated scope:** Large；必要时拆分 layout 与 DTD writer。

#### Task 6: 提供 CTA mutation 与完整 roundtrip API

**Description:** 在 CTA layout 基础上增加 Data Block 增删改、DTD 增删改和 extension 生命周期操作，确保修改 CTA 后仍能通过完整 EDID checked serialization。

- [x] CTA Data Block 可替换并重新布局。
- [x] CTA DTD 与 Header capability mutation 已提供 checked API，并保持失败原子性。
- [x] 替换操作保持原子性并重新计算 checksum/offset。
- [x] 输出可被本库重新解析。

**Verification:** 集成测试覆盖 video/audio/HDR/VRR/DTD 编辑及 unknown preservation。

**Dependencies:** Task 5

**Files likely touched:** `src/edid.rs`, `src/extensions.rs`, `src/lib.rs`, `tests/core_regression.rs`

**Estimated scope:** Medium

#### Checkpoint: CTA 写入

- [x] 所有当前 CTA typed blocks 可编码。
- [x] CTA 原始集合与 DTD 自动布局通过边界测试。
- [x] CTA 构造/替换可完成 parse-edit-serialize。

### Phase 4: DisplayID 写入

#### Task 7: 定义 DisplayID typed/raw block 编码器

**Description:** 为 Product Identification、Display Parameters、Type I/Type VII Detailed Timing 和 embedded CTA 建立编码器，同时保留 unknown DisplayID block 与 raw 字段。

- [x] DisplayID raw data block 有编码器。
- [x] Product Identification、Display Parameters、Type I/Type VII timing 和 embedded CTA 已有 typed encoder；未知块与 raw tail 保留。
- [x] payload 长度非法时返回结构化错误。
- [x] raw known/unknown block roundtrip 可验证。

**Verification:** DisplayID golden bytes 和 typed roundtrip tests。

**Dependencies:** Task 2、Task 4

**Files likely touched:** `src/extensions.rs`, `src/error.rs`, `src/lib.rs`, `tests/core_regression.rs`

**Estimated scope:** Large

#### Task 8: 实现 DisplayID payload 布局与 mutation

**Description:** 实现 DisplayID Header、data block 顺序、payload 长度、padding、checksum 和 embedded CTA 的整体布局，并提供扩展级增删改操作。

- [x] payload 长度严格不超过 121 字节。
- [x] Header payload length 与实际编码结果一致。
- [x] DisplayID data block 可替换。
- [x] unknown raw block 可通过原始模型保留；布局失败不返回部分输出。

**Verification:** 121 字节边界、满容量、空 block、unknown block 和 embedded CTA 集成测试。

**Dependencies:** Task 7

**Files likely touched:** `src/extensions.rs`, `src/edid.rs`, `src/error.rs`, `tests/core_regression.rs`

**Estimated scope:** Large

### Phase 5: 统一生命周期与发布质量

#### Task 9: 统一扩展生命周期 API

**Description:** 将 Base、CTA、DisplayID 和未知 extension 统一纳入 `Edid` 的增删改排与 checked serialization，隐藏或约束容易产生非法 count/checksum 的直接操作路径，同时保持低层 raw API 的兼容性。

**Acceptance criteria:**

- [x] 支持 extension 增加、删除、替换、重排。
- [x] extension count 始终由生命周期 API 与最终序列化根据实际块数量维护/验证。
- [x] 所有扩展最终经过统一 checksum/长度/边界验证。
- [x] 兼容 `to_bytes` 与严格 `to_bytes_checked` 的行为在文档中明确。

**Verification:** 完整 EDID 多扩展编辑集成测试；`cargo test --all-targets`、clippy、rustdoc。

**Dependencies:** Task 6、Task 8

**Files likely touched:** `src/edid.rs`, `src/extensions.rs`, `src/serialize.rs`, `src/error.rs`, `src/lib.rs`

**Estimated scope:** Large；必要时拆成 extension lifecycle 和 API compatibility 两个任务。

#### Task 10: fuzz、真实 corpus、文档和发布验证

**Description:** 建立序列化输入与输出的实际质量门禁，补齐 fuzz oracle、真实 EDID corpus、README/API 文档、CHANGELOG 和 package/release 检查。

- [x] fuzz target 覆盖 checked serializer、部分 typed/raw extension paths 和 parse-edit-serialize oracle。
- [ ] 真实 EDID corpus 验证 parse roundtrip 与目标字段编辑隔离。
- [x] README 明确 lossless/canonical、兼容/严格 API 和扩展写入范围。
- [x] 添加发布说明，记录新 enum variant 和破坏性 match 风险。
- [x] CI 通过 fmt、clippy、test、doc、package、MSRV 和跨平台检查；fuzz job 已配置但本地未执行。

**Verification:** `cargo fmt --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test --all-targets`、`RUSTDOCFLAGS="-D warnings" cargo doc --no-deps`、`cargo package`。

**Dependencies:** Task 1、Task 2、Task 6、Task 8、Task 9

**Files likely touched:** `fuzz/fuzz_targets/parse_edid.rs`, `tests/core_regression.rs`, `README.md`, `tasks/plan.md`, `tasks/todo.md`, `.github/workflows/ci.yml`

**Estimated scope:** Medium

### Checkpoint: Complete

- [ ] Base、CTA、DisplayID 均具备 typed write path。
- [ ] 完整 EDID 支持 parse-edit-validate-serialize。
- [ ] Unknown/reserved 数据保留契约有测试证明。
- [ ] 所有质量门禁和 MSRV 检查通过。
- [ ] 发布说明和 API 文档完成，等待人工 review。

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| CTA/DisplayID 标准字段多、版本差异大 | High | 按 block 家族拆任务；每类使用 golden bytes 和官方标准对照；未知块保持 raw |
| 重布局导致 unknown payload 丢失 | High | typed/raw 混合模型；默认 preserve；加入 edit isolation 测试 |
| 公开 `raw` 字段导致非法对象可构造 | High | additive checked validation/output API；文档明确 `to_bytes` 与 checked API 差异 |
| checksum/offset/count 更新不一致 | High | 最终统一 layout/serialize 阶段计算，禁止调用方手工维护关键派生字段 |
| CTA/DisplayID writer 破坏现有只读 API | Medium | 只新增 encoder/model；保留现有 view 类型与 raw payload |
| EDID 标准允许大量厂商非标准数据 | Medium | 不识别内容保留 raw；拒绝不可表示的 typed 写入，不静默截断 |
| builder 设计过早锁定公共 API | Medium | 先完成内部模型与 checked contract，再发布 builder；使用 additive API |
| 真实 EDID 样本带有不规范字段 | Medium | 严格 parser、兼容 parser 分离；corpus 测试区分可解析、可保留和可重编码 |

## Resolved Design Decisions

- `Edid::to_bytes()` remains the unchecked, byte-preserving compatibility API for version 0.1; additive `validate_for_serialization()` and `to_bytes_checked()` APIs provide the safe publishing path.
- CTA/DisplayID blocks preserve input order and unknown payloads by default. Canonicalization, if added, will be an explicit mode rather than implicit writer behavior.
- Delivery is sliced: publish the strict Base Block serialization boundary before exposing incomplete CTA/DisplayID writers.
- `EdidBlock.raw` and `Edid.extensions` remain public for 0.1 compatibility. New controlled mutation APIs will be preferred; restricting fields requires a later semver-major release.
