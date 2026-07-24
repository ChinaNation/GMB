# 全仓审计整改 · 第 2 轮：移动端(TC3)

任务需求：落地审计 TC3 移动端项。承接第 1 轮(链端安全+后端鉴权,卡 20260723-audit-fix-round1)。
所属模块：citizenapp(Flutter) + citizenwallet(冷钱包) + citizenapp/cloudflare(item 14 后端协同)。

## 本轮条目与处置(用户确认)
| # | 处置 | 落点 |
|---|---|---|
| 12 | pallet 索引单源:建 `citizenapp/lib/rpc/pallet_registry.dart`,各 service 的 pallet 常量改引用(镜像冷钱包 pallet_registry) | citizenapp ~10 service |
| 17 | 冷钱包注册表补注释:FullnodeIssuance(6) 有意不登记(矿工热钱包自签绑定奖励账户,不冷签) | citizenwallet `pallet_registry.dart` |
| 16 | 启用 `use_build_context_synchronously` lint + 跑 `dart analyze` 报真实数 | citizenapp/citizenwallet `analysis_options.yaml` |
| 15 | debugPrint 收敛到编译期可关闭日志门面(release no-op) | citizenapp/citizenwallet(29 文件) |
| 14 | 客户端注销携带会话 + Worker account/delete/challenge 翻默认拒(与 TC2 协同,原因见 round1 卡) | square_api_client.dart + cloudflare request_guard/account |

## 必须遵守
- 只在 /Users/rhett/GMB 主检出;item 12 只搬既有正确常量、值不变(链上真源已核对:construct_runtime + 冷钱包注册表逐项一致)
- item 14 先改客户端携带会话,再翻后端默认拒,避免打断注销
- 死规则:扫码图标只用 scan-line.svg;展开指示器禁实心三角(本轮不涉及)

## 输出物
- pallet_registry.dart 单源 + 各 service 引用
- 日志门面 + debugPrint 迁移
- lint 启用 + analyze 真实数
- 代码 + 中文注释 + 残留清理

## 验收标准
- `dart analyze`(两端)无新增 error
- item 12:全仓 grep 无第二处 pallet 索引字面量(除 registry)
- item 14:worker vitest 绿;客户端注销流携带会话
- 残留清理干净

## 执行进度
| Step | 状态 | 说明 |
|---|---|---|
| 17 冷钱包 FullnodeIssuance 注释 | ✅ | pallet_registry.dart 补「有意不登记(6)」注释:矿工热钥自签绑定奖励账户,不冷签,冷钱包 decodeFailed 红拒属预期 |
| 12 pallet 索引单源 | ✅ | 新建 `citizenapp/lib/rpc/pallet_registry.dart`(pallet+call 常量,值对齐 construct_runtime);11 个 service 的散落常量改引用 PalletRegistry(保留本地别名、只换 RHS);消除 pallet 34 双定义;`flutter analyze` No issues |
| 16 lint 启用 + analyze | ✅ | 两端 analysis_options 显式开 use_build_context_synchronously;`flutter analyze` **两端均 No issues**——**审计的~120 疑似是 grep 噪音(mounted 在相邻行),真实数=0** |
| 15 日志门面 | ✅ | 新建 AppLog 门面两端(`citizenapp/lib/log/app_log.dart` + `citizenwallet/lib/util/app_log.dart`,kReleaseMode 编译期 no-op 并被 tree-shake);citizenapp 29 文件 205 处 debugPrint→AppLog.d(citizenwallet 无调用点);连带删 6 处已失效 foundation import + 修 show 子句;全仓 debugPrint 仅存门面内部;两端 analyze No issues |

## 本轮验证记录(全绿)
- item 12 值正确性:registry 12 个 pallet 索引逐项 == construct_runtime(脚本核对 ALL MATCH),非仅 analyze 过
- citizenapp:`flutter analyze` No issues;`flutter test` **792 passed / 5 skipped / 1 flaky**;唯一失败 `smoldot_client_lifecycle_test.dart`(真启 smoldot 计时竞态,见 [[citizenapp-test-smoldot-hermetic]]),单独重跑 16 全绿=确认 flaky 非本轮回归(AppLog 迁移在 test 模式 kReleaseMode=false 行为等同 debugPrint)
- citizenwallet:`flutter analyze` No issues;`flutter test` **190 全绿**(含 pallet_registry_test)
- worker(item 14):`vitest` **174 绿**(+1 注销默认拒回归测试)
- item 15 迁移由子代理执行,其终检 analyze 覆盖了 item12/14 改动一并 No issues
| 14 注销会话协同 | ✅ | 客户端 `_consumeAccountAction` 读缓存会话、挑战+确认都带 Bearer、未登录明确报错;Worker 删 account/delete 自证白名单臂→默认拒 requireSession;新增回归测试;worker **174 测试绿** |
