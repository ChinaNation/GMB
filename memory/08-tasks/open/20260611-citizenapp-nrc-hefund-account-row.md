# 任务卡：citizenapp 国家储委会详情页新增两和基金账户行 + 金额右对齐核验

## 需求(user 确认 2026-06-11)

1. 国家公民储备委员会详情页"更多账户"中,两和基金账户显示在安全基金账户下面;
2. 费用账户、安全基金账户、两和基金账户金额靠右显示。

## 方案

链端 `NRC_HE_ACCOUNT`(两和基金,`ce19b7f0…`,china_cb.rs)已存在,citizenapp 数据链路缺位:

1. 生成器 `scripts/generate_citizenapp_governance_registry.mjs` 提取 `NRC_HE_ACCOUNT` → 国家储委会条目新增 `heAccount`,重生成注册表;
2. 模型 `InstitutionAccounts` 新增 `heAccount`(仅国家储委会非空);
3. 详情页 `_extraAccountSources()` 在安全基金账户后插入「两和基金账户」行(行序:费用 → 安全基金 → 两和基金);余额复用现有展开拉取逻辑;
4. 金额右对齐:三账户共用同一行组件(`_buildExpandedAccountItem`,金额行右端 `textAlign.right`),新增行自动继承,真机核验。

## 验收

- [x] 注册表国家储委会 heAccount = ce19b7f0…(与 china_cb.rs NRC_HE_ACCOUNT 逐字节一致)
- [x] `flutter analyze` 0 issue + `flutter test --concurrency=1` 196/196 全过
- [ ] 真机:两和基金账户显示在安全基金账户下方、三账户金额右对齐(user 验证)

## 追加修正 + 追加需求(2026-06-11,user 复测)

- **金额不靠右的真根因**(两和靠右/费用安全基金不靠右的割裂现象):行布局 `Row[icon, Expanded(名称), Flexible(金额)]` 中 Flexible 默认 loose,金额按内容宽度排在中线右侧,`textAlign.right` 失效;两和的超长金额恰好填满半行才看似靠右。修正:金额 `Flexible`→`Expanded`(tight),三行真正顶右。**教训:loose Flexible + textAlign.right 是无效组合,右对齐必须 tight 填满。**
- **追加需求:完整 SS58 地址**:主账户信息行与展开账户行(费用/安全基金/两和/永久质押)地址全部去 `_shortAddress` 截断,显示完整 SS58 并允许换行;`_buildAccountInfoTile` value 去单行 ellipsis;`_shortAddress` 函数已无引用删除。
- analyze 0 + test 196/196(串行)。

## 完工记录(2026-06-11)

- 生成器:提取 `NRC_HE_ACCOUNT` → 国家储委会条目 `heAccount`(仅 NRC,省储委会/省储行为 null),重生成注册表 87 机构;
- 模型:`InstitutionAccounts` 新增可选 `heAccount` 字段;
- 详情页:`_extraAccountSources()` 安全基金账户后插入「两和基金账户」行(icon=handshake_outlined),行序 费用 → 安全基金 → 两和基金;余额复用展开拉取逻辑零改动;
- 金额右对齐:三账户共用 `_buildExpandedAccountItem`(金额行右端 textAlign.right),新增行自动继承。
