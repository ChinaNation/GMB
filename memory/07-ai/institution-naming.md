# 机构命名规范

## 1. 定位

本文件登记机构具体名称的统一口径。字段、类型、目录和文件命名仍以
`memory/07-ai/unified-naming.md` 为总入口；本文件只管“每个机构叫什么”以及
机构名称字段如何承载这些值。

## 2. 唯一字段

所有机构名称只允许使用以下四个字段：

| 中文含义 | 字段名 | Dart/TS 代码名 | 说明 |
|---|---|---|---|
| 中文全称 | `cid_full_name` | `cidFullName` | 机构中文完整名称 |
| 中文简称 | `cid_short_name` | `cidShortName` | 机构中文简称或紧凑展示名 |
| 英文全称 | `cid_full_name_en` | `cidFullNameEn` | 机构英文完整名称 |
| 英文简称 | `cid_short_name_en` | `cidShortNameEn` | 机构英文简称或紧凑展示名 |

禁止再用 `name`、`display_name`、`official_name`、`english_name`、
`full_name_en`、`short_name_en` 等字段承载机构名称。

## 3. 常量库机构

`citizenchain/runtime/primitives/cid/china/china_*.rs` 中的内置机构是链上名称保护锚。
这些机构在 OnChina 中可以修改链下运营展示名；若要让区块链侧保护锚同步改变，
必须修改 runtime 常量并通过 runtime 升级生效。

1.1 已统一的常量库机构范围：

| 文件 | 机构数 | 内容 |
|---|---:|---|
| `china_zf.rs` | 59 | 总统府、联邦局、部委、省级联邦政府 |
| `china_lf.rs` | 44 | 国家立法院、省级联邦立法院 |
| `china_sf.rs` | 44 | 国家司法院、省级联邦司法院 |
| `china_jc.rs` | 47 | 国家监察院、联邦署、省级联邦监察院 |
| `china_jy.rs` | 1 | 国家公民教育委员会 |
| `china_cb.rs` | 44 | 国储会、省储会 |
| `china_ch.rs` | 43 | 省储行 |

`china_zb.rs` 只保存制度保留账户，不保存机构名称。

## 4. 英文命名规则

### 4.1 国家级机构

| 中文全称 | 中文简称 | 英文全称 | 英文简称 |
|---|---|---|---|
| 中华民族联邦共和国总统府 | 总统府 | Presidential Office of the Federal Republic of the China Nation | Presidential Office |
| 总统府联邦安全局 | 联邦安全局 | Federal Security Bureau of the Presidential Office | Federal Security Bureau |
| 总统府联邦情报局 | 联邦情报局 | Federal Intelligence Bureau of the Presidential Office | Federal Intelligence Bureau |
| 总统府联邦特勤局 | 联邦特勤局 | Federal Secret Service Bureau of the Presidential Office | Federal Secret Service Bureau |
| 总统府联邦人事局 | 联邦人事局 | Federal Personnel Bureau of the Presidential Office | Federal Personnel Bureau |
| 总统府联邦注册局 | 联邦注册局 | Federal Registry Bureau of the Presidential Office | Federal Registry Bureau |
| 中华民族联邦共和国国家立法院 | 国家立法院 | National Legislative Yuan of the Federal Republic of the China Nation | National Legislative Yuan |
| 中华民族联邦共和国国家司法院 | 国家司法院 | National Judicial Yuan of the Federal Republic of the China Nation | National Judicial Yuan |
| 中华民族联邦共和国国家监察院 | 国家监察院 | National Control Yuan of the Federal Republic of the China Nation | National Control Yuan |
| 中华民族联邦共和国国家公民教育委员会 | 国家教委会 | National Citizen Education Committee of the Federal Republic of the China Nation | National Education Committee |
| 国家公民储备委员会 | 国储会 | National Citizen Reserve Committee | National Reserve Committee |

### 4.2 部委

