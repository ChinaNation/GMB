//! `propose_create_institution` SCALE call-data 编码器(onchina 侧唯一真源)。
//!
//! 中文注释:onchina 只构造**裸 call data**(pallet/call 前缀 + 15 个参数),
//! 不拼签名扩展尾、不提交 extrinsic;冷钱包对 origin 冷签后由 CitizenWallet 提交。
//!
//! **铁律**:参数顺序与 SCALE 类型必须与链端 `public-manage`/`private-manage`
//! `propose_create_{public,private}_institution`(call index 5,A1/A2 后)逐字节一致——
//! 两 pallet call 形态完全相同,仅 pallet 前缀不同(PublicManage=32 / PrivateManage=33),
//! 由 `institution_code` 经 `is_private_legal_code` 派生(机构管理已拆分公权/私权两 pallet)——
//! - `institution_code` 是 `[u8;4]` 裸字节,无长度前缀;
//! - `issuer_main_account` / `signer_pubkey` / `AdminProfile.account` 是 `[u8;32]` 裸字节;
//! - 所有 `Vec<u8>` / `BoundedVec<u8>`(cid_number / cid_full_name / cid_short_name /
//!   register_nonce / signature / issuer_cid_number / scope_*,以及每个 AdminProfile 的
//!   admin_cid_number / name / admin_role)带 `Compact<u32>` 长度前缀;
//! - `accounts` / `admins` 这类项目列表带 `Compact<u32>` 数量前缀;
//! - `admins_len` / `threshold` / `term_start` / `term_end` 是 u32 小端;
//! - `source` 是单字节枚举序号(`AdminSource::Registry` = 1)。
//!
//! `tests` 模块用真实的 `admin_primitives::AdminProfile` 与真实参数类型 `.encode()`
//! 做逐字节交叉校验,杜绝本编码器与链端 SCALE 静默漂移。

use parity_scale_codec::{Compact, Encode};

/// PublicManage pallet 在 runtime construct_runtime 中的索引(公权机构生命周期)。
pub const PUBLIC_MANAGE_PALLET_INDEX: u8 = 32;
/// PrivateManage pallet 在 runtime construct_runtime 中的索引(私权机构生命周期)。
pub const PRIVATE_MANAGE_PALLET_INDEX: u8 = 33;
/// `propose_create_{public,private}_institution` 的 call index(两 pallet 同为 5)。
pub const PROPOSE_CREATE_INSTITUTION_CALL_INDEX: u8 = 5;

/// 按机构码派生机构创建调用的目标 pallet 索引:私权法人码→PrivateManage,否则→PublicManage。
///
/// 中文注释:与链端 `is_private_legal_code` 单源一致;前缀由 call data 内的 institution_code
/// 派生,杜绝调用方手填 pallet 索引导致漂移。
pub fn create_institution_pallet_index(institution_code: &[u8; 4]) -> u8 {
    if primitives::cid::code::is_private_legal_code(institution_code) {
        PRIVATE_MANAGE_PALLET_INDEX
    } else {
        PUBLIC_MANAGE_PALLET_INDEX
    }
}

/// `AdminSource` 枚举序号(必须与 admin-primitives 的变体顺序一致)。
/// Genesis=0 / Registry=1 / InternalVote=2 / MutualElection=3 / PopularElection=4。
/// 全变体保留以锁死链端枚举序号(交叉校验测试逐个比对),生产路径只用 Registry。
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdminSourceTag {
    Genesis = 0,
    Registry = 1,
    InternalVote = 2,
    MutualElection = 3,
    PopularElection = 4,
}

impl AdminSourceTag {
    fn index(self) -> u8 {
        self as u8
    }
}

/// 单个创建账户项(链端 `InstitutionInitialAccount`)。
/// `account_name` 是进链 name 字段;`amount` 是初始余额(分,u128)。
#[derive(Debug, Clone)]
pub struct InitialAccountArg {
    pub account_name: String,
    pub amount: u128,
}

/// 单个管理员资料项(链端 `AdminProfile<AccountId>`)。
#[derive(Debug, Clone)]
pub struct AdminProfileArg {
    /// 管理员密码学账户,32 字节裸编码。
    pub account: [u8; 32],
    /// 实名锚:注册局签发的 CID 号。
    pub admin_cid_number: Vec<u8>,
    /// 姓名快照(来自注册局公民记录)。
    pub name: Vec<u8>,
    /// 对外法定职务。
    pub admin_role: Vec<u8>,
    /// 任期开始(天数自纪元;无任期填 0)。
    pub term_start: u32,
    /// 任期结束(天数自纪元;无任期填 0)。
    pub term_end: u32,
    /// 职务/任期来源。
    pub source: AdminSourceTag,
}

