# NodeUI SFID 服务地址配置

## 目标

NodeUI 需要在多个功能里访问 SFID HTTP API：

- 治理提案创建时读取人口快照
- 清算行添加页搜索具备清算行资格的机构

这些调用必须使用同一套地址规则，避免本地开发节点误连正式 SFID，
也避免正式节点误连本地测试数据。

## 配置优先级

实现位置：[citizenchain/node/src/ui/sfid_config.rs](../../../../../citizenchain/node/src/ui/sfid_config.rs)

优先级从高到低：

1. `SFID_BASE_URL` 环境变量
2. debug 构建默认值：`http://127.0.0.1:8899`
3. release 构建默认值：`http://147.224.14.117:8899`

`sfid_base_url()` 会清理末尾 `/`，调用方统一拼接 `/api/...` 路径。

## 本地开发

本地启动 SFID 后端后，NodeUI debug 构建默认访问：

```text
http://127.0.0.1:8899
```

如果需要用手机或其他设备访问本机 SFID，应单独使用当前局域网 IP。
局域网 IP 会随网络变化，不作为 NodeUI 本机开发默认值。

## 正式节点

正式 release 构建默认访问：

```text
http://147.224.14.117:8899
```

如果未来正式 SFID 服务迁移，只需要在节点启动环境中设置：

```bash
SFID_BASE_URL=http://新的-sfid-服务地址:8899
```

## 当前调用方

| 调用方 | 用途 |
|---|---|
| `ui/governance/sfid_api.rs` | 公民投票人口快照 |
| `ui/clearing_bank/sfid_proxy.rs` | 清算行资格候选搜索 |