| 中文全称 | 中文简称 | 英文全称 | 英文简称 |
|---|---|---|---|
| 中华民族联邦共和国外事交流部 | 外交部 | Ministry of Foreign Affairs and Exchange of the Federal Republic of the China Nation | Ministry of Foreign Affairs |
| 中华民族联邦共和国国家防务部 | 国防部 | Ministry of National Defense of the Federal Republic of the China Nation | Ministry of National Defense |
| 中华民族联邦共和国国土安全部 | 国安部 | Ministry of Homeland Security of the Federal Republic of the China Nation | Ministry of Homeland Security |
| 中华民族联邦共和国公民生活保障部 | 民生部 | Ministry of Citizen Welfare of the Federal Republic of the China Nation | Ministry of Citizen Welfare |
| 中华民族联邦共和国住房与城镇建设部 | 住建部 | Ministry of Housing and Urban Development of the Federal Republic of the China Nation | Ministry of Housing and Urban Development |
| 中华民族联邦共和国农业与农村发展部 | 农业部 | Ministry of Agriculture and Rural Development of the Federal Republic of the China Nation | Ministry of Agriculture |
| 中华民族联邦共和国商务与市场贸易部 | 商贸部 | Ministry of Commerce and Market Trade of the Federal Republic of the China Nation | Ministry of Commerce |
| 中华民族联邦共和国财政与税务部 | 财税部 | Ministry of Finance and Taxation of the Federal Republic of the China Nation | Ministry of Finance and Taxation |
| 中华民族联邦共和国能源与环保发展部 | 能源部 | Ministry of Energy and Environmental Development of the Federal Republic of the China Nation | Ministry of Energy |
| 中华民族联邦共和国交通运输部 | 交通部 | Ministry of Transport of the Federal Republic of the China Nation | Ministry of Transport |

### 4.3 监察机构

| 中文全称 | 中文简称 | 英文全称 | 英文简称 |
|---|---|---|---|
| 国家监察院联邦廉政署 | 联邦廉政署 | Federal Integrity Agency of the National Control Yuan | Federal Integrity Agency |
| 国家监察院联邦审计署 | 联邦审计署 | Federal Audit Agency of the National Control Yuan | Federal Audit Agency |
| 国家监察院联邦调查署 | 联邦调查署 | Federal Investigation Agency of the National Control Yuan | Federal Investigation Agency |

### 4.4 省级模板

省名作为独立地名时英文为 `${ProvinceBase} Province`；用于机构英文名前缀时使用
`${ProvinceBase} Provincial ...`。

| 中文模式 | 英文全称模式 | 英文简称模式 |
|---|---|---|
| `X省联邦政府` / `X省政府` | `${X} Provincial Federal Government` | `${X} Provincial Government` |
| `X省联邦立法院` / `X省立法院` | `${X} Provincial Federal Legislative Yuan` | `${X} Provincial Legislative Yuan` |
| `X省联邦司法院` / `X省司法院` | `${X} Provincial Federal Judicial Yuan` | `${X} Provincial Judicial Yuan` |
| `X省联邦监察院` / `X省监察院` | `${X} Provincial Federal Control Yuan` | `${X} Provincial Control Yuan` |
| `X省公民储备委员会` / `X省储会` | `${X} Provincial Citizen Reserve Committee` | `${X} Provincial Reserve Committee` |
| `X省公民储备银行` / `X省储行` | `${X} Provincial Citizen Reserve Bank` | `${X} Provincial Reserve Bank` |

### 4.5 省名英文

