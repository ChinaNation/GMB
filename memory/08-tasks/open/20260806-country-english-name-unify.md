# 任务卡：国名英文名全仓统一（China Nation → Chinese Nation）并重新创世

任务需求：
全仓统一国名四项定稿：中文全称「中华民族联邦共和国」、英文全称「Federal Republic of the Chinese Nation」、中文简称「中华联邦」、英文简称「Chinese Federation」。中文侧已全仓一致零改动；英文侧把现行「Federal Republic of the China Nation / the China Nation / China Federation」全量替换为 Chinese 形态。附带：`primitives/cid/code.rs` 的 `CountryCodeInfo` 增加英文全称/简称字段；宪法 `constitution.scale` 类型级往返重编；完成后重新创世。

所属模块：citizenchain（primitives / legislation-yuan / node）、citizenapp（生成物）、citizenweb（白皮书）、memory 文档

输入文档：
- memory/07-ai/institution-naming.md
- memory/05-modules/citizenchain/runtime/primitives/PRIMITIVES_TECHNICAL.md
- memory/07-ai/unified-naming.md

必须遵守：
- 不可突破模块边界
- 不可绕过既有契约
- 不可擅自修改安全红线
- constitution.scale 禁止字节级替换，必须用 pallet 自身 `ChaptersOf` 类型 decode→替换→re-encode（SCALE compact 长度前缀会变）
- 生成物（governance_registry.generated.dart、local-docs.generated.ts、frontend dist）只经生成器/构建更新，不手改
- done 任务卡为历史记录不动；open 任务卡同步
- 每步替换后 grep 复核：旧形态归零 + 新形态计数与预期一致

执行步骤：
1. 源码/文档英文名替换（scale 除外）：china_{zf,jc,lf,cb,jy,sf}.rs 26 处、constitution_shell.html 1 处、citizenweb/src/whitepaper.md 7 行、memory/07-ai/institution-naming.md 26 行、memory/08-tasks/open 全量扫描同步
2. `CountryCodeInfo` 增加 `country_full_name_en` / `country_short_name_en` 字段与断言（用户指定）
3. 一次性 Rust 往返工具重编 constitution.scale（英文全称 153 处 + 简称 2 处 + 独立 "the China Nation" 用法），跑既有解码测试后删除工具（无残留）
4. 重跑 scripts/generate_citizenapp_governance_registry.mjs 更新 citizenapp 生成物；重跑 node frontend 本地文档生成与 dist 构建
5. citizenchain 相关 crate 测试（primitives、legislation-yuan、runtime）
6. 重新创世：chainspec 重生成入库、chainspec_hash 四处同步 + manifest 回写、机构注册表生成器重跑、citizenapp assets/chainspec.json 同步、44 节点重部署、Cloudflare 侧按新创世重部署
7. GitHub 身份改名（用户自行执行）后：全仓把旧身份引用替换为新身份 ChineseFederation（LICENSE、package.json、Cargo.toml repository/homepage 与全部 polkadot-sdk fork git 依赖、Cargo.lock、download-wasm.sh、tauri.conf.json、DownloadButton.tsx、git remote、CI）

输出物：
- 代码（含 code.rs 英文字段）
- 重编后的 constitution.scale
- 生成物更新
- 测试
- 文档更新（institution-naming.md 等）
- 残留清理（一次性重编工具删除、旧英文名全仓归零）

执行记录（2026-08-06）：
- 步骤 1 完成：perl 替换 10 个文件（china_*.rs 26 处、constitution_shell.html 1 处、whitepaper.md 全称 9+独立 1+简称 1、institution-naming.md 26 处、20260704 卡 5 处），grep 复核旧形态归零、新形态计数一致
- 步骤 2 完成：`CountryCodeInfo` 增加 `country_full_name_en = "Federal Republic of the Chinese Nation"` / `country_short_name_en = "Chinese Federation"` 字段与测试断言
- 步骤 3 完成：临时 `#[ignore]` 测试用 `ChaptersOf<Test>` decode→替换→encode 回写 constitution.scale；新文件 226569 字节 = 旧 226257 + 156 处 × 2 字节精确吻合；Chinese Nation 154、Chinese Federation 2、旧形态 0；工具用后已删（cases.rs 零 diff）
- 步骤 4 完成：governance registry 生成器重跑（89 机构，diff 仅 2 处英文名）；生成器顺带最小修复——懒惰正则把 ChinaZf struct 定义体+FSC 常量误捕为数据块（FSC 常量加入后的既有断裂），extractStructs 增加「数据块必含带引号 cid_number」过滤；local-docs 生成器重跑；node frontend dist 重建
- 步骤 5 完成：legislation-yuan 45/45、primitives 78+/全过、runtime(citizenchain) 56/56
- 文档登记：unified-naming.md 新增 5.7 国名四项定稿；PRIMITIVES_TECHNICAL.md 国家码行更新
- 残留复核：全仓旧形态仅剩 done 历史卡 2 个与登记性引用（本卡、unified-naming.md 废弃旧名映射）
- 待办：②推送 CI → WASM CI → bake-chainspec.sh --finalize → 44 节点重部署（步骤 6/7）

执行记录（2026-08-07）：
- 步骤 7 完成：GitHub 身份已改为 ChineseFederation（经仓库 ID API 确认），全仓 20 个文件旧身份引用全部替换（含 citizenchain/Cargo.lock 176 处 fork 依赖 URL、两端 rust/Cargo.toml、podspec、smoldotdart pubspec、app_update_service.dart、tauri.conf.json、DownloadButton.tsx、LICENSE、package.json、memory 文档），残留 0；git remote 与仓库级 user.name 已同步

验收标准：
- 全仓 `rg -i "china nation|china federation"` 除 done 历史任务卡外全 0
- 新形态计数与替换总数一致
- citizenchain 相关 crate 测试通过；constitution.scale 解码测试通过
- 重新创世完成、44 节点上线、四端 chainspec 同步
- GitHub org 引用替换完成（待新 org 名）
- Review 问题已处理
