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

`citizenchain/runtime/primitives/china/china_*.rs` 中的内置机构是链上名称保护锚。
这些机构在 CID 系统中可以修改链下运营展示名；若要让区块链侧保护锚同步改变，
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

| 中文省名 | ProvinceBase | 独立英文名 |
|---|---|---|
| 中枢省 | Zhongshu | Zhongshu Province |
| 岭南省 | Lingnan | Lingnan Province |
| 广东省 | Guangdong | Guangdong Province |
| 广西省 | Guangxi | Guangxi Province |
| 福建省 | Fujian | Fujian Province |
| 海南省 | Hainan | Hainan Province |
| 云南省 | Yunnan | Yunnan Province |
| 贵州省 | Guizhou | Guizhou Province |
| 湖南省 | Hunan | Hunan Province |
| 江西省 | Jiangxi | Jiangxi Province |
| 浙江省 | Zhejiang | Zhejiang Province |
| 江苏省 | Jiangsu | Jiangsu Province |
| 山东省 | Shandong | Shandong Province |
| 山西省 | Shanxi | Shanxi Province |
| 河南省 | Henan | Henan Province |
| 河北省 | Hebei | Hebei Province |
| 湖北省 | Hubei | Hubei Province |
| 陕西省 | Shaanxi | Shaanxi Province |
| 重庆省 | Chongqing | Chongqing Province |
| 四川省 | Sichuan | Sichuan Province |
| 甘肃省 | Gansu | Gansu Province |
| 北平省 | Beiping | Beiping Province |
| 海滨省 | Haibin | Haibin Province |
| 松江省 | Songjiang | Songjiang Province |
| 龙江省 | Longjiang | Longjiang Province |
| 吉林省 | Jilin | Jilin Province |
| 辽宁省 | Liaoning | Liaoning Province |
| 宁夏省 | Ningxia | Ningxia Province |
| 青海省 | Qinghai | Qinghai Province |
| 安徽省 | Anhui | Anhui Province |
| 台湾省 | Taiwan | Taiwan Province |
| 西藏省 | Xizang | Xizang Province |
| 新疆省 | Xinjiang | Xinjiang Province |
| 西康省 | Xikang | Xikang Province |
| 阿里省 | Ali | Ali Province |
| 葱岭省 | Congling | Congling Province |
| 伊犁省 | Yili | Yili Province |
| 河西省 | Hexi | Hexi Province |
| 昆仑省 | Kunlun | Kunlun Province |
| 河套省 | Hetao | Hetao Province |
| 热河省 | Rehe | Rehe Province |
| 兴安省 | Xingan | Xingan Province |
| 合江省 | Hejiang | Hejiang Province |

## 5. 生成物边界

- `scripts/generate_citizenapp_governance_registry.mjs` 只能从 `china_cb.rs` / `china_ch.rs`
  读取四字段并生成公民端和公民钱包的治理机构注册表。
- 生成物不得手工维护机构名称；若生成物内容不一致，必须修改生成器或 runtime 常量。
- runtime 名称指纹统一使用 `builtin_institution_name_digest()` 和
  `BuiltinInstitutionNameApi`，指纹覆盖四字段，不再使用旧的二字段 API 命名。