| 省代码 | 中文省名 | ProvinceBase | 独立英文名 |
|---|---|---|---|
| ZS | 中枢省 | Zhongshu | Zhongshu Province |
| LN | 岭南省 | Lingnan | Lingnan Province |
| GD | 广东省 | Guangdong | Guangdong Province |
| GX | 广西省 | Guangxi | Guangxi Province |
| FJ | 福建省 | Fujian | Fujian Province |
| HN | 海南省 | Hainan | Hainan Province |
| YN | 云南省 | Yunnan | Yunnan Province |
| GZ | 贵州省 | Guizhou | Guizhou Province |
| HU | 湖南省 | Hunan | Hunan Province |
| JX | 江西省 | Jiangxi | Jiangxi Province |
| ZJ | 浙江省 | Zhejiang | Zhejiang Province |
| JS | 江苏省 | Jiangsu | Jiangsu Province |
| SD | 山东省 | Shandong | Shandong Province |
| SX | 山西省 | Shanxi | Shanxi Province |
| HE | 河南省 | Henan | Henan Province |
| HB | 河北省 | Hebei | Hebei Province |
| HI | 湖北省 | Hubei | Hubei Province |
| SI | 陕西省 | Shaanxi | Shaanxi Province |
| CQ | 重庆省 | Chongqing | Chongqing Province |
| SC | 四川省 | Sichuan | Sichuan Province |
| GS | 甘肃省 | Gansu | Gansu Province |
| BP | 北平省 | Beiping | Beiping Province |
| HA | 海滨省 | Haibin | Haibin Province |
| SJ | 松江省 | Songjiang | Songjiang Province |
| LJ | 龙江省 | Longjiang | Longjiang Province |
| JL | 吉林省 | Jilin | Jilin Province |
| LI | 辽宁省 | Liaoning | Liaoning Province |
| NX | 宁夏省 | Ningxia | Ningxia Province |
| QH | 青海省 | Qinghai | Qinghai Province |
| AH | 安徽省 | Anhui | Anhui Province |
| TW | 台湾省 | Taiwan | Taiwan Province |
| XZ | 西藏省 | Xizang | Xizang Province |
| XJ | 新疆省 | Xinjiang | Xinjiang Province |
| XK | 西康省 | Xikang | Xikang Province |
| AL | 阿里省 | Ali | Ali Province |
| CL | 葱岭省 | Congling | Congling Province |
| YL | 伊犁省 | Yili | Yili Province |
| HX | 河西省 | Hexi | Hexi Province |
| KL | 昆仑省 | Kunlun | Kunlun Province |
| HT | 河套省 | Hetao | Hetao Province |
| RH | 热河省 | Rehe | Rehe Province |
| XA | 兴安省 | Xingan | Xingan Province |
| HJ | 合江省 | Hejiang | Hejiang Province |

## 5. 非常量机构模板

本节只登记非常量机构的命名规范和行政区代码规则。非常量机构的英文名暂时不进入
CID 数据库、API、前端字段或生成物；需要英文展示时从本节规则取用。

行政区代码字段含义：

| 字段 | 含义 | 来源 |
|---|---|---|
| `province_code` | 所属省代码 | `citizenchain/onchina/src/cid/china/china.sqlite` 的 `provinces.code` |
| `city_code` | 所属市代码 | `citizenchain/onchina/src/cid/china/china.sqlite` 的 `cities.code` |
| `town_code` | 所属镇代码 | `citizenchain/onchina/src/cid/china/china.sqlite` 的 `towns.code`；非镇级机构为空 |

### 5.1 国家级非常量机构

| 机构码 | province_code | city_code | town_code | 中文全称 | 中文简称 | 英文全称规范 | 英文简称规范 |
|---|---|---|---|---|---|---|---|
| NSN | ZS | 001 | 空 | 中华民族联邦共和国国家立法院参议员议政会 | 国家参议会 | National Senate Deliberative Council of the National Legislative Yuan of the Federal Republic of the China Nation | National Senate |
| NRP | ZS | 001 | 空 | 中华民族联邦共和国国家立法院众议员议政会 | 国家众议会 | National House of Representatives Deliberative Council of the National Legislative Yuan of the Federal Republic of the China Nation | National House of Representatives |

### 5.2 省级非常量机构

省级非常量机构使用 `{province_code}` 和省本级锚定 `{city_code}`，当前锚定市代码为 `001`；
`town_code` 为空。中文名以 `{省名}` 拼接，英文名以 `{ProvinceBase}` 拼接。

