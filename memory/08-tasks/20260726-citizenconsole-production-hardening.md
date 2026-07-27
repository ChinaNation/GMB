# CitizenConsole 生产安全加固

> 状态：代码整改、测试、文档和旧运行物清理完成；Developer ID 正式签名与公证待外部证书
>
> 用户已确认本任务方案，以及任务卡、原生安全程序拆分源码和生产安全测试文件的新增。

## 任务目标

1. 修复 CitizenApp 用户资料页返回图标测试残留，恢复全量测试通过。
2. 删除 CitizenConsole 可由任意同用户进程调用的通用密钥命令行接口。
3. 将原生安全程序、Node 服务、网页、动作脚本与充值发币代码纳入正式签名和完整性边界。
4. 所有密钥操作只允许固定操作目录并通过真实 Touch ID，禁止电脑密码回退。
5. 充值发币保持页面生命周期内持续解锁，但发币私钥不进入普通 UI Node 进程。
6. 补齐 HTTP 来源校验、安全响应头、子进程环境白名单和生产签名启动守卫。

## 硬边界

- 不修改 `citizenchain/runtime/`。
- 不修改平台订阅、创作者订阅、续费、取消订阅或换档逻辑。
- 不读取、输出、删除或改写任何现有生产密钥值。
- 不执行真实购买、真实支付或真实发币。
- 不部署或修改 Cloudflare 生产资源。
- 不执行 GitHub push、PR或远端 CI。
- CitizenConsole 整目录继续由 Git 忽略，不提交或推送其中源码和运行产物。
- 正式签名缺少 Developer ID 或公证能力时必须失败关闭，禁止把 Apple Development
  签名当作生产验收完成。

## 预计修改目录

- `citizenapp/test/8964/`
  - 修改现有测试预期；只涉及测试代码，不改变已经设计好的 UI。
- `citizenconsole/`
  - 重构启动、私有通信、密钥操作、生产动作、充值发币隔离、HTTP 防护和残留清理；
    本机私有代码，整目录由 Git 忽略。
- `citizenconsole/security-app/`
  - 拆分原生安全程序并收紧 Xcode Release 签名、Hardened Runtime 与启动校验。
- `citizenconsole/test/`
  - 增加生产签名、私有通道、来源校验和密钥隔离回归测试。
- `citizenconsole/.runtime/`
  - 替换旧开发签名生成物；仅本机运行产物，Git 忽略。
- `memory/03-security/`
  - 更新 CitizenConsole 正式签名、生物识别、代码完整性和密钥使用规则。
- `memory/01-architecture/gmb/`
  - 更新 CitizenConsole 原生根进程、私有通信和充值发币隔离架构。
- `memory/08-tasks/`
  - 记录方案、执行过程、真实验收证据与旧安全任务的后续整改关系。

## 执行清单

- [x] 修复 CitizenApp 测试并完成全量分析和测试。
- [x] 建立固定原生操作目录和 Keychain 封装。
- [x] 建立仅父子进程可用的匿名私有通信通道。
- [x] 删除通用密钥 CLI 与任意 Touch ID 提示入口。
- [x] 将 Node、网页、动作和充值代码纳入签名密封资源。
- [x] 隔离充值发币密钥生命周期，保留无超时页面会话。
- [x] 收紧子进程环境、Origin、Fetch Metadata 与安全响应头。
- [ ] 完成 Developer ID 正式签名、公证、真实 Touch ID 和篡改拒绝验收。
- [x] 更新文档、中文注释并清理旧实现与生成物。

## 执行记录

- CitizenApp 用户资料页测试与已确认的 `chevron_left` UI 对齐；`flutter analyze` 无问题，
  全量测试 813 项通过、5 项跳过、0 项失败。
- 通用 `touchid.swift` CLI 已删除。Swift 原生根进程只接受固定操作目录，通过匿名
  `AF_UNIX socketpair` 与签名包内 Node 通信；缺少私有 fd 时 Node 直接拒绝启动。
- 原生操作目录固定 Keychain 环境、名称和 Touch ID 文案；发币私钥禁止经普通
  `secret.read` 返回 Node。充值解锁后私钥由原生根进程持有，每笔发币只通过匿名管道
  交给签名包内一次性工作进程，锁定、离页、断连或进程退出时清零。
- Xcode Release 已固定 Developer ID Application、Hardened Runtime、无调试授权；
  Node、网页、动作、充值代码和依赖均进入签名资源。原生程序内部与 `start.sh` 双重校验
  Developer ID OID、Team、Bundle、嵌套资源、Hardened Runtime、`get-task-allow` 和公证。
- 修改请求已强制 Origin + Fetch Metadata；响应增加 CSP、禁止嵌入、`nosniff`、
  Referrer Policy、Permissions Policy、COOP 和 CORP；子进程环境改为明确白名单。
- CitizenConsole `npm run check`、37 项测试和生产依赖审计全部通过，依赖漏洞为 0。
  未签名隔离构建成功，并真实验证锁屏页面、安全响应头及未解锁状态接口 403；加入正式
  签名内部守卫后，未签名构建不再允许运行。
- 已彻底删除旧 Apple Development 应用、旧直接启动 Node 的 socket launcher、旧通用
  CLI、未签名验收包、Swift 临时模块和构建输入。生产日志、Cloudflared、WASM 缓存及
  业务状态均保留。

## 当前唯一阻塞

本机 `security find-identity -p codesigning` 只有 `Apple Development`，没有
`Developer ID Application`；`start.sh build-production` 已真实验证会失败关闭，现有
CitizenConsole 也因无正式应用而拒绝启动。取得 Developer ID 证书并配置
`CitizenConsoleNotary` 公证 Keychain profile 后，执行：

```bash
bash citizenconsole/start.sh build-production
```

脚本才会构建、签名、公证、staple、复验并恢复正式 CitizenConsole。没有该外部证书前，
本任务不得标记为生产签名验收完成。
