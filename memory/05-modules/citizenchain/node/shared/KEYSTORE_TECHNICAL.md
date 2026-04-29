# Keystore 通用操作模块 — 技术文档

## 概述

`shared/keystore.rs` 提供 Substrate 节点 keystore 文件系统操作的通用抽象，
避免各密钥模块（GRANDPA、Bootnode）重复实现目录扫描、密钥写入和清理逻辑。

## 目录布局

```
<app_data>/node-data/               # node_data_dir()
└── chains/
    ├── citizenchain/               # 默认链 ID
    │   └── keystore/
    │       └── 6772616e<pubkey>     # GRANDPA 密钥文件（key_type_prefix + pubkey_hex）
    └── <other-chain-id>/
        └── keystore/
```

## API

| 函数 | 说明 |
|------|------|
| `node_data_dir(app)` | 返回节点数据根目录，自动创建 |
| `keystore_dirs(app)` | 扫描所有链的 keystore 目录列表 |
| `keystore_filename(prefix, pubkey)` | 生成 keystore 文件名 |
| `scan_keystore_files(dirs, prefix)` | 扫描匹配前缀的密钥文件 |
| `write_key_to_keystore(dirs, prefix, pubkey, content)` | 写入密钥并清理旧密钥 |
| `remove_other_keys(dirs, prefix, keep)` | 移除同类型旧密钥 |
| `has_key_in_keystore(dirs, prefix, pubkey)` | 检查密钥是否存在 |

## 安全特性

- Unix 下目录创建/打开使用 `mkdirat/openat(O_NOFOLLOW | O_DIRECTORY)` 逐级完成，避免“先检查再使用”的符号链接 TOCTOU
- `node-data`、`chains`、`<chain-id>`、`keystore` 目录都会显式收口到 Unix `0700` 权限，不依赖进程 `umask`
- 密钥文件在已打开的 keystore 目录句柄内以“临时文件 -> fsync -> renameat”原子写入，并显式收口到 Unix `0600` 权限
- 扫描、存在性检查、删除旧 key 都基于已打开目录句柄和 `fstatat/unlinkat` 完成，符号链接与非常规文件会被跳过
- 写入后自动清理同类型旧密钥，避免节点加载多把 authority key

## 调用方

- `settings/grandpa-address/mod.rs`：GRANDPA 投票密钥管理
- `home/process/mod.rs`：节点启动时获取数据目录路径
- `mining/dashboard/mod.rs`：挖矿面板获取数据目录
- `settings/fee-address/mod.rs`：手续费地址 keystore 操作