| 机构码 | province_code | city_code | town_code | 中文全称模式 | 中文简称模式 | 英文全称规范 | 英文简称规范 |
|---|---|---|---|---|---|---|---|
| PDF | `{province_code}` | `001` | 空 | `{省名}国家防务厅` | `{省名}国防厅` | `{ProvinceBase} Provincial Department of National Defense` | `{ProvinceBase} Defense Department` |
| PHS | `{province_code}` | `001` | 空 | `{省名}国土安全厅` | `{省名}国安厅` | `{ProvinceBase} Provincial Department of Homeland Security` | `{ProvinceBase} Homeland Security Department` |
| PCW | `{province_code}` | `001` | 空 | `{省名}公民生活保障厅` | `{省名}民生厅` | `{ProvinceBase} Provincial Department of Citizen Welfare` | `{ProvinceBase} Citizen Welfare Department` |
| PHU | `{province_code}` | `001` | 空 | `{省名}住房与城镇建设厅` | `{省名}住建厅` | `{ProvinceBase} Provincial Department of Housing and Urban Development` | `{ProvinceBase} Housing Department` |
| PAG | `{province_code}` | `001` | 空 | `{省名}农业与农村发展厅` | `{省名}农业厅` | `{ProvinceBase} Provincial Department of Agriculture and Rural Development` | `{ProvinceBase} Agriculture Department` |
| PCM | `{province_code}` | `001` | 空 | `{省名}商务与市场贸易厅` | `{省名}商贸厅` | `{ProvinceBase} Provincial Department of Commerce and Market Trade` | `{ProvinceBase} Commerce Department` |
| PFT | `{province_code}` | `001` | 空 | `{省名}财政与税务厅` | `{省名}财税厅` | `{ProvinceBase} Provincial Department of Finance and Taxation` | `{ProvinceBase} Finance and Taxation Department` |
| PEN | `{province_code}` | `001` | 空 | `{省名}能源与环保发展厅` | `{省名}能源厅` | `{ProvinceBase} Provincial Department of Energy and Environmental Development` | `{ProvinceBase} Energy Department` |
| PTR | `{province_code}` | `001` | 空 | `{省名}交通运输厅` | `{省名}交通厅` | `{ProvinceBase} Provincial Department of Transport` | `{ProvinceBase} Transport Department` |
| PSN | `{province_code}` | `001` | 空 | `{省名}参议员议政会` | `{省名}参议会` | `{ProvinceBase} Provincial Senate Deliberative Council` | `{ProvinceBase} Provincial Senate` |
| PRP | `{province_code}` | `001` | 空 | `{省名}众议员议政会` | `{省名}众议会` | `{ProvinceBase} Provincial House of Representatives Deliberative Council` | `{ProvinceBase} Provincial House of Representatives` |

### 5.3 市级非常量机构

市级非常量机构使用 `{province_code}` 和 `{city_code}`，`town_code` 为空。
中文名以 `{市名}` 拼接，英文名以 `{CityBase}` 拼接。

