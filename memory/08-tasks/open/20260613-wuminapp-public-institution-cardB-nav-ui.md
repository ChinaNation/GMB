# 任务卡:公权机构卡B — citizen/public 省/市/机构导航 + 订阅 UI

属 ADR-018 §九。替换 `citizen/public/public_page.dart` 占位页,实现公权机构浏览导航。依赖卡 A(数据层)。

状态:**代码完工(2026-06-13)**。`citizen/public/public_page.dart` 替换占位页:左竖向可滚动省导航(关注 + repo.listProvinces 省份,manifest 顺序)+ 右内容(关注=订阅机构扁平列表 / 省=市列表)。`city_institution_list_page.dart` 市机构列表(按类型标签 ZF/LF/SF/JC/JY)。`public_institution_detail_page.dart` **最小骨架**(名称/身份ID/省市/账户数 + 账户/提案/管理员占位卡片,卡C 扩充)。全程读本地 repo,省份打开尽力而为同步;活动钱包公钥经 walletPubkeyProvider 注入(测试免 Isar)。widget 测试 4/4(左栏渲染/选省出市/点市进列表/关注渲订阅)+ analyze 0 + citizen/public 子树 14/14。

详情页动态(余额/提案/管理员/订阅按钮)留卡C。

**UI 修订(2026-06-13,user review 3 项)**:
1. 左栏省份改由 `data/public_provinces.dart::publicProvinceNames()` **复用治理 `kProvincialCouncils` 同一行政区**(去 `公民储备委员会` 后缀,**保留"省"**→ `中枢省`,与 china.sqlite/SFID province 字段逐字对齐),始终全显 43 省,不再依赖数据包是否加载(原 repo.listProvinces 空 manifest 致只剩"关注"的 bug 修复)。
2. 恢复顶部"公权机构"标题(fontSize 22/w700,与治理 tab"治理机构"对称);布局改 Column(标题 → Row(左栏|内容))。
3. 左栏重设计:删竖线(容器右边框)、删选中左强调条、删灰底面板;关注+省直接在页面背景上排列,可上下滚动,选中态仅文字加粗变色 + 轻量圆角底(无任何线)。
测试同步更新省名为 `中枢省`;citizen/public 17/17 + 全量 232/232 无回归。

## 需求(用户口径)
- 公权机构页**靠左竖向可滚动省导航**(43 省一屏放不下,上下滑动):`关注`(默认选中)+ `中枢` + 43 省(岭南/广东/广西…)。
- `关注` = 该用户订阅的公权机构(跨省扁平列表)。
- 选中某省 → 右侧显示该省所有**市**;点某市 → 进入该市**公权机构列表**;点机构 → 详情页(卡 C)。

## 完工清单
- [ ] 替换 `citizen/public/public_page.dart`:左 NavigationRail/竖列(可滚动 ListView)+ 右内容区。
- [ ] 左列项顺序:`关注` → `中枢` → 省列表(名称从卡 A 的 listProvinces,Isar 本地)。
- [ ] 选 `关注`:右侧渲染订阅机构扁平列表(卡 A listSubscribed),点进详情。
- [ ] 选某省:右侧渲染该省市列表(listCitiesByProvince),点市 → `city_institution_list_page.dart`。
- [ ] `city_institution_list_page.dart`:listInstitutionsByCity → 机构列表(简要名称),点进详情页(卡 C)。
- [ ] 列表项展示简要信息(名称/简称 + 机构类型),复用 governance_list_page 样式与 hex→SS58 风格。
- [ ] 全程**只读 Isar,零网络**(导航不触发链读/SFID 调用)。

## 单测/widget 测
- [ ] 左列含 关注+中枢+省;关注默认选中。
- [ ] 选省→市列表;选市→机构列表(用 fake 数据层)。
- [ ] 空订阅时关注页空态文案。

## 验收
- [ ] flutter analyze 0 + flutter test 全过。
- [ ] 真机:省导航可上下滚动;省→市→机构→详情 跳转通畅;关注分组正确。

## 不做(边界)
- 不做详情页内容与动态(卡 C);订阅按钮在详情页右上角(卡 C 实现写入,卡 B 只消费关注集合)。
- 不做发起提案(v1 范围外)。

## 改动目录(中文注释)
- 改 `wuminapp/lib/citizen/public/public_page.dart`:占位页 → 省/市导航,代码。
- 新增 `wuminapp/lib/citizen/public/city_institution_list_page.dart` 及导航 widget,代码。
- 复用 `wuminapp/lib/governance/` 列表样式(只读复用,不改 governance)。
