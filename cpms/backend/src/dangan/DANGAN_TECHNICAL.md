# CPMS Dangan 模块技术文档

## 1. 模块定位
`backend/src/dangan/` 负责档案号生成与档案二维码构建相关能力。

该模块是档案数据标准化与二维码签发的核心算法模块。

## 2. 负责范围
- 省市代码校验（`province_codes`）
- 档案号生成与冲突重试
- 档案号校验位算法
- 档案业务二维码载荷构建与签名
- 机构公钥登记二维码载荷构建与签名
- 公民状态（`citizen_status`）合法性校验

## 3. 核心接口
- `generate_archive_no_with_retry(...)`
- `build_qr_payload(...)`
- `build_site_key_registration_payload(...)`
- `validate_citizen_status(...)`

## 4. 关键数据结构
- `QrPayload`
- `SiteKeyRegistrationPayload`
- `SiteKeyPublicItem`

## 5. 安全与一致性规则
- 档案号冲突时按 nonce 递增重试，避免重复档案号
- 校验位使用 `cpms-archive-v3` 固定串 + BLAKE2b 字节和 mod 10
- 二维码签名统一使用 `sr25519`
- 机构公钥登记二维码签名串采用固定字段顺序，避免跨系统验签串不一致
- 公民状态仅允许 `NORMAL` / `ABNORMAL`

## 6. 模块边界
- 本模块只提供档案号与二维码算法，不承载登录与权限逻辑
- 由 `operator_admin` 与 `super_admin` 模块调用本模块能力
- 主程序 `main.rs` 仅做模块装配与通用底座