| 机构码 | province_code | city_code | town_code | 中文全称模式 | 中文简称模式 | 英文全称规范 | 英文简称规范 |
|---|---|---|---|---|---|---|---|
| CGOV | `{province_code}` | `{city_code}` | 空 | `{市名}自治政府` | `{市名}政府` | `{CityBase} Municipal Autonomous Government` | `{CityBase} Municipal Government` |
| CLEG | `{province_code}` | `{city_code}` | 空 | `{市名}公民立法委员会` | `{市名}立法会` | `{CityBase} Municipal Citizen Legislative Committee` | `{CityBase} Municipal Legislative Committee` |
| CSUP | `{province_code}` | `{city_code}` | 空 | `{市名}监察院` | `{市名}监察院` | `{CityBase} Municipal Control Yuan` | `{CityBase} Municipal Control Yuan` |
| CJUD | `{province_code}` | `{city_code}` | 空 | `{市名}司法院` | `{市名}司法院` | `{CityBase} Municipal Judicial Yuan` | `{CityBase} Municipal Judicial Yuan` |
| CEDU | `{province_code}` | `{city_code}` | 空 | `{市名}公民教育委员会` | `{市名}教委会` | `{CityBase} Municipal Citizen Education Committee` | `{CityBase} Municipal Education Committee` |
| CSLF | `{province_code}` | `{city_code}` | 空 | `{市名}公民自治委员会` | `{市名}自治会` | `{CityBase} Municipal Citizen Self-Governance Committee` | `{CityBase} Municipal Self-Governance Committee` |
| CDEF | `{province_code}` | `{city_code}` | 空 | `{市名}国家防务局` | `{市名}国防局` | `{CityBase} Municipal Bureau of National Defense` | `{CityBase} Defense Bureau` |
| CHSC | `{province_code}` | `{city_code}` | 空 | `{市名}国土安全局` | `{市名}国安局` | `{CityBase} Municipal Bureau of Homeland Security` | `{CityBase} Homeland Security Bureau` |
| CCWF | `{province_code}` | `{city_code}` | 空 | `{市名}公民生活保障局` | `{市名}民生局` | `{CityBase} Municipal Bureau of Citizen Welfare` | `{CityBase} Citizen Welfare Bureau` |
| CHUD | `{province_code}` | `{city_code}` | 空 | `{市名}住房与城镇建设局` | `{市名}住建局` | `{CityBase} Municipal Bureau of Housing and Urban Development` | `{CityBase} Housing Bureau` |
| CAGR | `{province_code}` | `{city_code}` | 空 | `{市名}农业与农村发展局` | `{市名}农业局` | `{CityBase} Municipal Bureau of Agriculture and Rural Development` | `{CityBase} Agriculture Bureau` |
| CCOM | `{province_code}` | `{city_code}` | 空 | `{市名}商务与市场贸易局` | `{市名}商贸局` | `{CityBase} Municipal Bureau of Commerce and Market Trade` | `{CityBase} Commerce Bureau` |
| CFIN | `{province_code}` | `{city_code}` | 空 | `{市名}财政与税务局` | `{市名}财税局` | `{CityBase} Municipal Bureau of Finance and Taxation` | `{CityBase} Finance and Taxation Bureau` |
| CENR | `{province_code}` | `{city_code}` | 空 | `{市名}能源与环保发展局` | `{市名}能源局` | `{CityBase} Municipal Bureau of Energy and Environmental Development` | `{CityBase} Energy Bureau` |
| CTRN | `{province_code}` | `{city_code}` | 空 | `{市名}交通运输局` | `{市名}交通局` | `{CityBase} Municipal Bureau of Transport` | `{CityBase} Transport Bureau` |
| CREG | `{province_code}` | `{city_code}` | 空 | `{市名}身份注册局` | `{市名}注册局` | `{CityBase} Municipal Identity Registry Bureau` | `{CityBase} Registry Bureau` |
| CPOL | `{province_code}` | `{city_code}` | 空 | `{市名}公民安全局` | `{市名}公安局` | `{CityBase} Municipal Bureau of Citizen Security` | `{CityBase} Public Security Bureau` |

### 5.4 镇级非常量机构

镇级非常量机构使用 `{province_code}`、`{city_code}` 和 `{town_code}`。
中文名以 `{镇名}` 拼接，英文名以 `{TownBase}` 拼接。

| 机构码 | province_code | city_code | town_code | 中文全称模式 | 中文简称模式 | 英文全称规范 | 英文简称规范 |
|---|---|---|---|---|---|---|---|
| TGOV | `{province_code}` | `{city_code}` | `{town_code}` | `{镇名}自治政府` | `{镇名}政府` | `{TownBase} Town Autonomous Government` | `{TownBase} Town Government` |
| TCWF | `{province_code}` | `{city_code}` | `{town_code}` | `{镇名}公民生活保障科` | `{镇名}民生科` | `{TownBase} Town Citizen Welfare Section` | `{TownBase} Welfare Section` |
| THUD | `{province_code}` | `{city_code}` | `{town_code}` | `{镇名}住房与城镇建设科` | `{镇名}住建科` | `{TownBase} Town Housing and Urban Development Section` | `{TownBase} Housing Section` |
| TAGR | `{province_code}` | `{city_code}` | `{town_code}` | `{镇名}农业与农村发展科` | `{镇名}农业科` | `{TownBase} Town Agriculture and Rural Development Section` | `{TownBase} Agriculture Section` |
| TFIN | `{province_code}` | `{city_code}` | `{town_code}` | `{镇名}财政与税务科` | `{镇名}财税科` | `{TownBase} Town Finance and Taxation Section` | `{TownBase} Finance and Taxation Section` |

## 6. 生成物边界

- `scripts/generate_citizenapp_governance_registry.mjs` 只能从 `china_cb.rs` / `china_ch.rs`
  读取四字段并生成公民端和公民钱包的治理机构注册表。
- 生成物不得手工维护机构名称；若生成物内容不一致，必须修改生成器或 runtime 常量。
- runtime 名称指纹统一使用 `builtin_institution_name_digest()` 和
  `BuiltinInstitutionNameApi`，指纹覆盖四字段，不再使用旧的二字段 API 命名。
