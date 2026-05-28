# SFID 档案码驱动公民身份绑定与 wuminapp 电子护照状态机

- 状态: done
- 日期: 2026-05-27
- 模块: SFID 后端 / SFID 前端 / wuminapp / 文档

## 需求

SFID 公民身份绑定改为从 CPMS 档案码发起，不再接受 wuminapp 侧主动创建绑定记录。
wuminapp 电子护照页按 `unset / pending / bound` 三态显示按钮，并用已选择钱包完成扫码签名。

## 范围

- SFID 新增/更换统一走档案码扫码或上传。
- SFID 签名 challenge 锁定档案码中的 `wallet_address / wallet_pubkey / wallet_sig_alg`。
- SFID 扫描 wuminapp `sign_response` 后验签并生成/更新公民身份记录。
- wuminapp 查询 SFID 绑定状态，绑定成功后显示已绑定。
- 删除旧的空钱包账户注册语义。

## 验证

执行完成后补充。
