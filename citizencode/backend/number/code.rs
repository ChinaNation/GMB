#![allow(dead_code)]
//! 机构类型代码枚举(CID 号机构码,全仓库机构分类的**唯一真源**)
//!
//! 中文注释:
//!   主体属性折进机构码本身(每个码自带公/私/个人语义)。
//! - 国家级单体 + 省级类型用 **3 字符码**;市/镇/私权/个人/个人多签用 **4 字符码**。
//!   号码段二靠「机构码长度」分两种布局(见 `validator.rs`):
//!   - 3 字符码:`码(3) + 盈利位(1,恒0) + 校验(1, mod-36)`
//!   - 4 字符码:`码(4) + M1(1, 数字=盈利 mod-10 / 字母=非盈利 mod-26)`
//! - 盈利属性由 `profit_policy()` 决定;可变实体(SFAS/SMTP/UNIN)按实例/继承父级。
//!
//! 码表(92 个):
//! A 国家级单体(26,3 位)    B 省级类型(17,3 位)   C 市级类型(17,4 位)
//! D 镇级类型(14,4 位)     E 私权机构(7,4 位)    F 教育学校(6:大学3=3位/中小初学3=4位)
//! G 个人主体(3,4 位)      H 非法人组织(1,4 位)  I 个人多签(1,4 位,不发号)

use serde::{Deserialize, Serialize};

/// 机构码所属行政层级(由机构码本身派生)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdminLevel {
    /// 国家级(26 个 A 组国家级单体)。
    National,
    /// 省级(17 个 B 组省级类型)。
    Province,
    /// 市级(17 个 C 组市级类型)。
    City,
    /// 镇级(14 个 D 组镇级类型)。
    Town,
}

/// 机构码的盈利策略(决定号码 M1 / 盈利位如何生成与校验)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProfitPolicy {
    /// 固定非盈利(公权机构、公益组织)。
    NonProfit,
    /// 固定盈利(经营性私权实体、公民人/自然人)。
    Profit,
    /// 按实例可变(注册协会、智能人)。
    Variable,
    /// 继承父级法人盈利属性(非法人组织)。
    InheritParent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InstitutionCode {
    // A 国家级单体(26,3 位,公法人,非盈利)
    Prs, // 总统府
    Fsc, // 联邦安全局
    Fib, // 联邦情报局
    Fss, // 联邦特勤局
    Fpr, // 联邦人事局
    Frg, // 联邦注册局
    Mfa, // 外事交流部
    Mdf, // 国家防务部
    Mhs, // 国土安全部
    Mcw, // 公民生活保障部
    Mhu, // 住房与城镇建设部
    Mag, // 农业与农村发展部
    Mcm, // 商务与市场贸易部
    Mft, // 财政与税务部
    Men, // 能源与环保发展部
    Mtr, // 交通运输部
    Nlg, // 国家立法院
    Njd, // 国家司法院
    Nsp, // 国家监察院
    Fac, // 联邦廉政署
    Fau, // 联邦审计署
    Fiv, // 联邦调查署
    Ned, // 国家公民教育委员会
    Nrc, // 国家公民储备委员会(→NRC 治理档)
    Nsn, // 国家参议会
    Nrp, // 国家众议会
    // B 省级类型(17,3 位,43 省共用,R5 省码区分实例,非盈利)
    Pgv, // 省政府
    Plg, // 省立法院
    Pjd, // 省司法院
    Psp, // 省监察院
    Prc, // 省储会(→PRC 治理档)
    Prb, // 省储行(→PRB 治理档)
    Pdf, // 省防务厅
    Phs, // 省国安厅
    Pcw, // 省民生厅
    Phu, // 省住建厅
    Pag, // 省农业厅
    Pcm, // 省商贸厅
    Pft, // 省财税厅
    Pen, // 省能源厅
    Ptr, // 省交通厅
    Psn, // 省参议会
    Prp, // 省众议会
    // C 市级类型(17,4 位,非盈利)
    Cgov, // 市政府
    Cleg, // 市立法委
    Csup, // 市监察院
    Cjud, // 市司法院
    Cedu, // 市教委
    Cslf, // 市自治委
    Cdef, // 市国防局
    Chsc, // 市国安局
    Ccwf, // 市民生局
    Chud, // 市住建局
    Cagr, // 市农业局
    Ccom, // 市商贸局
    Cfin, // 市财税局
    Cenr, // 市能源局
    Ctrn, // 市交通局
    Creg, // 市注册局
    Cpol, // 市公安局
    // D 镇级类型(14,4 位,非盈利;部门是否启用由市注册局管理员运行期增删)
    Tgov, // 镇政府
    Tcwf, // 镇民生科
    Thud, // 镇住建科
    Tagr, // 镇农业科
    Tfin, // 镇财税科
    Tdef, // 镇国防科
    Thsc, // 镇国安科
    Tcom, // 镇商贸科
    Tenr, // 镇能源科
    Ttrn, // 镇交通科
    Tpol, // 镇公安科
    Tslf, // 镇自治委
    Tsup, // 镇监察院
    Tjud, // 镇司法院
    // E 私权机构(7,4 位)
    Sfgt, // 个体经营(盈利,非法人)
    Sfgp, // 无限合伙(盈利,非法人)
    Sflp, // 有限合伙(盈利,私法人)
    Sfgq, // 股权公司(盈利,私法人)
    Sfgf, // 股份公司(盈利,私法人)
    Sfgy, // 公益组织(非盈利,私法人)
    Sfas, // 注册协会(可盈利可不,私法人)
    // F 教育学校(公私大学=3 位 GUN/SUN / 公私中小初学=4 位;教育委员会 NED/CEDU 不是学校)
    Gun,  // 公立大学(公法人,非盈利,3 位)
    Sun,  // 私立大学(私法人,可盈利可不,3 位;盈利位 0/1 按实例)
    Jun,  // 教会大学(私法人,可盈利可不,3 位;盈利位 0/1 按实例)
    Gsch, // 公立学校(公法人,非盈利,4 位,初学/小学/中学)
    Sfsc, // 私立学校(私法人,可盈利可不,4 位,初学/小学/中学)
    Jsch, // 教会学校(私法人,可盈利可不,4 位,初学/小学/中学)
    // G 个人主体(3,4 位)
    Ctzn, // 公民人(盈利)
    Natp, // 自然人(盈利)
    Smtp, // 智能人(可盈利可不)
    // H 非法人组织(1,4 位)
    Unin, // 非法人组织(挂父级法人,盈利继承父级)
    // I 个人多签(1,4 位,不发号,仅链上/后端分类常量)
    Pmul,
}

