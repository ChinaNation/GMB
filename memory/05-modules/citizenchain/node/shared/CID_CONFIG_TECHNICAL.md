# 节点桌面端 OnChina 服务地址配置

## 目标

节点桌面端需要在多个功能里访问 OnChina HTTP API：

- 机构注册等 OnChina 服务调用
- 清算行添加页搜索具备清算行资格的机构

这些调用必须使用同一套地址规则，默认固定到局域网统一入口。

## 配置优先级

实现位置：[citizenchain/node/src/shared/cid_config.rs](../../../../../citizenchain/node/src/shared/cid_config.rs)

优先级从高到低：

1. `ONCHINA_BASE_URL` 环境变量
2. 默认值：`https://onchina.local:8964`

`onchina_base_url()` 会清理末尾 `/`，调用方统一拼接 `/api/...` 路径。

## 本地开发

本地启动 OnChina 后端后，节点桌面端默认访问：

```text
https://onchina.local:8964
```

OnChina 自签 HTTPS 证书由节点软件生成并持久化；节点桌面端只对固定本机入口放宽自签证书校验。

## 正式节点

正式 release 构建默认访问同一固定入口：

```text
https://onchina.local:8964
```

如果未来正式 OnChina 服务迁移，只需要在节点启动环境中设置：

```bash
ONCHINA_BASE_URL=https://新的-onchina-服务地址:8964
```

## 当前调用方

| 调用方 | 用途 |
|---|---|
| `transaction/offchain_transaction/institution_read/` | 清算行机构只读(B0:机构注册凭证已下沉 onchina;node 仅链上直读机构身份) |
| `offchain/cid.rs` | 清算行资格候选搜索 |
