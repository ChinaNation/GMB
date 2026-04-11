# SFID 省级签名密钥铁律

任务卡 `20260409-sfid-sheng-admin-per-province-keyring` 完成后的强制约定:

## 链端
- 链端新增 `sfid_code_auth::ShengSigningPubkey` 和 `ProvinceBySigningPubkey` 两张 storage map,永远保持对偶关系(通过 `set_sheng_signing_pubkey` extrinsic 原子维护)
- **业务 extrinsic 的 verifier(register_sfid_institution 等)要求 origin pubkey 属于某省 signing key**,`signing_province = None` 时回退到 SFID MAIN 验签(向后兼容模式)
- 所有 SFID 系统与链端之间的签名 payload 统一使用 `b"GMB_SFID_V1"` 作为第一字段,不再有 V2/V3 版本(4 个 verifier 已全部统一)
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
- 机构注册 / 账户注册 Modal 的 catch 分支必须识别"本省登录管理员未在线"和"密钥管理员不能直接推送"两种 503,翻译成友好中文

## 运维
- 唯一新增的部署要求:**确保 `SFID_SIGNING_SEED_HEX` 环境变量正确**(已有机制),不需要任何新 env
- SFID MAIN 轮换是高风险操作,执行前先备份 PG `admins` 表,失败可恢复
- 后端重启后所有省 signer 清零,需要 43 省各自重新扫码登录才能恢复业务推链
