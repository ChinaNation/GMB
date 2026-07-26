# 会员卡片提炼 + 会员详情页

任务需求(用户 2026-07-20 确认执行):
- 公民 App「我的-会员｜订阅」卡片:**保持现有样式/高度/左右滑动层叠功能不动**。
- 价格移到档色头带**右上角**,格式**千分号 + 两位小数 + 元**(如 `1,999.00 元/月`);链上读不到仍显示 `—`。
- 卡片核心权益**提炼为多条短行**(约 6 条,填满现高卡),每条 1 行不换行。
- 卡片底部加**「查看详细权益 ›」入口**,点击进入该档**会员详情页**。
- 新建**会员详情页**:分组展示该档完整权益(聊天 / 动态 / 文章 / 每月额度)+ 价格 + 订阅。
- 补齐 App model 缺字段:`dynamicMaxVideoBytes`(单视频体积,后端 plans.ts 已下发,App 未解析)、每月额度(monthly_images / monthly_video_seconds / active_uploads,现在只在 cloudflare `limits/catalog.ts`,plan 未下发)。后端 plans.ts + App model + App 兜底三处同步。

所属模块:
- CitizenApp:`lib/my/membership/`(卡片 + 新详情页 + model 解析)、`lib/8964/services/square_api_client.dart`(SquareMembershipPlan)
- Cloudflare:`cloudflare/src/membership/plans.ts`(plan 加每月额度字段)+ 端点序列化
- 价格换算:`lib/my/creator/creator_money.dart` 新增千分号+两位小数+元格式化

安全/边界:
- 不动左右滑动层叠交互(`_buildStackedCard` / 手势 / PageDots)。
- 不动 citizenchain(价格链上单源,App 只读)。
- 权益值双写口径(plans.ts + App `_fallbackMembershipPlans`)必须逐字段一致(现无漂移)。
- 改 DTO 形状:App 若缓存 plan 需 bump 缓存版本 + 形状校验(见 feedback-dto-field-rename-bump-cache-version)。

提炼字段(每档,来自 plans.ts / usageLimits 真值):
- 聊天文件:10MB / 100MB / 5GB(薪火 + 大文件中转)
- 动态视频:1分钟标清 / 30分钟高清 / 3小时高清;单视频体积 40MB / 1536MB / 8GB
- 动态图片:9 张,标清 / 高清 / 高清
- 文章:2万字·50图标清 / 3万字·100图高清 / 3万字·100图高清
- 每月额度:图 300·视频30分钟·并发1 / 图1500·视频3小时·并发2 / 图5000·视频30小时·并发3

验收标准:
- 卡片:价格右上角 `X,XXX.00 元/月`;6 条提炼短行;「查看详细权益」跳详情;左右滑动不受影响。
- 详情页:分组完整、逐档换色换值(自由金/民主蓝/薪火红)。
- `dart analyze` 通过;cloudflare `tsc` 通过;plans.ts 与 App 兜底数值一致。

当前进度:
- [x] 检查报表 + 3 卡片 UI + 详情页 UI 已确认。
- [x] 后端 plans.ts 加每月额度字段(`usage`=usageLimits 单源,端点 service.ts:80 透传)。
- [x] App model 加 dynamicMaxVideoBytes + monthlyImages/monthlyVideoSeconds/activeUploads + 解析 + 兜底;新增展示 getter。
- [x] 价格千分号两位小数元格式化(creator_money.fenToYuanMoneyLabel)。
- [x] 卡片改造:价格移头带右上(`X,XXX.00 元/月`)、权益 6 行提炼、加「查看详细权益」入口→详情页;左右滑动层叠未动;删死代码 _priceLabel。
- [x] 新建会员详情页 membership_detail_page.dart(聊天/动态/文章/每月额度分组 + 订阅按钮返回本页触发)。
- [x] 验证:dart analyze(5 文件)No issues;cloudflare tsc OK;plans.ts↔App 兜底逐档数值一致(视频体积 sd40MB/hd1536MB/spark8GB、每月额度 300·1800·1 / 1500·10800·2 / 5000·108000·3);修 _fileSize 让 1536MB 显示 1.5GB 不再误四舍五入为 2GB。
- [ ] 待用户经部署控制台「编译软件」在真机确认视觉。

