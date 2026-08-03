# CitizenConsole 生产密钥统一治理

## 任务需求

将 CitizenConsole 固化为全部生产密钥的唯一管理入口，彻底删除测试密钥、测试远端资源、
旧 Keychain 残留和废弃远端 Secret；现有且仍有用的生产密钥迁入真正受 Touch ID 保护的
Data Protection Keychain，远端运行所需 Secret 由 CitizenConsole 统一写入、轮换、检查和
删除，不在仓库、日志、浏览器、命令行参数或本机明文文件中保存。

## 用户确认边界

- 所有 staging/test 密钥和远端测试资源全部彻底删除，不保留测试环境。
- 国储会节点不连接、不部署、不修改远端；旧 CitizenConsole/Keychain 中的国储会节点配置删除。
- 只配齐以下五个生产节点：
  - 中枢省
  - 河南省
  - 河北省
  - 山东省
  - 山西省
- 其余节点暂时保持空配置；旧 Keychain 中不属于上述五个节点的节点配置全部删除。
- iOS、macOS、Windows 三端当前已经存在的正式签名材料全部配置好；当前没有的材料只登记
  缺失状态，后期取得正式账号或证书后再补充，不生成临时或测试签名材料。
- 用户已确认创建本任务卡。

## 已核实的当前事实

- `GMB CitizenConsole` Data Protection Keychain 已保存全部现有生产配置；读取、写入、
  删除和使用均由 Apple 签名安全代理逐次要求 Touch ID。
- 旧 `GMB Deploy` 登录 Keychain 服务已删除；一次性迁移入口已从安全代理源码和最终签名
  二进制中移除，不保留旧服务读取或兼容分支。
- 中枢省、河南省、河北省、山东省、山西省五个目标节点的 `SERVER_IP`、
  `BOOTNODE_KEY`、`VALIDATOR_KEY`、`SSH_KEY` 四项齐全并通过公开身份派生校验；其他节点
  和旧国储会本机配置为空。
- GitHub Actions 只保留真实 workflow 使用的 `GMB_APP_KEY`、`GMB_TOP_KEY` 和
  `GMB_TOP_PUBKEY`。
- Cloudflare 只保留 production Worker、D1、KV、两个 R2 桶、Queue、Route 和 16 项
  Worker Secret；远端测试资源已经删除。
- 本任务不得读取、记录或输出任何密钥明文。

## 技术方案

### 第一步：全量只读清点

- 清点新旧 Keychain 已知账户的存在性，不读取值。
- 清点 GitHub Actions Secrets、Variables、Environments 与 workflow 真实引用。
- 清点 Cloudflare staging/production Worker、Secret、D1、KV、R2、Queue、Route、Access、
  Tunnel 及关联测试资源。
- 清点 CitizenConsole 当前密钥目录和每个部署动作的真实依赖。
- 形成逐项 `保留 / 迁移 / 轮换 / 删除 / 缺失 / 外部后补` 清单。

### 第二步：生产密钥收敛

- 修复一次性旧 Keychain 迁移：旧登录 Keychain 与新 Data Protection Keychain 分区读取，
  一次 Touch ID 完成快照、写入、逐项读回验证；同名新旧值不一致时拒绝覆盖。
- 只迁移生产充值配置、production Cloudflare配置、GitHub本机生产 SSH 和五个目标节点配置。
- 旧 Keychain 曾不受真正生物识别保护的可轮换生产令牌，先创建替代值、同步远端、真实验收，
  再吊销旧值。
- CitizenConsole 只显示配置状态；读取、写入、使用和删除继续强制真实 Touch ID。

### 第三步：生产补齐与真实验收

- 配齐充值发币全部九项，并验证私钥派生发币地址、Worker、Base RPC、代币白名单、节点 WS
  和收款地址的一致性；不执行真实购买或真实发币。
- 配齐 production Worker 必需 Secret，验证 Worker 健康、充值订单读取和结算鉴权。
- 配齐中枢省、河南省、河北省、山东省、山西省五个节点的四项配置并逐项派生校验；除非用户
  另行授权，本任务不部署节点。
