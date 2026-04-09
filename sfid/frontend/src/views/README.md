# views/ 目录

**铁律**(任务卡 `20260408-sfid-frontend-app-tsx-split` 收官后生效):
以后所有新 sfid 前端业务功能必须放到 `views/<module>/`,严禁直接写进 `components/App.tsx`。
App.tsx 是路由壳子,只负责 Layout / AuthProvider / Menu / 按 activeView switch 渲染各 View。

## 当前实际子目录(拆分后)

- `auth/` — 登录鉴权(LoginView)
- `citizens/` — 注册局 + 业务员绑定/解绑
- `operators/` — 市管理员(ShiAdmin)
- `sheng-admins/` — 省管理员(ShengAdmin),含 system-settings 分支
- `keyring/` — KeyAdmin 密钥轮换 + 主账户余额
- `institutions/` — 私权/公权/公安局机构共享视图
- `common/` — 跨 view 共享小组件(ProvinceGrid/CityGrid)

## 新增功能流程

1. 判断属于哪个 view 模块
2. 编辑该 view 文件,**不动 App.tsx**
3. 如果是新 tab,先在 App.tsx 的 `ActiveView` 类型 + Tab items 数组 + Content switch 各加一行,然后新建 `views/<module>/`

---

## 历史占位说明

任务卡 3 建立的空占位目录,任务卡 4 开始填充 5 个实体的视图:

```
views/
├── common/               — 共享组件(ProvinceGrid / CityGrid / CityDetailShell / 等)
├── citizens/             — 首页公民身份列表
├── public_security/      — 公安局(GFR + ZF + 公民安全局)
├── gov_institutions/     — 公权机构(GFR,非公安局)
├── private_institutions/ — 私权机构(SFR / FFR)
└── registry/             — 注册局(sfid 系统 3 种管理员管理)
```

## 铁律

- 每个子目录下的 `index.tsx` 只负责 router + state 编排,**不超过 150 行**
- 业务 UI 拆到 `components/` 子目录
- 数据拉取用 `hooks/` 子目录的 `useXxxList` / `useXxxCrud`
- 所有 API 调用走 `src/api/` 层,**不能**在 views 内直接 fetch
- 每个文件**硬性 ≤ 300 行**

## 进入 tab 的三角色分流

所有业务 tab(public_security / gov_institutions / private_institutions / registry)的入口
必须用 `useScope()` 派生 VisibleScope,然后按以下流程:

```tsx
const scope = useScope(auth);

if (scope.skipCityList && scope.lockedCity) {
  return <CityDetailShell province={scope.lockedProvince!} city={scope.lockedCity} />;
}
if (scope.skipProvinceList && scope.lockedProvince) {
  return <CityGrid province={scope.lockedProvince} />;
}
return <ProvinceGrid />;
```

## 参考文档

- `feedback_sfid_three_roles_naming.md`
- `feedback_institutions_two_layer.md`
- `feedback_scope_auto_filter.md`
