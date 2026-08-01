# CitizenApp Cloudflare D1 schema 收敛为唯一基线

状态：open（2026-07-31，唯一基线已建立；部署须按当前 schema 执行）

## 最终口径

- D1 schema 唯一真源是 `citizenapp/cloudflare/schema/citizenapp.sql`。
- Worker 不保存任何用户私有数据密钥，不提供密钥领取或恢复接口。
- 私有数据密钥只由 CitizenApp 从 CID 当前链上绑定钱包账户 child 本地派生，因此 D1
  不需要任何用户密钥表。
- D1 继续保存 CID 归属的业务密文、会话索引、设备公钥、社交关系和业务镜像。

## 已完成改动

1. D1 目录收敛为单一 `schema/citizenapp.sql`，删除开发期迁移链和空目录。
2. `wrangler.toml` 删除 D1 `migrations_dir`；Durable Objects 的 `[[migrations]]` 保持不变。
3. `cloudflare.sh` 的重建入口执行唯一基线，并按基线声明表与远端实表做完整性对比。
4. `package.json` 的本地/生产数据库命令统一指向唯一基线。
5. 安全测试断言 schema 目录只有一个文件、版本存在、表名不重复、部署完整性门禁存在。
6. 当前基线已删除所有服务端用户私有数据密钥相关列、环境绑定和业务路由。

## 创世版本约定

文件名固定 `citizenapp.sql`，创世初始版本固定写为 `SCHEMA VERSION: v1.0.0`；文件名不带
版本，避免脚本、测试与文档路径漂移。

- **创世冻结前（零用户）**：直接修改唯一基线并清空重建；不累计版本号、不追加版本日志、
  不写迁移、不做兼容。
- **上线冻结后**：本文件转只读，真实结构变更另走增量迁移；实现前必须单独出方案确认。

## 部署边界

- 本卡不授权远端 D1 重建、Worker 部署、GitHub 推送或 CI。
- 生产部署必须由本机 `citizenconsole/` 经 Touch ID 执行，并先通过类型检查、测试和远端
  表完整性验证。
- 当前私有数据密钥模型不依赖 D1 清空或重建；任何 D1 destructive 操作仍需单独确认。
