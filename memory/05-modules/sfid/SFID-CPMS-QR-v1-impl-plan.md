# SFID_CPMS_V1 两码方案实施记录

- 状态:SFID 与 CPMS 两侧已按精简协议同步实施
- 当前协议真源:`memory/05-modules/sfid/SFID-CPMS-QR-v1.md`
- 当前任务卡:`memory/08-tasks/open/20260516-sfid-cpms-install-archive.md`

## 目标

SFID 与 CPMS 两侧完成以下能力:

1. 为每个市公安局 CPMS 签发唯一安装授权。
2. INSTALL 安装码只携带 `sfid_number / province_name / city_name / install_secret / sig`。
3. SFID 能验证 CPMS 生成的 ARCHIVE 档案码。
4. SFID 能把档案号归入省、市和公安局机构 `sfid_number`。
5. 其他 CPMS 或普通扫码方不能从档案号明文推断签发城市或机构。

## 已落地目录

| 路径 | 用途 |
|---|---|
| `sfid/backend/cpms/model.rs` | CPMS 授权模型、INSTALL/ARCHIVE DTO、验真结果 |
| `sfid/backend/cpms/handler.rs` | INSTALL 签发、ARCHIVE 验真、档案导入、授权状态治理 |
| `sfid/backend/cpms/mod.rs` | CPMS 模块导出 |
| `sfid/backend/citizens/model.rs` | 公民档案省市归属字段 |
| `sfid/backend/citizens/binding.rs` | 公民绑定复用 CPMS ARCHIVE 验真入口 |
| `sfid/frontend/cpms/` | CPMS 授权前端 API 与面板 |
| `sfid/frontend/institutions/InstitutionDetailPage.tsx` | 公安局详情页生成和展示 INSTALL 安装码 |

## 后端要点

- `CpmsSiteKeys` 保存 `sfid_number / install_secret / install_secret_hash / cpms_pubkey_hash` 等授权验真必要字段。
- `GenerateCpmsInstallOutput` 对外返回 `sfid_number / qr1_payload`。
- `CpmsArchiveQrPayload` 对外只接受 `proto / type / ano / cs / ve / cpms_pubkey / geo_seal / sig`。
- `verify_cpms_archive_qr` 是 SFID 侧唯一 ARCHIVE 验真入口。
- `archive_import` 和 `citizen_bind(bind_archive)` 共用同一验真入口。
- `ImportedArchive` 保存 `province_code / city_code / sfid_number / cpms_pubkey_hash / geo_seal_hash`。

## 前端要点

- 公安局机构详情页只提供“生成 CPMS 安装二维码”。
- CPMS 面板只展示 INSTALL 安装码和授权状态。
- ACTIVE 后不再展示安装码，只展示签发公钥是否已绑定。
- REVOKED 可重新签发 INSTALL。

## 验收

```text
cd sfid/backend && cargo fmt && cargo check && cargo test
cd sfid/frontend && npm run build
rg "<旧协议字段关键词>" sfid cpms memory
```
