# PQC card2:链端 GmbPqcAuth 扩展授权 + account-keys + 验签器 + L3 PQC(seal 已移 card7)

关联决策:`memory/04-decisions/ADR-022-unified-pqc-crypto.md`(§3/§4/§5/§6)
状态:open(依赖 card1 spike 全绿)

任务需求(活链 runtime 升级,地址/余额/权限不变):
1. **GmbPqcAuth TransactionExtension(路线 A)**:
   - `validate` 验 ML-DSA-65 后**返回 `Signed(account)`**(`prepare` 不改 origin);后续 CheckNonce/ChargeTransactionPayment 走标准逻辑。
   - 🔴 **嵌套 `(GmbPqcAuth, AuthorizeCall)` 占第一项**,outer 仍 12 项,不加第 13 项。
   - 🔴 **本次升级 `spec_version++` 且 `transaction_version`(0→1)++**(改 TxExtension 结构;`CheckTxVersion` 明确拒旧口径)。
   - `extra={None|Pqc{sig,key_version}|Bootstrap{pqc_pubkey,ml_dsa_sig,sr25519_bootstrap_sig,key_version}}`;**`extra=None` 返回未改动原 origin 透传 AuthorizeCall**(authorized call 不误伤);仅 `extra=None`+sr25519 signed origin 才判"已绑定拒 sr25519"。
   - 🔴 **授权模式与 origin 互斥**:`Pqc|Bootstrap` 只许 General Transaction(origin=None);sr25519 signed 夹带 PQC proof → `BadProof`。
   - 🔴 **`weight()` 纯 self.extra 路由 card1 benchmark 常量,禁读 storage**;Bootstrap 取最坏。
   - 🔴 **fail-open**:PqcPolicy 缺失/解码失败 → 安全默认(phase=B/reject=false),绝不冻结全链。
2. **account-keys pallet(idx=27)只承载存储/策略/查询/事件/轮换,不承载主交易派发**:
   - `AccountPqcKey{alg,key_version,pubkey:BoundedVec<u8,ConstU32<2048>>,bound_at}`(删 bootstrap_mode);`PqcPolicy{phase,bootstrap_deadline,reject_sr25519_when_bound,allow_bootstrap_unbound}` 安全默认。
   - 🔴 **(H3)轮换双签**:当前 PQC 私钥授权 + **新私钥对(新公钥+key_version+account+genesis)自签 PoP**,两签过才 key_version++。
3. **bootstrap**:
   - 🔴 **(B1)challenge 钉死**:`sr25519_sign(blake2_256(DOMAIN_BOOTSTRAP++SCALE(genesis,spec,tx_version,account,pqc_pubkey_hash,key_version,nonce,call_hash,following_extensions_hash)))`;sr25519 **必须覆盖 pqc_pubkey_hash**;ML-DSA 反向覆盖 `blake2_256(sr25519_sig)`(双向交叉绑定)。
   - 验序:① `blake2_256(body.pqc_pubkey)==pqc_pubkey_hash` → ② sr25519 challenge → ③ ML-DSA → 写绑定。
   - 🔴 **(H1)绑定写 post_dispatch,绝不返回 Err**(冲突判定前移 validate 拒;post_dispatch 未绑定→写/已绑定→no-op Ok)——返回 Err 会作废整区块=DoS。
   - 🔴 **(H2)bootstrap 账户须 providers/sufficients>0**(否则 CheckNonce 以 Payment 先拒),provider=0 给明确错误语义 + 单测。
   - 🔴 **(H12)body 长度上限硬校验(~10KB)+ 未绑定 bootstrap 按 (account,source) 限速**。
4. **payload/反重放**:`GMB_PQC_TX_V1`/`GMB_PQC_BOOTSTRAP_V1`(字段集对齐,bootstrap 补 tx_version/tip/era;含 `following_extensions_hash`);🔴 **(B3)`following_extensions_hash` = SDK `inherited_implication` 精确递归编码**(ImplicationParts{base,explicit,implicit},非扁平拼接),单测断言链端 `inherited_implication.encode()` 与协议口径逐字节相等;hash 全 blake2_256;**era immortal**(CheckMortality.implicit 仍 genesis,纳入 hash);**CheckMetadataHash 保持 Disabled**(implicit=None);txpool provides=(account,nonce) 由 CheckNonce 自动产生,GmbPqcAuth 不重复设。
5. **5 个 CID/机构验签器 algo-tag 路由**(`configs/mod.rs:781/890/961/1037` + organization-manage)→ verify_by_algo。🔴 **(B6)与 card4(CID MAIN signer 迁 ML-DSA)原子同批上线**,或链端先双 algo 路由再切。
6. **ShengSigningPubkey/ShengAdmins**:🔴 **(H6)value(签名公钥)→ BoundedVec 容 ML-DSA ~1952B;key 索引保持 `[u8;32]` admin 身份键不动**;4 验签器 from_raw(32)+sr25519_verify 整体改 verify_by_algo。
7. 🔴 **(H5)6 处签名长度上限放宽**:`MaxRegisterSignatureLength(:800)`/`MaxCredentialSignatureLength(:970)`/`MaxVoteSignatureLength(:1410)`/population-snapshot 签名上限/`MaxBatchSignatureLength` → 容 ML-DSA(~3309B,建议 ConstU32<4096>)。
8. **(B4)L3/offchain PQC 授权**:`settlement.rs:172`/`lib.rs:644-663` 废 `sr25519_pubkey_from_account` 假设;payer_sig/batch_signature 改**带 sig_alg 标签的变长结构**,ML-DSA 走 AccountPqcKey 查公钥;**Phase C/D 已绑定账户拒 sr25519 payer**,未绑定 sr25519 仍可用(双算法共存到 Phase D)。

> **seal 共识签名 ML-DSA 已移 card7**(节点二进制协调升级,非本卡 setCode 范围)。

所属模块:Blockchain(runtime)

必须遵守:绝不扩 MultiSignature;AccountId 永远 sr25519 锚点;授权唯一走 GmbPqcAuth;account-keys 不派发主交易;依赖 card1 spike S2/S3 结论。

输出物:GmbPqcAuth 扩展 + account-keys(存储/策略/轮换PoP)+ 5验签器 + 6签名上限 + L3 PQC + 中文注释 + 单测(下列)+ benchmark + 文档。

验收标准:
- 嵌套 tuple 编译+按序;following_extensions_hash 链端/钱包逐字节一致;CheckNonce/ChargeTransactionPayment 真实生效。
- 未绑定首次 bootstrap+execute 成功(post_dispatch 写、内层失败绑定仍留、**冲突不作废区块**);已绑定后续 PQC 成功;已绑定 sr25519 被拒、PQC 不误伤;None+authorized call 透传成功;sr25519 夹带 PQC proof 被拒;改 alg 降级被拒;挪用其它域 sr25519 构造 bootstrap 被拒。
- 5 验签器分流、ShengSigningPubkey value 容 ML-DSA、6 签名上限放宽生效;L3 已绑定拒 sr25519 payer、未绑定 sr25519 可用;transaction_version 已 bump 旧口径被明确拒;真实运行态验收。