/// `propose_create_institution` 的完整参数集合。
#[derive(Debug, Clone)]
pub struct ProposeCreateInstitutionArgs {
    pub cid_number: Vec<u8>,
    pub cid_full_name: Vec<u8>,
    /// 私权机构留空(链端按 A1 存空);公权机构填简称。
    pub cid_short_name: Vec<u8>,
    pub accounts: Vec<InitialAccountArg>,
    pub institution_code: [u8; 4],
    pub admins_len: u32,
    pub admins: Vec<AdminProfileArg>,
    pub threshold: u32,
    pub register_nonce: Vec<u8>,
    pub signature: Vec<u8>,
    pub issuer_cid_number: Vec<u8>,
    pub issuer_main_account: [u8; 32],
    pub signer_pubkey: [u8; 32],
    pub scope_province_name: Vec<u8>,
    pub scope_city_name: Vec<u8>,
}

/// 把单个 `AdminProfile` 追加进输出缓冲(字段顺序锁死链端结构)。
fn encode_admin_profile(out: &mut Vec<u8>, profile: &AdminProfileArg) {
    out.extend_from_slice(&profile.account); // account: [u8;32] 裸字节
    out.extend(Compact(profile.admin_cid_number.len() as u32).encode());
    out.extend_from_slice(&profile.admin_cid_number);
    out.extend(Compact(profile.name.len() as u32).encode());
    out.extend_from_slice(&profile.name);
    out.extend(Compact(profile.admin_role.len() as u32).encode());
    out.extend_from_slice(&profile.admin_role);
    out.extend(profile.term_start.to_le_bytes()); // u32 小端
    out.extend(profile.term_end.to_le_bytes()); // u32 小端
    out.push(profile.source.index()); // 枚举单字节序号
}

/// QR_V1 链交易动作码:`a = (pallet_index << 8) | call_index`。
///
/// 中文注释:扫码端(CitizenWallet)按 `b.a` 路由 decoder,链交易统一用本公式
/// (见 `memory/01-architecture/qr/qr-action-registry.md`「链交易动作码」)。
/// 禁止再为链交易另发明扁平小整数动作码(会与非链动作码 1..7 冲突)。
pub const fn chain_action_code(pallet_index: u8, call_index: u8) -> u16 {
    ((pallet_index as u16) << 8) | (call_index as u16)
}

/// 一条链上调用的 QR 动作码 + 裸 SCALE call data。
///
/// 中文注释:`action`(b.a)与 `call_data`(b.d)由同一 pallet/call 派生,杜绝两者漂移。
pub struct ChainCall {
    pub action: u16,
    pub call_data: Vec<u8>,
}

