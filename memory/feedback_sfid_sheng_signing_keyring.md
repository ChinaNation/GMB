# SFID 省级签名密钥铁律

任务卡 `20260409-sfid-sheng-admin-per-province-keyring` 完成后的强制约定:

## 链端
- 链端新增 `sfid_system::ShengSigningPubkey` 和 `ProvinceBySigningPubkey` 两张 storage map,永远保持对偶关系(通过 `set_sheng_signing_pubkey` extrinsic 原子维护)
- **业务 extrinsic 的 verifier(register_sfid_institution 等)要求 origin pubkey 属于某省 signing key**,`signing_province = None` 时回退到 SFID MAIN 验签(向后兼容模式)
- **2026-04-20 起升级到 DUOQIAN_V1 + op_tag 统一域**：所有 SFID 系统与链端之间的签名 payload 第一字段为 `DUOQIAN_DOMAIN = b"DUOQIAN_V1"`(10 字节),紧跟 1 字节 `op_tag`(`OP_SIGN_BIND=0x10` / `OP_SIGN_VOTE=0x11` / `OP_SIGN_POP=0x12` / `OP_SIGN_INST=0x13`)区分业务,再拼 `genesis_hash` 和业务字段。地址派生也统一到同一域(`OP_MAIN=0x00` / `OP_FEE=0x01` / `OP_STAKE=0x02` / `OP_AN=0x03` / `OP_PERSONAL=0x04`)。所有旧域前缀(`GMB_SFID_V1` / `DUOQIAN_SFID_V1` / `DUOQIAN_PERSONAL_V1` / `FEIYONG_SFID_V1` / `ANQUAN_SFID_V1` / `GMB_SFID_*_V2/V3`)彻底退役,按 `feedback_no_compatibility.md` 不留兼容。常量见 `primitives::core_const::{DUOQIAN_DOMAIN, OP_*}`。
- **2026-04-21 起 op_tag 新增 `OP_INSTITUTION=0x05`**：专供 SFID 登记机构的自定义命名账户（临时/工资/运营...），派生 `blake2_256(DUOQIAN_V1 || 0x05 || ss58 || sfid_id || account_name)`。`OP_MAIN`/`OP_FEE` 统一为 `ss58 || sfid_id`（无 account_name 后缀）——宪法机构和 SFID 机构的主账户/费用账户派生公式**彻底对齐**。链端以 `InstitutionAccountRole` 枚举三分派（`Main`/`Fee`/`Named(account_name)`），保留名 `"主账户"`/`"费用账户"` 强制走 `Role::Main`/`Role::Fee`，不得作为 `Role::Named` 参数（返回 `ReservedAccountName` 错误）。老函数 `derive_duoqian_address_from_sfid_id` 重构为 `derive_institution_address(sfid_id, role)` + `role_from_account_name(account_name)` 翻译辅助。链端字段名 `name`/`SfidNameOf` 同步重命名为 `account_name`/`AccountNameOf`，与 SFID 后端 `MultisigAccount.account_name` 对齐（2026-04-21 第二轮，见 `20260421-name-to-account-name-rename`）。
- `set_sheng_signing_pubkey` 的 origin 必须是当前 SfidMainAccount,通过 `ensure_signed` + `who == main` 校验

## 后端
- 省签名密钥私钥**加密存储在 Postgres admins 表的 encrypted_signing_privkey TEXT 列**,AES-256-GCM 格式 `base64(nonce_12B || ciphertext || tag_16B)`
- Wrap key 由 HKDF-SHA256(SFID MAIN seed, salt, info) 派生,salt = `sfid-sheng-signer-v1-salt`, info = `sfid-sheng-signer-v1-info`
- **SFID MAIN 轮换时必须级联重加密**所有 sheng admin 密文(已在 `set_active_main_signer` 落地),明文 seed 全生命周期 zeroize
- 省签名 signer 的生命周期:省登录管理员登录时 bootstrap(首次生成+推链,复用解密),session 过期驱逐 cache,替换省管理员时级联清链+清密文
- `AdminAuthContext` 必须一路传到业务 extrinsic 提交函数(`submit_register_sfid_institution_extrinsic`),`resolve_business_signer` 根据 ctx.role 路由到本省 Pair
- 业务 extrinsic 的字段包签名和 submit 的 signer 必须是**同一把**省级 Pair,否则链端 verifier 会挂

## 前端
- `ShengAdminsView` 表格必须显示"签名密钥状态"列(Tag),`signing_pubkey` 为 None 显示未初始化,有值显示已激活 + Tooltip 完整 pubkey

## 运维
- 唯一新增的部署要求:**确保 `SFID_SIGNING_SEED_HEX` 环境变量正确**(已有机制),不需要任何新 env
- SFID MAIN 轮换是高风险操作,执行前先备份 PG `admins` 表,失败可恢复
- 后端重启后所有省 signer 清零,需要 43 省各自重新扫码登录才能恢复业务推链
