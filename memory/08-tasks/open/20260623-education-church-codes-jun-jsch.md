# 20260623 教育机构新增「教会大学/教会学校」两码(JUN/JSCH)

## 需求
教育组(F)新增:JUN 教会大学(3 位)、JSCH 教会学校(4 位);常量库与 CID 系统同步。码表 已并入 92,F 教育组 4→6,私法人桶 7→9。

## 决策(用户确认)
- 码名:**JUN**(J 教会+UN 大学,镜像 GUN/SUN)、**JSCH**(J 教会+SCH 学校,镜像 GSCH)。
- 范围:**本次只补码表 + 识别枚举**(让两码合法、被当教育/私法人处理);「教会」作第三种办学主体的**选择/派生入口**(用户建校时怎么分配到 JUN/JSCH)列 **follow-up**(未找到 computeEducationInstitutionCode,需单独排查接入)。
- 两码均私法人、profit Variable、教育机构;大学不需 education_level,学校需要。

## 改动(9 文件)
- `citizencode/backend/number/code.rs`:enum+2、as_code+2、cid_short_name+2、ALL+2(已并入 92)、profit_policy(Variable)、is_private_legal、is_education_institution、requires_education_level(仅 JSCH)、头注释/单测 已并入 92。
- `citizenchain/runtime/primitives/src/code.rs`:私法人谓词覆盖新增教育码、ALL_CODES 已并入 92、单测(all_codes_has_92/private=9)、头注释。
- `frontend/subjects/labels.ts`:INSTITUTION_CODE_LABEL + EDUCATION_INSTITUTION_CODE_LABEL 各 +2。
- `backend/main.rs`:教育排除 SQL `NOT IN(...)` +2(2 处)。
- `backend/subjects/admin.rs`:教育 IN + 私法人 SQL 清单 +2(各 2 处)。
- `backend/subjects/unincorporated_org/mod.rs`:`parent_is_education_school` 学校/大学父级 matches +2。
- `citizenapp` / `citizenwallet` institution_code:`_privateLegalCodes` +2 + 镜像注释 已并入 92、= 7→9。

## 验证(全绿)
- number 测试 6/6(codes_are_unique=92);primitives 6/6(all_codes_has_92_unique、桶一致、private=9)。
- CID 后端 cargo check 0 + test **83 passed**;链端 organization-manage/multisig-transfer 编译通过。
- 前端 tsc 0;citizenapp/citizenwallet analyze 0、format 干净;零残留。
- gov/service.rs **无** JUN/JSCH 模板(正确:教育机构不走区划 reconcile 模板,按需创建)。

## 遗留
- **链上生效**:用户跑 bake-chainspec.sh + clean-run.sh(常量库进 WASM)。
- **follow-up**:教育创建入口/派生加「教会」办学主体选项,才能真正生成 JUN/JSCH 号。
