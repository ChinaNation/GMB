# 20260623 机构码表补全(镇 4 码)+ 常量库改名 code.rs + 两边 A-I 九组对齐

## 需求
1. CID `number/code.rs` 补 4 个镇公权机构码:TPOL 镇公安科 / TSLF 镇自治会 / TSUP 镇监察院 / TJUD 镇司法院(镜像市级 CPOL/CSLF/CSUP/CJUD)。码表 已并入 92,D 镇组 10→14。
2. 常量库 `primitives/src/code.rs` → `code.rs`(与 CID 对称);补 4 镇码;按 A-I 九组重排。

## 决策(用户确认)
- A-I 九组:**CID code.rs 与常量库都重排**(教育 E2→F、个人→G、非法人→H、个人多签→I)。
- 镇级模板统一补齐 `TDEF/THSC/TCOM/TENR/TTRN/TPOL/TSLF/TSUP/TJUD`，其中 `TSLF` 为镇自治会，`TSUP/TJUD` 为镇监察院/镇司法院。
- 常量库属链上 WASM:**只改代码 + 验证**,重烤 SSOT(bake-chainspec.sh)+ clean-run 由用户自行执行。

## A-I 九组终态(90 码)
A 国家级26 / B 省级17 / C 市级17 / D 镇级**14** / E 私权7 / F 教育4 / G 个人3 / H 非法人1 / I 个人多签1

## 改动清单
- `citizencode/backend/number/code.rs`:enum+4、as_code+4、cid_short_name+4、ALL+4(已并入 92)、admin_level Town+4、头注释/组注释重排 A-I、单测 已并入 92。
- `citizencode/frontend/subjects/labels.ts`:INSTITUTION_CODE_LABEL +4。
- `citizencode/backend/gov/service.rs`:TOWN_TEMPLATES +4。
- `citizenapp` 机构码标签字典:+4。
- `citizenchain/runtime/primitives/src/code.rs`→`code.rs`:改名 + D 组 +4(已并入 92)+ A-I 重排 + 单测 已并入 92。
- `citizenchain/runtime/primitives/src/lib.rs`:`mod institution_code`→`mod code`。
- 4 个引用文件(organization-manage traits/types、votingengine types 等):`institution_code::`→`code::`。

## 验证
- 链端 cargo check + 相关 test;CID 后端 cargo test;前端 tsc;citizenapp flutter analyze。

## 遗留
- 链上生效需用户跑 bake-chainspec.sh + clean-run.sh(常量库进 WASM)。
