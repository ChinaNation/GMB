# SFID Step 2d:wumin / wuminapp 凭证 decoder 双端联动

- 状态:open
- 创建日期:2026-05-02
- 模块:`wumin/`(Flutter 冷钱包)+ `wuminapp/`(Flutter 在线端)
- 关联 ADR:`memory/04-decisions/ADR-008-sheng-admin-3tier-and-key-admin-removal.md`
- 前置依赖:step2a + step2b 完成(链端凭证字段已固定)
- 跨模块协议触发:**`memory/07-ai/chat-protocol.md` 第 5 条**(runtime 凭证签名/验签改动必须双端联动)

## 任务需求

step2b 在 `duoqian-manage` 凭证 payload 加 `signer_admin_pubkey: [u8;32]` 字段。wumin 冷钱包 + wuminapp 在线端的扫码签名 decoder **必须同步识别 + 显示**新字段:
- `feedback_qr_signing_two_color.md` 铁律:禁止白盲签;decoder 必须能解出全部字段并展示给用户
- 不识别新字段会导致两色识别从绿色降级为黄色(盲签),被 UX 拒绝

## 影响范围

### wumin 冷钱包(Flutter + Dart)

- decoder:`lib/...` 下 institution registration 凭证解码逻辑
- 加字段 `signer_admin_pubkey` 解码 + 显示("签名管理员公钥:0x...")
- sign_display_fields 加新字段输出
- analyze + test 全绿

### wuminapp 在线端(Flutter + Dart)

- decoder:同步加字段
- sign_display_fields 加新字段
- analyze + test 全绿

### 凭证 payload 兼容性

SCALE 编码字段顺序硬编码,前端冷钱包/在线端/SFID 后端必须**完全对齐**:
```
sfid_id || institution_name || a3 || sub_type || parent_sfid_id ||
province || register_nonce || signer_admin_pubkey || signature
```

新字段插入位置:`register_nonce` 后、`signature` 前。

## 主要风险点

- **签名 payload 哈希**:加字段后 hash 输入变化;decoder 必须用与 SFID 后端 + duoqian-manage runtime 完全相同的序列化
- **未识别字段提示降级**:wumin 两色识别铁律严格,decoder 不识别会显黄盲签 → UX 阻塞;必须 PR 同步上线
- **测试覆盖**:wumin/wuminapp 已有 institution registration scan/sign decode 测试,本卡更新现有 + 新加 ≥ 2 测试覆盖新字段

## 是否需要先沟通

- 否(`chat-protocol.md` 第 5 条已强制双端联动,本卡作为跨模块任务卡注册即合规)

## 验收清单

- `cd wumin && flutter analyze && flutter test` 全绿
- `cd wuminapp && flutter analyze && flutter test` 全绿
- 现有 institution registration decode 测试更新通过
- 新加 ≥ 2 测试:
  - `decode_with_signer_admin_pubkey_success`
  - `decode_without_signer_admin_pubkey_legacy_payload_rejected`
- 双端 signing payload 字节序列与 runtime + SFID 后端完全一致(可加 fixture 测试)
- Grep 残留 0:`signing_province=None` 分支在 decoder
- 任务卡 progress 章节回写

## 不要做的事

- 不要碰 runtime / SFID 后端(已在 step2a/b/c 完成)
- 不要 commit

## 工作量

~300 行(双端 ~150 行/端)+ 4 测试,~1.5 agent rounds。

## 提交策略

wumin 与 wuminapp 必须**同一 PR 内提交**或**严格按"先 wumin → 再 wuminapp"顺序合并**,确保任何中间快照都不出现 decoder 不一致状态。