- 盘点三端现有正式发布材料并写入正确管理位置；没有的保持明确缺失，不建立测试替代物。

### 第四步：彻底删除测试与旧残留

- 删除全部 staging/test 本机 Keychain 项、GitHub测试 Secret/Variable/Environment和
  Cloudflare测试 Secret及测试资源。
- 删除旧 `GMB Deploy` 服务中的全部项目，包括国储会和非目标节点配置。
- 删除仓库、本机运行目录和动作脚本中的旧测试入口、旧环境名、旧注释、旧文案和临时残留。
- 删除前再次按精确目标清单核对，禁止使用宽泛目录、通配符或未解析变量作为删除目标。

### 第五步：复验与收口

- 真实打开 CitizenConsole，验证 Touch ID 打开、生产状态页、充值发币长会话锁定和五节点状态。
- 复查新安全区、旧 Keychain、GitHub和Cloudflare，确认只剩生产目标项。
- 运行 CitizenConsole检查与测试，更新安全和架构文档，完善中文注释并清理残留。
- 不执行 GitHub push、PR或远端 CI；如后续需要推送，必须再次取得当前任务的单独明确授权。

## 预计修改目录

- `/Users/rhett/GMB/citizenconsole/`
  - 修复安全迁移、生产密钥目录、远端生命周期动作和状态展示；涉及本机私有代码、中文注释、
    测试与旧测试入口/残留清理，整目录继续由 Git 忽略。
- `/Users/rhett/GMB/memory/08-tasks/open/`
  - 维护本任务卡、清单、执行记录和验收证据；涉及文档，不记录任何密钥值。
- `/Users/rhett/GMB/memory/03-security/`
  - 更新生产密钥唯一控制面、Touch ID、远端 Secret和轮换规则；只修改现有文档。
- `/Users/rhett/GMB/memory/01-architecture/`
  - 更新 CitizenConsole生产密钥控制面和环境边界；只修改现有文档。
- `/Users/rhett/GMB/.github/workflows/`
  - 仅在真实审计确认 workflow仍引用废弃测试或旧 Secret名称时修改；涉及代码/配置和残留清理，
    不新建 workflow。
- `/Users/rhett/GMB/citizenapp/cloudflare/`
  - 删除 staging/test环境配置和旧测试入口，收敛 production契约；涉及代码、配置、测试和
    残留清理，不写入密钥。

## 禁止事项

- 不修改 `citizenchain/runtime/`。
- 不读取、打印、记录或提交密钥明文。
- 不触碰国储会远端节点。
- 不为缺失的正式发行证书生成测试替代物。
- 不在生产替代凭据尚未验证前删除仍在使用的旧凭据。
- 不执行未经单独授权的 GitHub推送、PR、远端 workflow触发或节点部署。

## 执行状态

- [x] 需求和破坏性范围确认
- [x] 任务卡创建
- [x] 第一步：全量只读清点
- [x] 第二步：生产密钥收敛
- [x] 第三步：生产补齐与真实验收
- [x] 第四步：彻底删除测试与旧残留
- [x] 第五步：复验与收口

## 执行记录

- 2026-08-02：GitHub“推送CI”在 Touch ID 成功后、进入推送脚本前被 SSH 私钥结构校验拦截。
  根因是 `github:SSH_KEY` 仍保留旧 `base64:` 单行外层封装；用户确认不增加新按钮，改为
  现有推送读取链路在同一次 Touch ID 上下文中自动解码、校验并原子覆盖，随后继续原推送流程。
  同时补齐写入前格式门禁；修复本身不得打印、回显或落盘私钥，不主动推送或触发 CI。
- 2026-08-02：自动正规化与永久写入门禁已实现并换入正式签名 CitizenConsole；源码检查、
  Swift 类型检查和本项新增安全回归测试通过，签名、Hardened Runtime 与密封资源复验通过。
  全套 50 项测试为 49 通过、1 项既有 D1 schema 版本注释断言失败，与本次 GitHub SSH 修复
  无代码交集。当前等待用户在原“推送CI”入口完成 Touch ID 和真实 GitHub SSH 验证；成功后
  必须删除一次性 `base64:` 正规化路径，仅保留正确格式写入门禁。

