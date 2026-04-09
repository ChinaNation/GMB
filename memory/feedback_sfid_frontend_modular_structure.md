# sfid-frontend 模块化目录铁律

任务卡 `20260408-sfid-frontend-app-tsx-split` 拆分后的强制约定:

- `components/App.tsx` 仅作为路由壳子,行数应保持在 ≤ 400 行
- 所有业务功能必须放在 `views/<module>/<ModuleView>.tsx`
- 跨 view 共享逻辑放 `hooks/` `contexts/` `utils/`
- 跨 view 共享组件放 `components/`(如 ScanAccountModal)
- 严禁在 App.tsx 里新加 useState / handler / 业务 JSX —— 一律下沉到对应 view

## 目录结构

```
src/views/
├── auth/           登录鉴权
├── citizens/       注册局 + 业务员绑定/解绑
├── operators/      市管理员 ShiAdmin
├── sheng-admins/   省管理员 ShengAdmin + system-settings
├── keyring/        KeyAdmin 密钥轮换
├── institutions/   私权/公权/公安局机构
└── common/         跨 view 共享小组件
```

## 新增功能流程

1. 判断属于哪个 view(auth/citizens/operators/sheng-admins/keyring/institutions)
2. 编辑该 view 文件,不动 App.tsx
3. 如果是新 tab,先在 App.tsx 的 activeView 类型 + Menu items + Content switch 各加一行,然后新建 `views/<module>/`

## 拆分成果

- App.tsx: 3431 行 → 357 行
- 所有业务 state/handler/JSX 全部下沉到 views/
- tsc + build 全绿

## 参考

- `sfid/frontend/src/views/README.md`
- 任务卡:`memory/08-tasks/done/20260408-sfid-frontend-app-tsx-split.md`
