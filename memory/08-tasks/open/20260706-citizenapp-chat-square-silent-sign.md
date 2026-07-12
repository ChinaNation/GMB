# CitizenApp Chat 与广场静默签名边界

- 状态：已完成并由当前架构继续约束
- 模块：`citizenapp`

## 当前结论

- Chat 会话和设备绑定使用硬件 P-256 设备子钥静默签名，不读取 seed、不调用 CitizenWallet、不弹生物识别。
- Chat 消息本体只使用 OpenMLS 设备密钥，不逐条使用钱包签名。
- 广场发布使用默认热钱包按链上当前最低费用静默签名；服务端仍强制校验有效会员、身份权限、上传内容和链上发布事件。
- 转账、充值、提现、投票、治理、多签和默认钱包切换等用户动权操作继续执行对应授权流程。
- 普通内容按会员档控制；竞选内容只允许竞选公民会员，不保留未落地字段占位或兼容结构。

## 禁止事项

- 禁止恢复 Chat 生物识别、钱包主私钥逐消息签名或冷钱包扫码路径。
- 禁止把客户端静默签名当作广场发布权限真源。
- 禁止保留旧费用常量、旧竞选占位字段或旧授权分支。

当前真源见 `memory/05-modules/citizenapp/chat/CHAT_TECHNICAL.md` 与 `memory/01-architecture/citizenapp/CITIZENAPP_TECHNICAL.md`。