impl InstitutionCode {
    /// 返回 CID 号里使用的 3 或 4 字符代码(全大写)。
    pub fn as_code(self) -> &'static str {
        match self {
            Self::Prs => "PRS",
            Self::Fsc => "FSC",
            Self::Fib => "FIB",
            Self::Fss => "FSS",
            Self::Fpr => "FPR",
            Self::Frg => "FRG",
            Self::Mfa => "MFA",
            Self::Mdf => "MDF",
            Self::Mhs => "MHS",
            Self::Mcw => "MCW",
            Self::Mhu => "MHU",
            Self::Mag => "MAG",
            Self::Mcm => "MCM",
            Self::Mft => "MFT",
            Self::Men => "MEN",
            Self::Mtr => "MTR",
            Self::Nlg => "NLG",
            Self::Njd => "NJD",
            Self::Nsp => "NSP",
            Self::Fac => "FAC",
            Self::Fau => "FAU",
            Self::Fiv => "FIV",
            Self::Ned => "NED",
            Self::Nrc => "NRC",
            Self::Nsn => "NSN",
            Self::Nrp => "NRP",
            Self::Pgv => "PGV",
            Self::Plg => "PLG",
            Self::Pjd => "PJD",
            Self::Psp => "PSP",
            Self::Prc => "PRC",
            Self::Prb => "PRB",
            Self::Pdf => "PDF",
            Self::Phs => "PHS",
            Self::Pcw => "PCW",
            Self::Phu => "PHU",
            Self::Pag => "PAG",
            Self::Pcm => "PCM",
            Self::Pft => "PFT",
            Self::Pen => "PEN",
            Self::Ptr => "PTR",
            Self::Psn => "PSN",
            Self::Prp => "PRP",
            Self::Cgov => "CGOV",
            Self::Cleg => "CLEG",
            Self::Csup => "CSUP",
            Self::Cjud => "CJUD",
            Self::Cedu => "CEDU",
            Self::Cslf => "CSLF",
            Self::Cdef => "CDEF",
            Self::Chsc => "CHSC",
            Self::Ccwf => "CCWF",
            Self::Chud => "CHUD",
            Self::Cagr => "CAGR",
            Self::Ccom => "CCOM",
            Self::Cfin => "CFIN",
            Self::Cenr => "CENR",
            Self::Ctrn => "CTRN",
            Self::Creg => "CREG",
            Self::Cpol => "CPOL",
            Self::Tgov => "TGOV",
            Self::Tcwf => "TCWF",
            Self::Thud => "THUD",
            Self::Tagr => "TAGR",
            Self::Tfin => "TFIN",
            Self::Tdef => "TDEF",
            Self::Thsc => "THSC",
            Self::Tcom => "TCOM",
            Self::Tenr => "TENR",
            Self::Ttrn => "TTRN",
            Self::Tpol => "TPOL",
            Self::Tslf => "TSLF",
            Self::Tsup => "TSUP",
            Self::Tjud => "TJUD",
            Self::Sfgt => "SFGT",
            Self::Sfgp => "SFGP",
            Self::Sflp => "SFLP",
            Self::Sfgq => "SFGQ",
            Self::Sfgf => "SFGF",
            Self::Sfgy => "SFGY",
            Self::Sfas => "SFAS",
            Self::Gun => "GUN",
            Self::Sun => "SUN",
            Self::Jun => "JUN",
            Self::Gsch => "GSCH",
            Self::Sfsc => "SFSC",
            Self::Jsch => "JSCH",
            Self::Ctzn => "CTZN",
            Self::Natp => "NATP",
            Self::Smtp => "SMTP",
            Self::Unin => "UNIN",
            Self::Pmul => "PMUL",
        }
    }

    /// 中文显示标签(机构类型,用于 UI / 日志)。
    pub fn label_zh(self) -> &'static str {
        match self {
            Self::Prs => "总统府",
            Self::Fsc => "联邦安全局",
            Self::Fib => "联邦情报局",
            Self::Fss => "联邦特勤局",
            Self::Fpr => "联邦人事局",
            Self::Frg => "联邦注册局",
            Self::Mfa => "外事交流部",
            Self::Mdf => "国家防务部",
            Self::Mhs => "国土安全部",
            Self::Mcw => "公民生活保障部",
            Self::Mhu => "住房与城镇建设部",
            Self::Mag => "农业与农村发展部",
            Self::Mcm => "商务与市场贸易部",
            Self::Mft => "财政与税务部",
            Self::Men => "能源与环保发展部",
            Self::Mtr => "交通运输部",
            Self::Nlg => "国家立法院",
            Self::Njd => "国家司法院",
            Self::Nsp => "国家监察院",
            Self::Fac => "联邦廉政署",
            Self::Fau => "联邦审计署",
            Self::Fiv => "联邦调查署",
            Self::Ned => "国家公民教育委员会",
            Self::Nrc => "国家公民储备委员会",
            Self::Nsn => "国家参议会",
            Self::Nrp => "国家众议会",
            Self::Pgv => "省政府",
            Self::Plg => "省立法院",
            Self::Pjd => "省司法院",
            Self::Psp => "省监察院",
            Self::Prc => "省储会",
            Self::Prb => "省储行",
            Self::Pdf => "省防务厅",
            Self::Phs => "省国安厅",
            Self::Pcw => "省民生厅",
            Self::Phu => "省住建厅",
            Self::Pag => "省农业厅",
            Self::Pcm => "省商贸厅",
            Self::Pft => "省财税厅",
            Self::Pen => "省能源厅",
            Self::Ptr => "省交通厅",
            Self::Psn => "省参议会",
            Self::Prp => "省众议会",
            Self::Cgov => "市政府",
            Self::Cleg => "市立法委",
            Self::Csup => "市监察院",
            Self::Cjud => "市司法院",
            Self::Cedu => "市教委",
            Self::Cslf => "市自治委",
            Self::Cdef => "市国防局",
            Self::Chsc => "市国安局",
            Self::Ccwf => "市民生局",
            Self::Chud => "市住建局",
            Self::Cagr => "市农业局",
            Self::Ccom => "市商贸局",
            Self::Cfin => "市财税局",
            Self::Cenr => "市能源局",
            Self::Ctrn => "市交通局",
            Self::Creg => "市注册局",
            Self::Cpol => "市公安局",
            Self::Tgov => "镇政府",
            Self::Tcwf => "镇民生科",
            Self::Thud => "镇住建科",
            Self::Tagr => "镇农业科",
            Self::Tfin => "镇财税科",
            Self::Tdef => "镇国防科",
            Self::Thsc => "镇国安科",
            Self::Tcom => "镇商贸科",
            Self::Tenr => "镇能源科",
            Self::Ttrn => "镇交通科",
            Self::Tpol => "镇公安科",
            Self::Tslf => "镇自治委",
            Self::Tsup => "镇监察院",
            Self::Tjud => "镇司法院",
            Self::Sfgt => "个体经营",
            Self::Sfgp => "无限合伙",
            Self::Sflp => "有限合伙",
            Self::Sfgq => "股权公司",
            Self::Sfgf => "股份公司",
            Self::Sfgy => "公益组织",
            Self::Sfas => "注册协会",
            Self::Gun => "公立大学",
            Self::Sun => "私立大学",
            Self::Jun => "教会大学",
            Self::Gsch => "公立学校",
            Self::Sfsc => "私立学校",
            Self::Jsch => "教会学校",
            Self::Ctzn => "公民人",
            Self::Natp => "自然人",
            Self::Smtp => "智能人",
            Self::Unin => "非法人组织",
            Self::Pmul => "个人多签",
        }
    }

    /// 全部 86 个机构码(用于 from_str 反查、前端枚举、生成器白名单)。
    pub const ALL: [InstitutionCode; 92] = [
        Self::Prs,
        Self::Fsc,
        Self::Fib,
        Self::Fss,
        Self::Fpr,
        Self::Frg,
        Self::Mfa,
        Self::Mdf,
        Self::Mhs,
        Self::Mcw,
        Self::Mhu,
        Self::Mag,
        Self::Mcm,
        Self::Mft,
        Self::Men,
        Self::Mtr,
        Self::Nlg,
        Self::Njd,
        Self::Nsp,
        Self::Fac,
        Self::Fau,
        Self::Fiv,
        Self::Ned,
        Self::Nrc,
        Self::Nsn,
        Self::Nrp,
        Self::Pgv,
        Self::Plg,
        Self::Pjd,
        Self::Psp,
        Self::Prc,
        Self::Prb,
        Self::Pdf,
        Self::Phs,
        Self::Pcw,
        Self::Phu,
        Self::Pag,
        Self::Pcm,
        Self::Pft,
        Self::Pen,
        Self::Ptr,
        Self::Psn,
        Self::Prp,
        Self::Cgov,
        Self::Cleg,
        Self::Csup,
        Self::Cjud,
        Self::Cedu,
        Self::Cslf,
        Self::Cdef,
        Self::Chsc,
        Self::Ccwf,
        Self::Chud,
        Self::Cagr,
        Self::Ccom,
        Self::Cfin,
        Self::Cenr,
        Self::Ctrn,
        Self::Creg,
        Self::Cpol,
        Self::Tgov,
        Self::Tcwf,
        Self::Thud,
        Self::Tagr,
        Self::Tfin,
        Self::Tdef,
        Self::Thsc,
        Self::Tcom,
        Self::Tenr,
        Self::Ttrn,
        Self::Tpol,
        Self::Tslf,
        Self::Tsup,
        Self::Tjud,
        Self::Sfgt,
        Self::Sfgp,
        Self::Sflp,
        Self::Sfgq,
        Self::Sfgf,
        Self::Sfgy,
        Self::Sfas,
        Self::Gun,
        Self::Sun,
        Self::Jun,
        Self::Gsch,
        Self::Sfsc,
        Self::Jsch,
        Self::Ctzn,
        Self::Natp,
        Self::Smtp,
        Self::Unin,
        Self::Pmul,
    ];

    /// 从字符串解析:接受机构码(如 "NRC")或中文类型标签(如 "国家公民储备委员会")。
    pub fn from_str(s: &str) -> Option<Self> {
        let v = s.trim();
        Self::ALL
            .into_iter()
            .find(|code| code.as_code() == v || code.label_zh() == v)
    }

    /// 机构码字符长度(3 = 国家/省部布局, 4 = 其他布局)。号码段二解析据此分流。
    pub fn code_len(self) -> usize {
        self.as_code().len()
    }

    /// 是否为 3 字符码(国家级单体 / 省级类型)。
    pub fn is_three_char(self) -> bool {
        self.code_len() == 3
    }

    /// 盈利策略(决定号码 M1 / 盈利位的生成与校验)。
    pub fn profit_policy(self) -> ProfitPolicy {
        match self {
            // 私权经营体 + 公民人/自然人:固定盈利
            Self::Sfgt
            | Self::Sfgp
            | Self::Sflp
            | Self::Sfgq
            | Self::Sfgf
            | Self::Ctzn
            | Self::Natp => ProfitPolicy::Profit,
            // 公益组织:固定非盈利
            Self::Sfgy => ProfitPolicy::NonProfit,
            // 注册协会 / 智能人 / 私立大学 / 私立学校:按实例可变
            Self::Sfas | Self::Smtp | Self::Sun | Self::Sfsc | Self::Jun | Self::Jsch => {
                ProfitPolicy::Variable
            }
            // 非法人组织:继承父级法人
            Self::Unin => ProfitPolicy::InheritParent,
            // 其余(国家/省部/市镇公权、个人多签):固定非盈利
            _ => ProfitPolicy::NonProfit,
        }
    }

    /// 个人主体(公民人/自然人/智能人)——不是注册型机构。
    pub fn is_person(self) -> bool {
        matches!(self, Self::Ctzn | Self::Natp | Self::Smtp)
    }

    /// 非法人(个体经营/无限合伙/非法人组织)。
    pub fn is_unincorporated(self) -> bool {
        matches!(self, Self::Sfgt | Self::Sfgp | Self::Unin)
    }

    /// 私法人(有限合伙/股权/股份/公益/协会/私立大学/私立学校)。
    pub fn is_private_legal(self) -> bool {
        matches!(
            self,
            Self::Sflp
                | Self::Sfgq
                | Self::Sfgf
                | Self::Sfgy
                | Self::Sfas
                | Self::Sun
                | Self::Sfsc
                | Self::Jun
                | Self::Jsch
        )
    }

    /// 公法人(国家/省部/市镇公权机构、委员会、公立大学/学校)。
    pub fn is_public_legal(self) -> bool {
        !self.is_person()
            && !self.is_unincorporated()
            && !self.is_private_legal()
            && self != Self::Pmul
    }

    /// 是否教育机构(公私大学/学校)。教育机构走通用注册路径、免 private_type,
    /// 且公权教育机构不受手动公权机构类型限制(走教育流程)。
    pub fn is_education_institution(self) -> bool {
        matches!(
            self,
            Self::Gun | Self::Sun | Self::Jun | Self::Gsch | Self::Sfsc | Self::Jsch
        )
    }

    /// 是否基础教育学校(初学/小学/中学),需要 education_type 级别字段。大学不需要。
    pub fn requires_education_level(self) -> bool {
        matches!(self, Self::Gsch | Self::Sfsc | Self::Jsch)
    }

    /// 机构码所属行政层级。
    /// 仅 4 组公权机构码(国家 26 / 省 17 / 市 17 / 镇 14)有层级;
    /// 私权 / 教育 / 个人 / 非法人 / 个人多签返回 None。
    pub fn admin_level(self) -> Option<AdminLevel> {
        match self {
            // A 国家级单体(26)
            Self::Prs
            | Self::Fsc
            | Self::Fib
            | Self::Fss
            | Self::Fpr
            | Self::Frg
            | Self::Mfa
            | Self::Mdf
            | Self::Mhs
            | Self::Mcw
            | Self::Mhu
            | Self::Mag
            | Self::Mcm
            | Self::Mft
            | Self::Men
            | Self::Mtr
            | Self::Nlg
            | Self::Njd
            | Self::Nsp
            | Self::Fac
            | Self::Fau
            | Self::Fiv
            | Self::Ned
            | Self::Nrc
            | Self::Nsn
            | Self::Nrp => Some(AdminLevel::National),
            // B 省级类型(17)
            Self::Pgv
            | Self::Plg
            | Self::Pjd
            | Self::Psp
            | Self::Prc
            | Self::Prb
            | Self::Pdf
            | Self::Phs
            | Self::Pcw
            | Self::Phu
            | Self::Pag
            | Self::Pcm
            | Self::Pft
            | Self::Pen
            | Self::Ptr
            | Self::Psn
            | Self::Prp => Some(AdminLevel::Province),
            // C 市级类型(17)
            Self::Cgov
            | Self::Cleg
            | Self::Csup
            | Self::Cjud
            | Self::Cedu
            | Self::Cslf
            | Self::Cdef
            | Self::Chsc
            | Self::Ccwf
            | Self::Chud
            | Self::Cagr
            | Self::Ccom
            | Self::Cfin
            | Self::Cenr
            | Self::Ctrn
            | Self::Creg
            | Self::Cpol => Some(AdminLevel::City),
            // D 镇级类型(14)
            Self::Tgov
            | Self::Tcwf
            | Self::Thud
            | Self::Tagr
            | Self::Tfin
            | Self::Tdef
            | Self::Thsc
            | Self::Tcom
            | Self::Tenr
            | Self::Ttrn
            | Self::Tpol
            | Self::Tslf
            | Self::Tsup
            | Self::Tjud => Some(AdminLevel::Town),
            // 其余(私权 / 教育 / 个人 / 非法人 / 个人多签)无行政层级
            _ => None,
        }
    }

    /// 是否市公安局(== CPOL)。
    pub fn is_city_police(self) -> bool {
        self == Self::Cpol
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_codes_are_three_or_four_ascii_upper() {
        for code in InstitutionCode::ALL {
            let c = code.as_code();
            assert!(c.len() == 3 || c.len() == 4, "{c} must be 3 or 4 chars",);
            assert!(
                c.chars().all(|ch| ch.is_ascii_uppercase()),
                "{c} must be ascii uppercase",
            );
        }
    }

    #[test]
    fn codes_are_unique() {
        let mut seen = std::collections::HashSet::new();
        for code in InstitutionCode::ALL {
            assert!(
                seen.insert(code.as_code()),
                "duplicate code {}",
                code.as_code()
            );
        }
        assert_eq!(seen.len(), 92);
    }

    #[test]
    fn four_char_codes_have_letter_at_index_3() {
        // 4 字符码第 4 位必须是字母(号码段二解析靠 index3 数字/字母分流)。
        for code in InstitutionCode::ALL {
            if code.code_len() == 4 {
                let ch = code.as_code().as_bytes()[3] as char;
                assert!(
                    ch.is_ascii_uppercase(),
                    "{} index3 must be letter",
                    code.as_code()
                );
            }
        }
    }

    #[test]
    fn national_and_province_are_three_char_nonprofit() {
        for code in [
            InstitutionCode::Prs,
            InstitutionCode::Nrc,
            InstitutionCode::Prc,
        ] {
            assert!(code.is_three_char());
            assert_eq!(code.profit_policy(), ProfitPolicy::NonProfit);
        }
    }

    #[test]
    fn parse_code_and_label() {
        assert_eq!(InstitutionCode::from_str("NRC"), Some(InstitutionCode::Nrc));
        assert_eq!(
            InstitutionCode::from_str("国家公民储备委员会"),
            Some(InstitutionCode::Nrc)
        );
        assert_eq!(
            InstitutionCode::from_str("SFGQ"),
            Some(InstitutionCode::Sfgq)
        );
        assert_eq!(InstitutionCode::from_str("xyz"), None);
    }

    #[test]
    fn profit_policy_and_category_spot_check() {
        assert_eq!(InstitutionCode::Sfgq.profit_policy(), ProfitPolicy::Profit);
        assert_eq!(
            InstitutionCode::Sfgy.profit_policy(),
            ProfitPolicy::NonProfit
        );
        assert_eq!(
            InstitutionCode::Sfas.profit_policy(),
            ProfitPolicy::Variable
        );
        assert_eq!(
            InstitutionCode::Smtp.profit_policy(),
            ProfitPolicy::Variable
        );
        assert_eq!(
            InstitutionCode::Unin.profit_policy(),
            ProfitPolicy::InheritParent
        );

        assert!(InstitutionCode::Sfgt.is_unincorporated());
        assert!(InstitutionCode::Sfgq.is_private_legal());
        assert!(InstitutionCode::Ctzn.is_person());
        assert!(InstitutionCode::Nrc.is_public_legal());
        assert!(!InstitutionCode::Pmul.is_public_legal() && !InstitutionCode::Pmul.is_person());
    }
}
