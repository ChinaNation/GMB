# OnChina 完成定义

OnChina 任务完成必须同时满足：

- 没有恢复旧独立身份系统、旧 registry 目录、`backend/src`、`frontend/api` 或 `frontend/chain`。
- 后端、前端、文档、协议登记中的路径都指向 `citizenchain/onchina/`。
- 扫码、签名、验签和交易载荷只使用 `QR_V1` 与统一协议登记。
- 页面展示字段左侧分类名为中文，账户类字段展示 SS58 地址。
- 改代码后同步更新 `memory/01-architecture/onchina/` 或 `memory/05-modules/citizenchain/onchina/`。
- 涉及真实接口、数据库、登录、权限、扫码或页面展示时，必须完成真实运行态验收。