- 2026-07-26：用户确认所有测试密钥与测试远端资源彻底删除；只配齐中枢省、河南省、河北省、
  山东省、山西省五个节点；删除旧国储会本机配置；三端只配置当前已有正式材料。
- 2026-07-26：修复旧登录 Keychain 查询边界，一次 Touch ID 原子迁移 39 项生产配置后删除
  整个旧服务；随后立即删除迁移代码并重新构建、签名、验证安全代理。
- 2026-07-26：充值发币九项配置、production 结算令牌、Base 主网 RPC、Worker 公开配置、
  正式链创世哈希均交叉一致；发币私钥可派生规范 AccountId，未执行真实购买或发币。
- 2026-07-26：五个目标节点的 IP、PeerId、GRANDPA 公钥和 SSH 私钥逐项验收通过；迁移时
  发现 SSH 私钥存在旧 `base64:` 外层封装，已在双次 Touch ID 下原子去除且未改变私钥内容。
- 2026-07-26：彻底删除 Cloudflare 测试路由、队列 consumer、Worker、Queue、两个 Access
  应用、D1、KV 和两个空 R2 桶；复验所有测试资源均不存在，production 资源完整保留。
- 2026-07-26：删除 GitHub 未被 workflow 使用的旧 Secret，只保留三个生产签名 Secret。
- 2026-07-26：CitizenApp Cloudflare 配置收敛为单一 production，部署 production Worker
  并通过真实健康接口；远端 16 项 Worker Secret 与本机目录契约完全一致。
- 2026-07-26：Worker 29 个测试文件、178 项测试全部通过；CitizenConsole 31 项检查和
  测试全部通过。`CF_DEPLOY_TOKEN` 覆盖 Turnstile，`CF_ZT_TOKEN` 覆盖 Access 应用和
  Tunnel；随后已在 Cloudflare 控制台为 `CF_ZT_TOKEN` 补齐 Access Service Tokens
  Read/Write 最小权限，并通过真实 API 复验，不再存在该权限缺口。
- 2026-07-26：真实 CitizenConsole 运行态验收通过：未解锁首页只返回锁屏且状态接口
  403；Touch ID 后状态与节点接口均返回 200，Cloudflare production 配置完整且仅五个
  指定节点为就绪。`CitizenConsole · 充值发币` 独立 Touch ID 长会话锁定前为已解锁，
  点击锁定后立即变为未解锁，过程中未运行结算、未购买、未支付、未发币。
- 2026-07-26：在 Cloudflare 用户令牌控制台为 `CF_ZT_TOKEN` 补充账户级
  `访问：服务令牌 / 编辑` 最小权限，真实 API 返回 200。发现正式 Access 策略同时引用
  两枚 Service Token 后，以当前 `CHAIN_ID` 为真源保留正在使用的较新凭据：账户级可复用
  策略缩减为只引用当前令牌，删除未被 Worker 使用的更旧重复令牌，并把当前令牌统一命名为
  `CitizenChain`。凭据值未轮换；策略修改前后正式创世 RPC 和 production Worker 健康
  检查均通过。远端最终只剩一个 `CitizenChain` Service Token。
- 2026-07-26：最终远端复查发现两个已经失去应用引用的 staging Access 可复用策略仍残留，
  已按精确名称删除。复验 Cloudflare 只剩正式 `CitizenChain` Access 应用、同名可复用
  策略和唯一同名 Service Token，策略只引用该令牌；Tunnel 为 healthy，Worker Secret
  精确为 16 项，production D1、KV、两个 R2 桶、Queue 和 Route 均与仓库契约一致，全部
  远端资源名称中不再包含 staging/testnet/test 残留。
- 2026-07-26：最终再次运行 CitizenConsole 检查与 31 项测试、CitizenApp Worker 类型检查、
  29 个测试文件 178 项测试及生成类型一致性检查，全部通过；正式链创世 RPC 和 Worker
  健康接口再次通过，未执行真实购买、支付、发币、节点部署、GitHub 推送或远端 CI。
