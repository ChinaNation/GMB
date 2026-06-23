# 20260623 公权机构「市名」展示修复(Bug1 citizenapp + Bug2 CID 前端)

## 背景 / 现象
- Bug1(citizenapp 公民→公权→公权机构):市卡片副文冗余「查看 xx市 公权机构」;市名字典 join 取空时整段留白(card 显示「查看 公权机构」、顶部「公权机构」)。
- Bug2(CID 系统 公权机构/教育机构 tab,联邦注册局管理员视图):城市选择卡(CityGrid)市名取空时渲染成空白细卡。

## 根因(已核实,代码层字段全对)
- 字段映射全部正确:citizenapp DTO `json['name']→divisionName`、后端 `CidCityItem.city_name`(无 rename)、前端 `CidCityItem.city_name`、CityGrid 渲染 `c.city_name`、city_institution_list_page 顶部已是 `${cityName}公权机构`。
- 「空市名」根因 = 设备端行政区字典/城市缓存未同步或为旧数据,**非代码 bug**。
- 设计诉求:卡片只显「xx市」、进入后顶部「xx市公权机构」;CID 城市选择卡只显「xx市」。

## 改动(本卡)
1. citizenapp `public_page.dart`
   - 市卡片改为只显 `city.name`(去掉「查看 … 公权机构」副文);`_InstitutionTile.subtitle` 改可空。
   - `_loadCityVms` 防御:字典名为空串时回退 code(原 `?? code` 不挡空串),杜绝留白。
2. CID 前端 `CityGrid.tsx`
   - 市卡 `c.city_name` 为空时回退 `c.code`,杜绝空白细卡。
3. 顶部标题 `city_institution_list_page.dart` 已是 `${cityName}公权机构`,**无需改**。

## 验证
- citizenapp `flutter analyze` 0 error;CID 前端 `tsc` 0 error。

## 遗留 / 提示
- 真要显示真实市名,设备端需重新同步公权机构 + 行政区数据包(本卡只保证不留白、按设计精简)。
