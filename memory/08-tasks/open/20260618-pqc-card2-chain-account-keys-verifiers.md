# PQC card2:链端 GmbPqcAuth 扩展授权 + account-keys + 验签器 + L3 PQC(seal 已移 card7)

关联决策:`memory/04-decisions/ADR-022-unified-pqc-crypto.md`(§3/§4/§5/§6)
状态:open(依赖 card1 spike 全绿)

任务需求(活链 runtime 升级,地址/余额/权限不变):
1. **GmbPqcAuth TransactionExtension(路线 A)**:
   - `validate` 验 ML-DSA-65 后**返回 `Signed(account)`**(`prepare` 不改 origin);后续 CheckNonce/ChargeTransactionPayment 走标准逻辑。
   - 🔴 **嵌套 `(GmbPqcAuth, AuthorizeCall)` 占第一项**,outer 仍 12 项,不加第 13 项。
   - 🔴 **(v5 修正:创世前对 PQC 零改动,本卡=创世后启用 PQC 的 setCode)**:创世前不加 GmbPqcAuth、`transaction_version` 保持 0;**本次 setCode 把 TxExtension 第一项改成嵌套 `(GmbPqcAuth, AuthorizeCall)` → `transaction_version` 0→1++ 且 `spec_version++`**(`CheckTxVersion` 据此拒未升级旧客户端,那一刻推送 PQC 版客户端=计划内)。详见 ADR §16。
   - 🔴 **(v5)`extra={None|Pqc{account,sig,key_version}|Bootstrap{account,pqc_pubkey,ml_dsa_sig,sr25519_bootstrap_sig,key_version}}`**——`Pqc/Bootstrap` **必带 account**:ML-DSA 无公钥恢复,origin=None 下 validate 必须靠 extra 内 account 查 `AccountPqcKey[account]` 取验签公钥;account 是查表 hint,随后被 ML-DSA 签名+payload 内 account 双重绑定(填他人 account 伪造不出签名)。
   - 🔴 **(v5)`extra=None` 透传语义**:`extra=None` 不改 origin、**不写 storage**;但对 sr25519 signed origin,`validate` **仍读 `AccountPqcKey[signer]`+`PqcPolicy`** 判"已绑定拒 sr25519";origin=None 的 authorized call 直接透传 AuthorizeCall 不误伤。
   - 🔴 **授权模式与 origin 互斥**:`Pqc|Bootstrap` 只许 General Transaction(origin=None);sr25519 signed 夹带 PQC proof → `BadProof`。
   - 🔴 **(v5)`weight()` 纯 self.extra 路由 card1 benchmark 常量、禁读 storage**(weight 须在 PqcPolicy decode 前可算);**`validate()` 可读 storage**(查 AccountPqcKey/PqcPolicy)——二者分工不矛盾;Bootstrap weight 取最坏。
   - 🔴 **(v5)fail-safe 拆三语境**:① 创世初值 `phase=A`/`reject=false`/`allow_bootstrap_unbound=false`(PQC 未 setCode、Pqc/Bootstrap reject-stub);② 正常运营值治理逐阶段设(Phase B 起 已绑定 reject=true、未绑定 allow_bootstrap=true);③ **decode 失败/缺失 fallback = 等价 phase=A 安全态(sr25519 不冻结 + PQC/bootstrap 拒)**,绝不 fail-closed 冻链、绝不 fail-open 开 bootstrap;此 fallback 瞬态(setCode 修复即恢复),非永久锁人。
2. **account-keys pallet(idx=27)只承载存储/策略/查询/事件/轮换,不承载主交易派发**:
   - `AccountPqcKey{alg,key_version,pubkey:BoundedVec<u8,ConstU32<2048>>,bound_at}`(删 bootstrap_mode;2048 只覆盖 ML-DSA-65,87 须新 alg+新 schema);`PqcPolicy{phase,bootstrap_deadline,reject_sr25519_when_bound,allow_bootstrap_unbound}` 本 setCode 建表初值 phase=A(见上 fail-safe 三语境)。
   - 🔴 **(H3)轮换双签**:当前 PQC 私钥授权 + **新私钥对(新公钥+key_version+account+genesis)自签 PoP**,两签过才 key_version++。
3. **bootstrap**:
   - 🔴 **(B1)challenge 钉死**:`sr25519_sign(blake2_256(DOMAIN_BOOTSTRAP++SCALE(genesis,spec,tx_version,account,pqc_pubkey_hash,key_version,nonce,call_hash,following_extensions_hash)))`;sr25519 **必须覆盖 pqc_pubkey_hash**;ML-DSA 反向覆盖 `blake2_256(sr25519_sig)`(双向交叉绑定)。
   - 验序:① `blake2_256(body.pqc_pubkey)==pqc_pubkey_hash` → ② sr25519 challenge → ③ ML-DSA → 写绑定。
   - 🔴 **(H1)绑定写 post_dispatch,绝不返回 Err**(冲突判定前移 validate 拒;post_dispatch 未绑定→写/已绑定→no-op Ok)——返回 Err 会作废整区块=DoS。
   - 🔴 **(H2 v5 修正)bootstrap 账户须有 AccountInfo(有余额即 providers>0)且余额≥手续费**;机制纠正:**CheckNonce 只查 nonce 窗口,费由 ChargeTransactionPayment 拒**(非 CheckNonce);provider=0 但余额充足应过 CheckNonce、在 ChargeTransactionPayment 生效 + 单测;客户端构造前检查账户存在性+可付余额。
   - 🔴 **(H12 v5)txpool DoS 具体节点级机制**:body 长度上限(~10KB)硬校验 + cheap-checks(decode/AccountPqcKey 存在/alg 匹配/nonce 窗口)全在 ML-DSA 验签**之前** + **node 层 `(account,source)` 失败签名滑窗计数 + 未绑定 bootstrap 池占比上限 + 超限淘汰**(Substrate txpool 无内建 per-account 限速,须自实装);单测 100 笔同账户错误签名 bootstrap 不拖垮池。