/// 编码完整 `propose_create_{public,private}_institution` 裸 call data。
///
/// 输出 = `[pallet, 0x05]` + 15 个参数(顺序与链端逐字节一致);pallet 由 institution_code
/// 经 `create_institution_pallet_index` 派生(公权 32→动作码 0x2005 / 私权 33→0x2105)。
pub fn encode_propose_create_institution(args: &ProposeCreateInstitutionArgs) -> ChainCall {
    let pallet_index = create_institution_pallet_index(&args.institution_code);
    let mut out = Vec::new();
    out.push(pallet_index);
    out.push(PROPOSE_CREATE_INSTITUTION_CALL_INDEX);

    // cid_number: BoundedVec<u8>
    out.extend(Compact(args.cid_number.len() as u32).encode());
    out.extend_from_slice(&args.cid_number);

    // cid_full_name: BoundedVec<u8>
    out.extend(Compact(args.cid_full_name.len() as u32).encode());
    out.extend_from_slice(&args.cid_full_name);

    // cid_short_name: BoundedVec<u8>
    out.extend(Compact(args.cid_short_name.len() as u32).encode());
    out.extend_from_slice(&args.cid_short_name);

    // accounts: BoundedVec<InstitutionInitialAccount> = Compact<N> + N × (name + amount)
    out.extend(Compact(args.accounts.len() as u32).encode());
    for account in &args.accounts {
        out.extend(Compact(account.account_name.len() as u32).encode());
        out.extend_from_slice(account.account_name.as_bytes());
        out.extend(account.amount.to_le_bytes()); // u128 小端
    }

    // institution_code: [u8;4] 裸字节(无长度前缀)
    out.extend_from_slice(&args.institution_code);

    // admins_len: u32 小端
    out.extend(args.admins_len.to_le_bytes());

    // admins: BoundedVec<AdminProfile> = Compact<N> + N × AdminProfile
    out.extend(Compact(args.admins.len() as u32).encode());
    for profile in &args.admins {
        encode_admin_profile(&mut out, profile);
    }

    // threshold: u32 小端
    out.extend(args.threshold.to_le_bytes());

    // register_nonce: BoundedVec<u8>
    out.extend(Compact(args.register_nonce.len() as u32).encode());
    out.extend_from_slice(&args.register_nonce);

    // signature: BoundedVec<u8>
    out.extend(Compact(args.signature.len() as u32).encode());
    out.extend_from_slice(&args.signature);

    // issuer_cid_number: Vec<u8>
    out.extend(Compact(args.issuer_cid_number.len() as u32).encode());
    out.extend_from_slice(&args.issuer_cid_number);

    // issuer_main_account: AccountId(32 字节裸)
    out.extend_from_slice(&args.issuer_main_account);

    // signer_pubkey: [u8;32] 裸字节
    out.extend_from_slice(&args.signer_pubkey);

    // scope_province_name: Vec<u8>
    out.extend(Compact(args.scope_province_name.len() as u32).encode());
    out.extend_from_slice(&args.scope_province_name);

    // scope_city_name: Vec<u8>
    out.extend(Compact(args.scope_city_name.len() as u32).encode());
    out.extend_from_slice(&args.scope_city_name);

    ChainCall {
        action: chain_action_code(pallet_index, PROPOSE_CREATE_INSTITUTION_CALL_INDEX),
        call_data: out,
    }
}

/// Admin pallet 在 runtime construct_runtime 中的索引。
pub const GENESIS_ADMINS_PALLET_INDEX: u8 = 12;
/// `federal_set_city_registry_admins` 的 call index(genesis pallet,联邦特权直设)。
pub const FEDERAL_SET_CITY_REGISTRY_ADMINS_CALL_INDEX: u8 = 1;
/// `propose_federal_registry_province_admin_set_change` 的 call index。
pub const PROPOSE_FRG_PROVINCE_ADMIN_SET_CHANGE_CALL_INDEX: u8 = 2;

/// 管理员集合变更类调用参数。
///
/// `federal_set_city_registry_admins` 与 `propose_admin_set_change` 共用此形态:
/// `(institution_code, account, admins: BoundedVec<AdminProfile>, threshold)`。
#[derive(Debug, Clone)]
pub struct AdminSetCallArgs {
    pub pallet_index: u8,
    pub call_index: u8,
    pub institution_code: [u8; 4],
    /// 机构主账户(= AdminAccounts 键)。
    pub account: [u8; 32],
    pub admins: Vec<AdminProfileArg>,
    pub threshold: u32,
}

/// 联邦注册局省级管理员组更换调用参数。
///
/// 中文注释:链端 call 形态为 `(province_code: [u8;2], admins, threshold)`；
/// 省级组账户由链端按省码派生,因此 call data 不再携带 FRG 主账户或 institution_code。
#[derive(Debug, Clone)]
pub struct FederalRegistryProvinceAdminSetCallArgs {
    pub pallet_index: u8,
    pub call_index: u8,
    pub province_code: [u8; 2],
    pub admins: Vec<AdminProfileArg>,
    pub threshold: u32,
}

/// 编码管理员集合变更类裸 call data。
///
/// 输出 = `[pallet, call]` + `institution_code[u8;4]` + `account[u8;32]`
///       + `admins`(Compact<N> + N×AdminProfile) + `threshold`(u32 小端);
/// 动作码 = `(pallet_index<<8)|call_index`(CREG=`0x0c01`,FRG=`0x0c00`)。
pub fn encode_admin_set_call(args: &AdminSetCallArgs) -> ChainCall {
    let mut out = Vec::new();
    out.push(args.pallet_index);
    out.push(args.call_index);
    out.extend_from_slice(&args.institution_code); // [u8;4] 裸
    out.extend_from_slice(&args.account); // [u8;32] 裸
    out.extend(Compact(args.admins.len() as u32).encode());
    for profile in &args.admins {
        encode_admin_profile(&mut out, profile);
    }
    out.extend(args.threshold.to_le_bytes()); // u32 小端
    ChainCall {
        action: chain_action_code(args.pallet_index, args.call_index),
        call_data: out,
    }
}

