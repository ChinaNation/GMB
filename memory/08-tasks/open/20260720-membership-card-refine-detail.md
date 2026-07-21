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