4. **payload/反重放**:域标签统一 `GMB_PQC_TX_MLDSA65_V1`/`GMB_PQC_BOOTSTRAP_MLDSA65_V1`(算法标识进域字面量;字段集对齐,bootstrap 补 tx_version/tip/era;含 `following_extensions_hash`);🔴 **(B3)`following_extensions_hash` = SDK `inherited_implication` 精确递归编码**(ImplicationParts{base,explicit,implicit},非扁平拼接),单测断言链端 `inherited_implication.encode()` 与协议口径逐字节相等;hash 全 blake2_256;**era immortal**(CheckMortality.implicit 仍 genesis,纳入 hash);**CheckMetadataHash 保持 Disabled**(implicit=None);txpool provides=(account,nonce) 由 CheckNonce 自动产生,GmbPqcAuth 不重复设。
5. **5 个 CID/机构验签器 algo-tag 路由**(`configs/mod.rs:781/890/961/1037` + organization-manage)→ verify_by_algo。🔴 **(B6)与 card4(CID MAIN signer 迁 ML-DSA)原子同批上线**,或链端先双 algo 路由再切。
6. **ShengSigningPubkey/ShengAdmins**:🔴 **(H6)value(签名公钥)→ BoundedVec 容 ML-DSA ~1952B;key 索引保持 `[u8;32]` admin 身份键不动**;4 验签器 from_raw(32)+sr25519_verify 整体改 verify_by_algo。
7. 🔴 **(H5)6 处签名长度上限放宽**:`MaxRegisterSignatureLength(:800)`/`MaxCredentialSignatureLength(:970)`/`MaxVoteSignatureLength(:1410)`/population-snapshot 签名上限/`MaxBatchSignatureLength` → 容 ML-DSA(~3309B,建议 ConstU32<4096>)。
8. **(B4 v5 真实行号)L3/offchain PQC 授权(ADR §5.1)**:
   - 🔴 `configs/mod.rs:1261` `MaxBatchSignatureLength=ConstU32<128>` → `ConstU32<4096>`(128B 装不下 ML-DSA ~3309B)。
   - 🔴 `settlement.rs:169` 当前 `Sr25519Signature::try_from(&item.payer_sig[..])` 硬编 sr25519 → `verify_by_algo` 按 `AccountPqcKey[payer]` 路由;`lib.rs:644-663` `sr25519_pubkey_from_account` 假设作废。
   - 🔴 payer_sig/batch_signature 改 `{sig_alg:u8,key_version:u32,sig:BoundedVec<u8,4096>}` 变长结构。
   - 🔴 **`settlement.rs` 当前无 PqcPolicy phase 检查 → 必须加**(这才真堵后门,只改长度不够):Phase C/D 已绑定拒 sr25519 payer、Phase D 后未绑定不可继续 sr25519 L3 花费;未绑定 sr25519 共存到 Phase D。offchain worker 预聚合须同步感知 PqcPolicy(或 settlement 提交前再验),避免 L3/L2 phase 视图不一致。

> **seal 共识签名 ML-DSA 已移 card7**(节点二进制协调升级,非本卡 setCode 范围)。

所属模块:Blockchain(runtime)

必须遵守:绝不扩 MultiSignature;AccountId 永远 sr25519 锚点;授权唯一走 GmbPqcAuth;account-keys 不派发主交易;依赖 card1 spike S2/S3 结论。

输出物:GmbPqcAuth 扩展 + account-keys(存储/策略/轮换PoP)+ 5验签器 + 6签名上限 + L3 PQC + 中文注释 + 单测(下列)+ benchmark + 文档。

验收标准:
- 嵌套 tuple 编译+按序;following_extensions_hash 链端/钱包逐字节一致;CheckNonce/ChargeTransactionPayment 真实生效。
- 未绑定首次 bootstrap+execute 成功(post_dispatch 写、内层失败绑定仍留、**冲突不作废区块**);已绑定后续 PQC 成功;已绑定 sr25519 被拒、PQC 不误伤;None+authorized call 透传成功;sr25519 夹带 PQC proof 被拒;改 alg 降级被拒;挪用其它域 sr25519 构造 bootstrap 被拒。
- 5 验签器分流、ShengSigningPubkey value 容 ML-DSA、6 签名上限放宽生效;L3 已绑定拒 sr25519 payer(settlement 有 PqcPolicy phase 检查)、未绑定 sr25519 可用、MaxBatchSignatureLength=4096;transaction_version 本 setCode 0→1 bump(未升级旧客户端被 CheckTxVersion 拒);fail-safe 三语境单测(建表 phase=A / decode 失败不冻链不开 bootstrap);真实运行态验收。
