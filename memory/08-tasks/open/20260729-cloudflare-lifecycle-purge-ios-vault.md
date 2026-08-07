# 20260729 Cloudflare 生命周期清理 + purge 注释订正 + iOS 硬件金库(线程 B)

状态:open
所属模块:citizenapp(cloudflare Worker + ios 原生)
线程边界:本卡由**线程 B** 独占执行;线程 A 见
`20260729-citizenapp-local-at-rest-encryption.md`。两线程共用
`/Users/rhett/GMB` 同一工作树(禁 worktree),必须严守下方文件边界,不得越界改文件。

## 任务需求

三项互不依赖、与线程 A 零文件重叠的收尾工作。

### 1. 退订清理范围重做(❌ 未做)

**现状**:退订满 90 天**只归档视频**(Stream 导出 R2 冷存后删 Stream),
正文、manifest、图片一动不动 —— `cloudflare/src/membership/archive.ts:14`、
SQL 写死 `media_kind = 'video'`。

**新目标(用户 2026-07-29 决策)**:权益到期后云端**彻底清空**,只留本地 ——
删正文(`square_posts`)、R2 manifest、Cloudflare Images、Stream 及归档对象。

注意与既有事实的冲突,须在方案中正面处理:
- 媒体(照片/视频)按用户决策**只存云端、本地不双存**;云端删净后媒体即永久消失,
  本地只剩文字。这是决策的必然后果,需在方案中写明并再次确认。
- `restoreAccountVideos` 当前是**死代码**(生产零调用点,仅测试引用),
  重新订阅不会自动解冻;新方案须明确其去留。

### 2. purge 注释与保留策略订正(◻️ 非漏删,是文档矛盾)

`purge.ts` **不删** `chain_transaction_confirmations` / `topup_orders` /
`square_login_challenges` 三表,这是**已确认的保留决策**(链交易证明 + 财务台账 +
挑战阶段主体),见 `done/20260728-cloudflare-cid-identity-primary-key.md:105`。

**问题在注释**:`cloudflare/src/account/purge.ts:15` 写"硬删除某账户在 Cloudflare 的
**全部**数据",与保留策略自相矛盾。

**本项只改注释与文档,严禁改删除逻辑**,不得把三表加进删除清单。

### 3. iOS 硬件金库(⚠️ 只完成 Step 1)

- **已完成**:通用 Keychain 单源加固 `lib/security/secure_storage.dart`
  (iOS `first_unlock_this_device` + 不随 iCloud 同步)。
- **未完成**:iOS 原生侧无 `hw_seed_vault` / `device_subkey` 通道,
  无 Secure Enclave / `SecAccessControl`;见
  `open/20260728-citizenapp-ios-vault-secure-storage.md:40`。

**已知阻塞**:本机 `xcode-select` 只有 CommandLineTools,无模拟器运行时、无 CocoaPods、
无 iOS 真机;`~/Downloads/Xcode_27_beta_4.xip`(1.8G)已下载但未安装。
且模拟器无 Secure Enclave,硬件绑定必须真机验收
(任务卡铁律:硬件金库类改动真机 e2e 通过后才部署)。
**建议顺序**:先做第 1、2 项(Worker,无阻塞),iOS 等环境就绪再动。

技术要点:Apple SE 只支持 **P-256**,Android 那套 RSA-2048-OAEP KEK **不能照搬**;
iOS 路径为 Keychain `SecAccessControlCreateWithFlags(.biometryCurrentSet)`
或 SE P-256 + ECIES + `LAContext`。另需先验证
`flutter_secure_storage` 的 `biometryCurrentSet` 是否已等价(若是则可不写原生桥)。

## 文件边界(线程 B 独占,线程 A 不得触碰)

- `citizenapp/cloudflare/src/membership/archive.ts`
- `citizenapp/cloudflare/src/account/purge.ts`(仅注释)
- `citizenapp/cloudflare/test/{archive,account}.test.ts`
- `citizenapp/ios/**`
- `citizenapp/lib/wallet/core/hardware_bound_seed_vault.dart`
- `memory/08-tasks/open/20260728-citizenapp-ios-vault-secure-storage.md`

**不属于本卡**:`lib/isar/app_isar.dart`、`lib/chat/**`、`lib/my/user/contact_service.dart`、
`rust/src/chat_mls.rs`、`lib/wallet/core/wallet_manager.dart`(全部归线程 A)。

## 执行方式(用户明确要求)

每一步:**先出技术方案 → 等用户确认 → 执行 → 更新文档 + 完善注释 + 完善测试 +
清理残留 → 输出下一步技术方案**。不得连跳。

## 验收标准

- 退订到期后云端确无正文/manifest/Images/Stream/归档残留(真实 D1 + R2 验证)
- purge 注释与三表保留决策一致,文档同步,删除逻辑零改动
- iOS 原生金库真机 e2e 通过(无真机则明确标注阻塞,不得用模拟器冒充)
- 单测覆盖、文档更新、残留清理
