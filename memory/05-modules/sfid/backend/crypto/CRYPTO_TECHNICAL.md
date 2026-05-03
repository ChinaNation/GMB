# CRYPTO 模块技术文档

- 最后更新:2026-05-02
- 任务卡:
  - `memory/08-tasks/done/20260502-sfid-models-scope边界整改.md`

## 1. 模块定位

- 路径:`sfid/backend/crypto`
- 职责:承载与具体业务无关的低层加密、公钥规范化和密钥派生工具。

## 2. 当前结构

```text
sfid/backend/crypto/
├── mod.rs       # crypto 模块入口
├── sr25519.rs   # sr25519 seed -> keypair / pubkey hex 派生
└── pubkey.rs    # sr25519 pubkey 规范化与等值比较
```

## 3. 边界

- `crypto/pubkey.rs` 是跨业务工具,不属于 `scope`。
- 业务模块只调用 crypto 工具,不得在 crypto 内读取 Store、检查角色或执行 HTTP handler。
- 公钥展示仍由前端/业务模块决定;crypto 只做解析、规范化和比较。