/// 编码联邦注册局省级管理员组更换裸 call data。
///
/// 输出 = `[pallet, call]` + `province_code[u8;2]`
///       + `admins`(Compact<N> + N×AdminProfile) + `threshold`(u32 小端);
/// 动作码 = `(pallet_index<<8)|call_index`(FRG 省级组=`0x0c02`)。
pub fn encode_federal_registry_province_admin_set_call(
    args: &FederalRegistryProvinceAdminSetCallArgs,
) -> ChainCall {
    let mut out = Vec::new();
    out.push(args.pallet_index);
    out.push(args.call_index);
    out.extend_from_slice(&args.province_code);
    out.extend(Compact(args.admins.len() as u32).encode());
    for profile in &args.admins {
        encode_admin_profile(&mut out, profile);
    }
    out.extend(args.threshold.to_le_bytes());
    ChainCall {
        action: chain_action_code(args.pallet_index, args.call_index),
        call_data: out,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use admin_primitives::{AdminProfile, AdminSource};
    use frame_support::BoundedVec;

    /// 用真实链端 `AdminProfile<[u8;32]>` 构造并 `.encode()`,与本编码器的
    /// `encode_admin_profile` 输出逐字节比对。`AdminProfile` 对 AccountId 泛型,
    /// 用 `[u8;32]` 作 AccountId 即可复用链端 SCALE,无需 sp-runtime AccountId32。
    fn real_admin_profile(arg: &AdminProfileArg) -> AdminProfile<[u8; 32]> {
        AdminProfile {
            account: arg.account,
            admin_cid_number: BoundedVec::try_from(arg.admin_cid_number.clone())
                .expect("admin_cid_number within bound"),
            name: BoundedVec::try_from(arg.name.clone()).expect("name within bound"),
            admin_role: BoundedVec::try_from(arg.admin_role.clone())
                .expect("admin_role within bound"),
            term_start: arg.term_start,
            term_end: arg.term_end,
            source: match arg.source {
                AdminSourceTag::Genesis => AdminSource::Genesis,
                AdminSourceTag::Registry => AdminSource::Registry,
                AdminSourceTag::InternalVote => AdminSource::InternalVote,
                AdminSourceTag::MutualElection => AdminSource::MutualElection,
                AdminSourceTag::PopularElection => AdminSource::PopularElection,
            },
        }
    }

    fn sample_admin(seed: u8) -> AdminProfileArg {
        AdminProfileArg {
            account: [seed; 32],
            admin_cid_number: format!("CID{seed:03}").into_bytes(),
            name: "张三".as_bytes().to_vec(),
            admin_role: "主任".as_bytes().to_vec(),
            term_start: 19_700 + seed as u32,
            term_end: 28_900 + seed as u32,
            source: AdminSourceTag::Registry,
        }
    }

    /// `AdminSourceTag` 序号必须与链端 `AdminSource` 变体 `.encode()` 单字节一致。
    #[test]
    fn admin_source_tag_matches_runtime_enum_index() {
        let cases = [
            (AdminSourceTag::Genesis, AdminSource::Genesis),
            (AdminSourceTag::Registry, AdminSource::Registry),
            (AdminSourceTag::InternalVote, AdminSource::InternalVote),
            (AdminSourceTag::MutualElection, AdminSource::MutualElection),
            (
                AdminSourceTag::PopularElection,
                AdminSource::PopularElection,
            ),
        ];
        for (tag, real) in cases {
            assert_eq!(
                vec![tag.index()],
                real.encode(),
                "AdminSource 序号漂移: {tag:?}"
            );
        }
    }

    /// 管理员集合变更 call data 必须与链端 `(InstitutionCode, AccountId, Vec<AdminProfile>, u32)`
    /// tuple `.encode()` 逐字节一致 + 前缀正确(federal_set_city_registry_admins = [12,1])。
    #[test]
    fn admin_set_call_encoding_matches_runtime_tuple_and_prefix() {
        let args = AdminSetCallArgs {
            pallet_index: GENESIS_ADMINS_PALLET_INDEX,
            call_index: FEDERAL_SET_CITY_REGISTRY_ADMINS_CALL_INDEX,
            institution_code: *b"CREG",
            account: [0x42; 32],
            admins: vec![sample_admin(1), sample_admin(2)],
            threshold: 2,
        };
        let chain = encode_admin_set_call(&args);
        let manual = chain.call_data;
        assert_eq!(
            &manual[..2],
            &[
                GENESIS_ADMINS_PALLET_INDEX,
                FEDERAL_SET_CITY_REGISTRY_ADMINS_CALL_INDEX
            ]
        );
        // 动作码 = (pallet<<8)|call;federal_set = 0x0c01。
        assert_eq!(chain.action, 0x0c01, "federal_set 动作码必须 = (12<<8)|1");
        assert_eq!(
            chain.action,
            chain_action_code(
                GENESIS_ADMINS_PALLET_INDEX,
                FEDERAL_SET_CITY_REGISTRY_ADMINS_CALL_INDEX
            )
        );
        let real_admins: Vec<AdminProfile<[u8; 32]>> =
            args.admins.iter().map(real_admin_profile).collect();
        let real = (
            args.institution_code,
            args.account,
            real_admins,
            args.threshold,
        )
            .encode();
        assert_eq!(&manual[2..], real.as_slice());
    }

    /// FRG 省级组更换 call data 必须与链端
    /// `(ProvinceCode, Vec<AdminProfile>, u32)` tuple `.encode()` 逐字节一致。
    #[test]
    fn federal_registry_province_admin_set_call_encoding_matches_runtime_tuple_and_prefix() {
        let args = FederalRegistryProvinceAdminSetCallArgs {
            pallet_index: GENESIS_ADMINS_PALLET_INDEX,
            call_index: PROPOSE_FRG_PROVINCE_ADMIN_SET_CHANGE_CALL_INDEX,
            province_code: *b"GZ",
            admins: vec![
                sample_admin(1),
                sample_admin(2),
                sample_admin(3),
                sample_admin(4),
                sample_admin(5),
            ],
            threshold: 3,
        };
        let chain = encode_federal_registry_province_admin_set_call(&args);
        let manual = chain.call_data;
        assert_eq!(
            &manual[..2],
            &[
                GENESIS_ADMINS_PALLET_INDEX,
                PROPOSE_FRG_PROVINCE_ADMIN_SET_CHANGE_CALL_INDEX
            ]
        );
        assert_eq!(chain.action, 0x0c02, "FRG 省级组动作码必须 = (12<<8)|2");
        let real_admins: Vec<AdminProfile<[u8; 32]>> =
            args.admins.iter().map(real_admin_profile).collect();
        let real = (args.province_code, real_admins, args.threshold).encode();
        assert_eq!(&manual[2..], real.as_slice());
    }

    /// 单个 AdminProfile 编码必须与链端真实类型 `.encode()` 逐字节一致。
    #[test]
    fn admin_profile_encoding_matches_runtime_type() {
        let arg = sample_admin(7);
        let mut manual = Vec::new();
        encode_admin_profile(&mut manual, &arg);
        let golden = real_admin_profile(&arg).encode();
        assert_eq!(manual, golden, "AdminProfile SCALE 漂移");
    }

    /// AdminProfile 列表(BoundedVec<AdminProfile>)整体编码与链端一致(含 Compact 数量前缀)。
    #[test]
    fn admin_profile_vec_encoding_matches_runtime_type() {
        let args = vec![sample_admin(1), sample_admin(2), sample_admin(3)];

        // 本编码器:Compact<N> + N × AdminProfile。
        let mut manual = Vec::new();
        manual.extend(Compact(args.len() as u32).encode());
        for arg in &args {
            encode_admin_profile(&mut manual, arg);
        }

        // 链端真实类型:Vec<AdminProfile<[u8;32]>>(SCALE 与 BoundedVec 同布局)。
        let real: Vec<AdminProfile<[u8; 32]>> = args.iter().map(real_admin_profile).collect();
        let golden = real.encode();

        assert_eq!(manual, golden, "AdminProfile 列表 SCALE 漂移");
    }

    /// 完整参数元组按链端参数顺序用真实类型 `.encode()` 拼接,与本编码器去掉 [0x11,0x05]
    /// 前缀后的输出逐字节一致。这把 cid_short_name 插入位置、accounts(name+amount)、
    /// institution_code 裸 4 字节、admins(AdminProfile)、issuer/signer 裸 32 字节、
    /// 各 Vec<u8> 的 Compact 前缀全部锁死到链端 SCALE。
    #[test]
    fn full_args_encoding_matches_runtime_tuple_and_prefix() {
        let args = ProposeCreateInstitutionArgs {
            cid_number: b"110000200001011234".to_vec(),
            cid_full_name: "北京市某某有限公司".as_bytes().to_vec(),
            cid_short_name: "某某公司".as_bytes().to_vec(),
            accounts: vec![
                InitialAccountArg {
                    account_name: "主账户".to_string(),
                    amount: 1_000_00,
                },
                InitialAccountArg {
                    account_name: "费用账户".to_string(),
                    amount: 250_50,
                },
            ],
            institution_code: *b"SFLP",
            admins_len: 2,
            admins: vec![sample_admin(11), sample_admin(22)],
            threshold: 2,
            register_nonce: b"nonce-xyz".to_vec(),
            signature: vec![0xABu8; 64],
            issuer_cid_number: b"FRG000000000000001".to_vec(),
            issuer_main_account: [0x33u8; 32],
            signer_pubkey: [0x44u8; 32],
            scope_province_name: "北京市".as_bytes().to_vec(),
            scope_city_name: "北京市".as_bytes().to_vec(),
        };

        let chain = encode_propose_create_institution(&args);
        let manual = chain.call_data;

        // SFLP 是私权法人码 → PrivateManage(33) call 5,前缀 [33,5]、动作码 0x2105。
        assert_eq!(
            &manual[..2],
            &[
                PRIVATE_MANAGE_PALLET_INDEX,
                PROPOSE_CREATE_INSTITUTION_CALL_INDEX
            ],
            "私权机构创建前缀必须是 [33,5]"
        );
        assert_eq!(
            chain.action, 0x2105,
            "私权机构创建动作码必须 = (33<<8)|5 = 0x2105"
        );

        // 用链端真实类型按参数顺序逐个 .encode() 拼接出 golden(不含前缀)。
        let real_accounts: Vec<(Vec<u8>, u128)> = args
            .accounts
            .iter()
            .map(|a| (a.account_name.clone().into_bytes(), a.amount))
            .collect();
        let real_admins: Vec<AdminProfile<[u8; 32]>> =
            args.admins.iter().map(real_admin_profile).collect();

        let mut golden = Vec::new();
        golden.extend(args.cid_number.encode());
        golden.extend(args.cid_full_name.encode());
        golden.extend(args.cid_short_name.encode());
        golden.extend(real_accounts.encode());
        golden.extend(args.institution_code.encode()); // [u8;4] 裸 4 字节
        golden.extend(args.admins_len.encode());
        golden.extend(real_admins.encode());
        golden.extend(args.threshold.encode());
        golden.extend(args.register_nonce.encode());
        golden.extend(args.signature.encode());
        golden.extend(args.issuer_cid_number.encode());
        golden.extend(args.issuer_main_account.encode()); // [u8;32] 裸 32 字节
        golden.extend(args.signer_pubkey.encode());
        golden.extend(args.scope_province_name.encode());
        golden.extend(args.scope_city_name.encode());

        assert_eq!(&manual[2..], &golden[..], "完整参数 SCALE 与链端类型漂移");

        // 公权码分支:同一编码器换公权机构码必须路由到 PublicManage(32) call 5、动作码 0x2005。
        let mut public_args = args;
        public_args.institution_code = *b"PRC\0"; // 省公民储备银行(公权法人)
        assert!(!primitives::cid::code::is_private_legal_code(
            &public_args.institution_code
        ));
        let public_chain = encode_propose_create_institution(&public_args);
        assert_eq!(
            &public_chain.call_data[..2],
            &[
                PUBLIC_MANAGE_PALLET_INDEX,
                PROPOSE_CREATE_INSTITUTION_CALL_INDEX
            ],
            "公权机构创建前缀必须是 [32,5]"
        );
        assert_eq!(
            public_chain.action, 0x2005,
            "公权机构创建动作码必须 = (32<<8)|5 = 0x2005"
        );
    }
}
