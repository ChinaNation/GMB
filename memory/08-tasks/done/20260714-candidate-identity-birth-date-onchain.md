# 竞选身份上链携带出生日期 + 链上算年龄

状态：已完成并通过验证（链端测试、citizenapp 单测/组件测试、citizenwallet/citizenweb 静态分析、官网浏览器核对）。

## 任务目标

- 注册局新增公民时出生日期必填、持久化、写入后不可修改（onchina 本已必填 + NOT NULL，无 UPDATE 触碰该列）。
- 竞选身份（`CandidateIdentity`）上链新增出生日期字段 `birth_date`（YYYYMMDD 整数）。
- 链上提供 `candidate_age`，任何调用方读公开的出生日期实时计算竞选公民年龄。
- 公民 App 与官网的「身份会员卡片 · 竞选公民 · 链上公开的身份信息」增加一条：出生日期。
- 投票身份不携带出生日期（保持最小公开面）。

## 安全边界

- 出生日期是链上公开信息，仅竞选身份携带；投票身份不含。
- 写一次即锁定：`upgrade_to_candidate_identity` 与 `update_candidate_identity` 写入前均过 `ensure_birth_date_immutable`，已存在值与入参不一致返回 `BirthDateImmutable`。
- 校验 `is_plausible_yyyymmdd` + 推导年龄 ≥ `MIN_ONCHAIN_CITIZEN_AGE_YEARS`（16），未来出生/时间戳未初始化 fail-closed。
- 开发期重新创世口径，无 migration、不 bump spec、StorageVersion 保持 1。

## 实施范围（五端逐字节一致的 SCALE 布局）

链端 `citizenchain/runtime/misc/citizen-identity/src/lib.rs`
- `CandidateIdentityPayload` / `CandidateIdentity` 末字段加 `birth_date: u32`。
- 两处写入填值 + 不可改守卫；`ensure_valid_candidate_payload` 校验；新增 `age_from_birth_date` / `candidate_age` / `is_plausible_yyyymmdd`；新增 `Error::InvalidBirthDate`、`BirthDateImmutable`。
- benchmark（`runtime/src/configs.rs`）、测试（`citizen-identity/src/tests/mod.rs`、`runtime/src/tests/cases.rs`）补字段 + 新增年龄边界/不可改/非法日期/未来出生用例。

onchina `citizenchain/onchina/src/domains/citizens/chain_identity.rs`
- 竞选 payload 末尾序列化 `birth_date`（u32 YYYYMMDD LE）；出生日期已在 PG（`citizen_birth_date`），直接复用。
- `occupy.rs` 更新路径旁加不可改注释。表单/建表不改（已必填 + NOT NULL）。

citizenwallet `citizenwallet/lib/signer/`
- `payload_decoder.dart` `_readCandidateIdentityPayload` 末尾解 `birth_date`；`field_labels.dart` 加 `'birth_date' => '出生日期'`；签名确认页展示。

citizenapp `citizenapp/lib/my/myid/`
- `voting_identity_payload.dart`（签名载荷解码 + reviewEntries 展示出生日期；修正过时路径注释 otherpallet→misc）。
- `myid_service.dart`（`_decodeCandidateIdentity` 存储结构末尾在 sex 后、updated_at 前解 `birth_date` + `MyIdState.citizenBirthDate`）。
- `myid_page.dart` 出生地后加「出生日期」行；`membership_page.dart` 竞选模板加「出生日期」。

官网 citizenweb `citizenweb/src/pages/Membership.tsx`
- 竞选档链上公开字段模板数组加「出生日期」（投票档不加）。

## SCALE 布局（三处解码器必须一致）

竞选 payload = 投票段 + `birth_province/city/town`(Compact+bytes) + `citizen_full_name`(Compact+bytes) + `citizen_sex`(u8) + `birth_date`(u32 LE)。
存储 `CandidateIdentity` = birth 三码 + full_name + sex(u8) + `birth_date`(u32) + `updated_at`(BlockNumber)。

## 验收

- 链端 `cargo check -p citizen-identity` 通过；`cargo test -p citizen-identity` 27 项全绿（含新增 4 项）；整 workspace（runtime + onchina）`cargo check` 通过。
- citizenapp `flutter test`（myid 三套）全绿；citizenwallet `dart analyze` 无问题；citizenweb `tsc --noEmit` 通过。
- 官网浏览器核对：竞选公民卡「链上公开的身份信息」显示出生日期，投票公民卡不显示。

## 残留清理

- 文档路径 `runtime/otherpallet/…` 已随代码统一到 `runtime/misc/…`：文档目录 `memory/05-modules/.../runtime/otherpallet/` 已 `git mv` 为 `misc/`，并把描述当前结构的活文档（repo-map、target-structure、unified-naming/protocols、CITIZENCHAIN/CITIZEN_IDENTITY 技术文档、runtime README、模板等）里的 `otherpallet` 全部改为 `misc`。
- 有意保留旧名的仅：记录本次改名历史的任务卡（`20260711`、`20260712`、`20260622`）与 ADR 决策记录（point-in-time，不改写历史）。
