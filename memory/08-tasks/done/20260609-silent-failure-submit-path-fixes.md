# 任务卡:交易提交路径静默失败修复(node + wuminapp,13 项)

## 任务需求

silent-failure-hunter 扫描 node/wuminapp 交易提交路径发现 13 处静默失败(3 CRITICAL / 4 HIGH / 4 MEDIUM / 2 LOW):失败被伪装成"成功"、校验失败仍继续提交、错误只吞不报。按已确认技术方案修复,完成后更新文档、完善注释、清理残留。

所属模块:citizenchain/node(Blockchain Agent)+ wuminapp(Mobile Agent)

## 必须遵守

- 不动 runtime 行为(L-1 仅核对 deposit_event 是否无条件触发)
- 不动 spec_version / 不 setCode / 不重新创世
- wuminapp 两档提交标准不破坏:普通转账 submit-only+后台 watch;提案创建类 InBlock+事件核对(builder:139 既定约定)
- 失败如实上报,绝不占位值伪装成功;无法即时确认的用后台异步核对,不阻塞 UI

## 修复清单

A 组 node(Rust):
- [ ] C-1 signing.rs:493-512 dry-run 返回 InvalidTransaction(Future/Stale/BadProof…)一律拒绝提交,抽 classify_invalid_tx 共用 helper
- [ ] C-2 signing.rs:532 提交结果 as_str().unwrap_or("unknown") → ok_or_else 上抛
- [ ] C-3 signing.rs:490-492 dry-run 结果解码失败/为空 → 拒绝提交(保留"dry-run RPC 不可用"的可用性兜底)
- [ ] H-2 submitter.rs:137-141 lookup_nonce unwrap_or(0) → 错误上抛
- [ ] M-3 signing.rs:742-746 now_secs 时钟异常静默返回 epoch0 → 显式失败
- [ ] 横切:提交成功后非阻塞后台核对(N 秒后查 nonce 是否消费/是否仍在池),未上链打告警日志
- [ ] L-1 核对 onchain-transaction emit_fee_share_burn 的 deposit_event 是否无条件触发(只读)

B 组 wuminapp(Dart):
- [ ] H-1 submitProposeSafetyFund/submitProposeSweep 从 _signAndSubmit 切换 _signAndSubmitInBlock;泛化 _confirmTransferProposedEvent → _confirmProposalEvent(参数化事件名+匹配键,三类提案共用);返回 proposalId;两个调用页对齐主转账等待/成功形态
- [ ] H-3 duoqian_transfer_page.dart:112-120 余额查询失败:记日志 + UI"余额可能已过期"提示
- [ ] H-4 service:867/975 SCALE 解码 catch(_) → 记日志后返回 null
- [ ] M-1 service:432-435 fetchProposalDisplayId catch 加日志
- [ ] M-2 service:655/659/676 safetyFund/sweep/jointTag 查询 catch 加日志
- [ ] M-4 page:395-397 Isar 写入失败 catch 加日志
- [ ] L-2 account_balance_snapshot_store.dart:125 缓存解析失败 catch 加日志

## 验收标准

- cargo check/test(node 相关 crate)通过;flutter analyze 通过
- 全部失败路径有日志或上抛,无裸 catch(_){} 残留(本次触及文件内)
- 文档/注释更新;残留清理;任务卡回写执行记录

## 执行记录(2026-06-09 完成)

A 组 node(Rust,3 文件):
- [x] C-1/C-3 governance/signing.rs:dry-run 块重写——结果解码失败/为空→拒绝;InvalidTransaction(含 Future/Stale)→拒绝并抛 `classify_invalid_tx` 可读原因;保留"dry-run RPC 不可用"可用性兜底
- [x] C-2 提交结果 `ok_or_else` 上抛,删 `unwrap_or("unknown")`
- [x] 横切 `spawn_post_submit_audit`:提交后 90 秒后台核对 nonce 是否消费(accountNextIndex 含就绪队列),未消费打 ⚠ 告警,纯观测不阻塞
- [x] M-3 `now_secs()` → `Result`,时钟早于 epoch 显式失败;3 处调用加 `?`(含 runtime_upgrade/signing.rs 漏网一处,编译揪出)
- [x] H-2 submitter.rs `lookup_nonce` → `Result`,删 `unwrap_or(0)`
- [x] L-1 核对:`emit_fee_share_burn` 仅 amount==0 跳过(零额销毁无事可报,语义正确),非零无条件 `deposit_event` ✅ 非静默失败,不改
- [x] 单测:classify_invalid_tx 已知变体/UnknownTransaction/越界不 panic + now_secs 正值,**8 passed 0 failed**;cargo fmt + check 过

B 组 wuminapp(Dart,5 文件):
- [x] H-1 safety-fund/sweep 切 `_signAndSubmitInBlock` + `_confirmProposalEvent`(泛化:`_findProposalIdInEvents` 共用扫描骨架 + `_decodeSafetyFundProposedEvent`/`_decodeSweepProposedEvent`,事件序号 3/6 按声明序)返回 proposalId;两调用页 SnackBar 改"提案已创建(#id)";删除已无调用方的 `_signAndSubmit`(残留清理)
- [x] H-3 余额刷新失败:debugPrint + `_balanceStale` 置位 + UI"(链上刷新失败,金额可能已过期)"
- [x] H-4 两处 SCALE 解码 catch 加日志;**顺手补同类 2 处**(SafetyFundAction/SweepAction 解码,与 H-4 同模式且会让 M-2 外层日志失效)
- [x] M-1/M-2/M-4/L-2 全部加带上下文 debugPrint;page:108 快照写失败也补日志
- [x] 保留 2 处合理 catch(_)(`_ss58AddressToAccountId`/地址校验:转译为明确错误,非吞错)
- [x] dart format + flutter analyze **No issues** (3 轮)

文档:DUOQIAN_TRANSFER_APP_TECHNICAL.md 补"三类提案统一标准"段(InBlock+事件核对、事件序号、submit-only 删除、错误处理铁律)。

遗留(非本卡,记录备查):admin_unlock.rs/onchain mod.rs/activation.rs 各有本地 `now_secs` 副本(同 unwrap_or_default 模式,用途为记录时间戳非 QR TTL,风险低);未扫描文件中的裸 catch(_) 不在本卡范围。
工作树未提交:与你其它线程改动(sfid/cpms 等)混在工作区,提交由你统一决定。
