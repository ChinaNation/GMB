# 20260704 第44条修复与国家级机构码补齐

## 任务需求

1. 修复公民宪法第 44 条第一款中的“国聘公职人员”残留，统一为“聘用公职人员”，并同步英文。
2. 补齐以下国家级公权机构的机构码、中文全称、中文简称、英文全称和英文简称：
   - 公民生活保障部食品药品监督管理局 / 食品药品监管局 / Food and Drug Administration of the Ministry of Citizen Welfare / Food and Drug Administration / `FDA`
   - 国土安全部国民警卫局 / 国民警卫局 / National Guard Bureau of the Ministry of Homeland Security / National Guard Bureau / `NGB`
   - 国家防务部陆军部 / 陆军部 / Department of the Army of the Ministry of National Defense / Department of the Army / `ARM`
   - 国家防务部海军部 / 海军部 / Department of the Navy of the Ministry of National Defense / Department of the Navy / `NAV`
   - 国家防务部空军部 / 空军部 / Department of the Air Force of the Ministry of National Defense / Department of the Air Force / `AIR`
   - 国家防务部天军部 / 天军部 / Department of the Space Force of the Ministry of National Defense / Department of the Space Force / `SPF`
   - 国家防务部联合作战参谋部 / 联合作战参谋部 / Joint Operations Staff of the Ministry of National Defense / Joint Operations Staff / `JOS`
   - 中华民族联邦共和国陆军司令部 / 陆军司令部 / Army Command of the Federal Republic of the China Nation / Army Command / `ARC`
   - 中华民族联邦共和国海军司令部 / 海军司令部 / Navy Command of the Federal Republic of the China Nation / Navy Command / `NVC`
   - 中华民族联邦共和国空军司令部 / 空军司令部 / Air Force Command of the Federal Republic of the China Nation / Air Force Command / `AFC`
   - 中华民族联邦共和国天军司令部 / 天军司令部 / Space Force Command of the Federal Republic of the China Nation / Space Force Command / `SFC`
   - 中华民族联邦共和国国民警卫队司令部 / 国民警卫队司令部 / National Guard Command of the Federal Republic of the China Nation / National Guard Command / `NGC`

## 范围

- `citizenchain/runtime/public/legislation-yuan/src/constitution.scale`
- `citizenchain/runtime/primitives/cid/code.rs`
- `citizenapp/lib/citizen/shared/institution_code_label.dart`
- `citizenwallet/lib/signer/institution_code.dart`
- `citizenchain/onchina/src/` 机构码分类镜像
- `memory/05-modules/` 相关技术文档

## 边界

- 本任务只补机构码和修复第 44 条。
- 不把这些机构写入创世机构常量；创世写入作为下一步任务处理。
- 不新增旧命名兼容分支。

## 验收

- 第 44 条结构化宪法可解码，且不再含“国聘公职人员”现行残留。
- 新机构码与既有机构码不冲突。
- runtime、CitizenApp、CitizenWallet、OnChina 的机构码分类保持一致。
- 文档已同步，注释和残留已清理。

## 执行记录

- 已修复 `constitution.scale` 第 44 条第一款为“任意合法公民均可参与聘用公职人员竞聘”，英文同步为 “Any lawful citizen may compete for positions as employed public officials.”
- 已将 `INSTITUTION_CODE_INFOS` 从 92 码扩展为 104 码，A 国家级从 26 码扩展为 38 码。
- 已同步 CitizenApp、CitizenWallet 和 OnChina 既有机构码分类镜像。
- 已在 `memory/07-ai/institution-naming.md` 登记 12 个机构的中英文全称、简称和机构码。
- 已将国家立法院参议会 `NSN`、国家立法院众议会 `NRP` 排序移动到国家立法院 `NLG` 下方,避免继续贴近储备体系。
- 已将省联邦立法院参议会 `PSN`、省联邦立法院众议会 `PRP` 排序移动到省联邦立法院 `PLG` 下方。
- 已清理现行文档中的 92 码口径残留；“国聘”仅保留在历史任务记录或本任务需求/验收描述中。

## 验证记录

- `cargo fmt --manifest-path citizenchain/runtime/primitives/Cargo.toml --check`：通过。
- `cargo test --manifest-path citizenchain/runtime/primitives/Cargo.toml cid::code::tests`：通过，7 个测试全绿。
- `cargo test --manifest-path citizenchain/runtime/public/legislation-yuan/Cargo.toml constitution`：通过，10 个测试全绿。
- `dart format --output=none --set-exit-if-changed citizenapp/lib/citizen/shared/institution_code_label.dart citizenwallet/lib/signer/institution_code.dart`：通过，0 个文件变更。
- `cargo check --manifest-path citizenchain/onchina/Cargo.toml`：通过。
- `cargo fmt --manifest-path citizenchain/onchina/Cargo.toml --check`：未作为本任务通过项；报出既有未格式化文件 `citizenchain/onchina/src/domains/gov/service.rs`，未擅自改动无关文件。