## 2026-07-25 首屏即时渲染与低频刷新修复

用户确认修复「每次进入会员页都全屏转圈，等待三张静态会员卡」的问题。

检查结论:
- `MembershipPage` 首屏被 `_loading` 全屏进度圈占据，必须等待会话、Cloudflare 会员接口、三档链上价格和 finalized 订阅真态全部完成后才渲染卡片。
- 三档权益已有 App 内置兜底定义，本不应依赖网络；Cloudflare 返回的 `plans` 同样是静态定义。
- 三档价格当前逐档串行链读；订阅真态读取前还同步重试历史 Cloudflare 镜像回执，任一慢请求都会拖住整页。
- 页面销毁重建后没有会员展示快照缓存；现有测试只在 `pumpAndSettle()` 后断言，没有覆盖请求未完成时的首屏。

目标状态:
- [x] 进入页面第一帧立即显示 App 内置的自由/民主/薪火三张卡，不出现全屏加载圈。
- [x] 按 `account_id` 持久化价格和 finalized 订阅展示快照；缓存有效期内进入页面不发起重复链读。
- [x] 缓存过期后后台刷新，仅局部更新价格、当前会员和周期信息；首次无缓存时价格显示占位且订阅按钮禁用。
- [x] 订阅、换档、取消成功、用户手动刷新时绕过缓存，强制读取最新链上状态。
- [x] Cloudflare 镜像回执重试不再阻塞 finalized 订阅读取。
- [x] 三档价格改为一次批量 finalized storage 读取。
- [x] 增加未完成请求首屏、缓存命中、手动强刷和失败保留旧快照测试。

预计修改目录:
- `citizenapp/lib/my/membership/`: 现有页面与订阅编排代码；实现即时首屏、快照缓存、后台刷新和镜像重试解耦，不新增目录。
- `citizenapp/lib/8964/chain/`: 现有链读服务；把三档价格串行读取改为批量读取。
- `citizenapp/test/8964/`: 现有广场链服务测试；验证三档价格只产生一次批量 finalized storage 读取。
- `citizenapp/test/my/membership/`: 现有会员页测试；补齐首屏与缓存刷新行为，不新增测试文件。
- `memory/01-architecture/citizenapp/`: 现有 CitizenApp 技术文档；记录缓存、失效和交易前真值校验边界。
- `memory/08-tasks/open/`: 本任务卡追加实施与验收记录；不新增任务卡。

实施与验收:
- `MembershipPage` 删除全屏 `_loading` / `_MembershipMessage` 首屏门禁，初始状态直接挂载 App 内置三档套餐；会话、缓存和链读只更新动态字段。无钱包显示「请先创建热钱包」，有钱包但 finalized 状态尚未成功读取时显示「会员状态同步中」，两种情况都不隐藏卡片且不放行订阅按钮。
- `MembershipDisplaySnapshot` 使用 SharedPreferences 按 `account_id` 保存订阅态、三档价格及各自更新时间；订阅态 TTL=5 分钟，价格 TTL=30 分钟。套餐权益不进入缓存，防止静态 DTO 陈旧。
- 订阅、换档、取消动权前重新读取 finalized 订阅态；需要价格的动作通过 `forceFresh` 绕过展示缓存。Cloudflare finalized 镜像回执改为后台重试。
- `SquareChainService.fetchAllPlatformPrices()` 从三次串行 storage 读取改为一次 `fetchStorageBatch()`，并支持手动刷新/动权前 `forceFresh`。
- 定向 `flutter analyze` 通过；会员页 + 广场链服务测试 25/25 通过。
- CitizenApp 全量 `flutter analyze` 通过；全量 `flutter test` 802 项通过、5 项按既有原生库缺失条件跳过，零失败。
- Android 16 模拟器真实运行：从「我的 → 会员｜订阅」点击进入后约 0.2 秒采集页面语义树，已存在自由/民主/薪火三张卡及静态权益；当时轻节点仍在后台同步，价格仅显示 `—` /「链上价格未就绪」，未出现全屏加载页。随后日志确认 smoldot 继续在后台完成 regular 同步。
