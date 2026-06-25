//! # 立法院模块 (legislation-yuan)
//!
//! 法律结构化上链 + 修法一律走投票引擎(ADR-027)。本 pallet 是「业务壳」:
//! 只承载法律数据(Law / LawVersion)、状态机、提案入口(立法/修法/废法)、
//! 投票通过回调写入、不可修改条款硬拒与查询;表决规则、计票、两院顺序、强制公投
//! 全部归属投票引擎 `legislation-vote` sub-pallet。
//!
//! 解耦:`Config::LegislationVoteEngine` 注入立法投票引擎(runtime 装配为 `LegislationVote`);
//! 业务壳通过它创建立法投票提案,投票终态经核心 `LegislationVoteResultCallback` 回调写回本壳。

#![cfg_attr(not(feature = "std"), no_std)]

pub mod types;
pub mod weights;

pub use pallet::*;
pub use types::{LawAction, LawStatus, Tier, VoteType};

/// 模块标识前缀,用于在 votingengine `ProposalData` 中区分本模块提案,防止跨模块误解码。
pub const MODULE_TAG: &[u8] = b"leg-yuan";

/// 法律全文大对象类型标记(写入 votingengine `ProposalObject`)。
pub const PROPOSAL_OBJECT_KIND_LAW_TEXT: u8 = 2;

/// 内置公民宪法(章>节>条>款)SCALE 字节,由 `scripts/parse_constitution.py` 从原始 HTML 生成。
/// 宪法唯一真源 = 本模块链上法律(创世注入);原始 `CitizenConstitution.html` 迁移后已废弃删除。
pub const CONSTITUTION_SCALE: &[u8] = include_bytes!("constitution.scale");

/// 国家立法院机构码(立法权最高机构,宪法 houses[0])。
pub const NATIONAL_LEGISLATURE_CODE: primitives::code::InstitutionCode = *b"NLG\0";

/// 不可修改条款 manifest 的最大容量(清单现 8 条,留余量)。
pub const MAX_IMMUTABLE_ARTICLES: u32 = 32;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use crate::weights::WeightInfo;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use primitives::china::china_lf::CHINA_LF;
    use primitives::code::InstitutionCode;
    use primitives::count_const::IMMUTABLE_CONSTITUTION_ARTICLES;
    use sp_runtime::DispatchError;
    use votingengine::{InternalAdminProvider, LegislationVoteEngine, ProposalExecutionOutcome};

    // 受 Config 常量约束的有界字符串别名。
    pub type TitleOf<T> = BoundedVec<u8, <T as Config>::MaxTitleLen>;
    pub type TextOf<T> = BoundedVec<u8, <T as Config>::MaxTextLen>;

    /// 院序列别名:`[(机构码, 机构账户), ...]`,发起院在前、终审院在后(ADR-027,提案携带)。
    /// 单院(市立法会)= 1 项;两院(国家/省立法院)= `[众议会, 参议会]`;教委会模式 = `[教委会, 参议会]`。
    pub type HousesOf<T> = BoundedVec<
        (InstitutionCode, <T as frame_system::Config>::AccountId),
        ConstU32<{ votingengine::types::MAX_LEGISLATION_HOUSES }>,
    >;

    /// 法律内容统一结构:章 > 节 > 条 > 款(ADR-027)。
    /// 章/节/条做目录,条款做正文;所有法律统一此结构(章/节/条必有,款可空)。
    /// 宪法双语(`_en` 全填),其他法律单语(`_en` 为 None)。

    /// 条文款(第 N 款,正文)。
    #[derive(
        Encode,
        Decode,
        DecodeWithMemTracking,
        CloneNoBound,
        PartialEqNoBound,
        EqNoBound,
        RuntimeDebugNoBound,
        TypeInfo,
        MaxEncodedLen,
    )]
    #[scale_info(skip_type_params(T))]
    pub struct Clause<T: Config> {
        /// 款序号(数字)
        pub number: u32,
        /// 款正文(中文)
        pub text: TextOf<T>,
        /// 款正文(英文;宪法填,普通法律 None)
        pub text_en: Option<TextOf<T>>,
    }

    /// 法律条文(第 N 条,目录叶 + 正文)。`number` 全法唯一连续,用于不可修改条款比对。
    #[derive(
        Encode,
        Decode,
        DecodeWithMemTracking,
        CloneNoBound,
        PartialEqNoBound,
        EqNoBound,
        RuntimeDebugNoBound,
        TypeInfo,
        MaxEncodedLen,
    )]
    #[scale_info(skip_type_params(T))]
    pub struct Article<T: Config> {
        /// 条序号(数字,全法唯一连续),如第一条 → 1
        pub number: u32,
        /// 条标题(中文,如「第一条」)
        pub title: TitleOf<T>,
        /// 条标题(英文)
        pub title_en: Option<TitleOf<T>>,
        /// 条正文(中文,必填;无款的条放此,有款的条作总述)
        pub body: TextOf<T>,
        /// 条正文(英文)
        pub body_en: Option<TextOf<T>>,
        /// 条下属各款(可空,不是每条都有款)
        pub clauses: BoundedVec<Clause<T>, <T as Config>::MaxClausesPerArticle>,
    }

    /// 法律节(第 N 节,目录)。
    #[derive(
        Encode,
        Decode,
        DecodeWithMemTracking,
        CloneNoBound,
        PartialEqNoBound,
        EqNoBound,
        RuntimeDebugNoBound,
        TypeInfo,
        MaxEncodedLen,
    )]
    #[scale_info(skip_type_params(T))]
    pub struct Section<T: Config> {
        /// 节序号(数字)
        pub number: u32,
        /// 节标题(中文)
        pub title: TitleOf<T>,
        /// 节标题(英文)
        pub title_en: Option<TitleOf<T>>,
        /// 节下属各条
        pub articles: BoundedVec<Article<T>, <T as Config>::MaxArticlesPerSection>,
    }

    /// 法律章(第 N 章,目录)。
    #[derive(
        Encode,
        Decode,
        DecodeWithMemTracking,
        CloneNoBound,
        PartialEqNoBound,
        EqNoBound,
        RuntimeDebugNoBound,
        TypeInfo,
        MaxEncodedLen,
    )]
    #[scale_info(skip_type_params(T))]
    pub struct Chapter<T: Config> {
        /// 章序号(数字)
        pub number: u32,
        /// 章标题(中文)
        pub title: TitleOf<T>,
        /// 章标题(英文)
        pub title_en: Option<TitleOf<T>>,
        /// 章下属各节
        pub sections: BoundedVec<Section<T>, <T as Config>::MaxSectionsPerChapter>,
    }

    /// 法律全文章节别名:章 > 节 > 条 > 款。
    pub type ChaptersOf<T> = BoundedVec<Chapter<T>, <T as Config>::MaxChaptersPerLaw>;

    /// 法律主体记录(状态 + 当前版本号 + 归属立法机构)。
    #[derive(
        Encode,
        Decode,
        DecodeWithMemTracking,
        CloneNoBound,
        PartialEqNoBound,
        EqNoBound,
        RuntimeDebugNoBound,
        TypeInfo,
        MaxEncodedLen,
    )]
    #[scale_info(skip_type_params(T))]
    pub struct Law<T: Config> {
        pub law_id: u64,
        pub tier: Tier,
        /// 行政区 code(0 = 全国;省/市用 china.sqlite code,遵守 ADR-021)
        pub scope_code: u32,
        /// 归属立法机构院序列(houses[0] = 发起院,其 admins = 现任议员/委员)。
        /// 单院(市立法会)= 1 项;两院(国家/省立法院)= [众议会, 参议会]。
        pub houses: HousesOf<T>,
        pub current_version: u32,
        pub status: LawStatus,
    }

    /// 法律单版本(整部全文快照)。
    #[derive(
        Encode,
        Decode,
        DecodeWithMemTracking,
        CloneNoBound,
        PartialEqNoBound,
        EqNoBound,
        RuntimeDebugNoBound,
        TypeInfo,
        MaxEncodedLen,
    )]
    #[scale_info(skip_type_params(T))]
    pub struct LawVersion<T: Config> {
        pub law_id: u64,
        pub version: u32,
        pub title: TitleOf<T>,
        pub title_en: Option<TitleOf<T>>,
        /// 法律全文:章 > 节 > 条 > 款
        pub chapters: ChaptersOf<T>,
        /// blake2_256(规范化 SCALE 全文),完整性 + 公投/签名绑定
        pub content_hash: [u8; 32],
        pub vote_type: VoteType,
        pub proposal_id: u64,
        pub published_at: BlockNumberFor<T>,
        pub effective_at: BlockNumberFor<T>,
    }

    /// 提案摘要:序列化后(带 MODULE_TAG 前缀)存入 votingengine `ProposalData`;
    /// 法律全文条文单独写入 `ProposalObject`,通过回调读回组装新版本。
    #[derive(
        Encode,
        Decode,
        DecodeWithMemTracking,
        CloneNoBound,
        PartialEqNoBound,
        EqNoBound,
        RuntimeDebugNoBound,
        TypeInfo,
        MaxEncodedLen,
    )]
    #[scale_info(skip_type_params(T))]
    pub struct LawProposalSummary<T: Config> {
        pub action: LawAction,
        /// Enact 时为 0(执行时分配);Amend/Repeal 为目标 law_id
        pub law_id: u64,
        pub tier: Tier,
        pub scope_code: u32,
        /// 归属立法机构院序列(houses[0] = 发起院)。
        pub houses: HousesOf<T>,
        pub vote_type: VoteType,
        pub title: TitleOf<T>,
        pub title_en: Option<TitleOf<T>>,
        pub content_hash: [u8; 32],
        pub effective_at: BlockNumberFor<T>,
    }

    #[pallet::config]
    pub trait Config: frame_system::Config + votingengine::Config {
        #[allow(deprecated)]
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// 立法投票引擎(runtime 装配为 `LegislationVote`),业务壳通过它创建立法投票提案。
        type LegislationVoteEngine: LegislationVoteEngine<Self::AccountId>;

        #[pallet::constant]
        type MaxTitleLen: Get<u32>;
        /// 条/款正文最大字节(body 与 clause text 共用)。
        #[pallet::constant]
        type MaxTextLen: Get<u32>;
        #[pallet::constant]
        type MaxClausesPerArticle: Get<u32>;
        #[pallet::constant]
        type MaxArticlesPerSection: Get<u32>;
        #[pallet::constant]
        type MaxSectionsPerChapter: Get<u32>;
        #[pallet::constant]
        type MaxChaptersPerLaw: Get<u32>;
        #[pallet::constant]
        type MaxLawsPerScope: Get<u32>;
        #[pallet::constant]
        type MaxActivationsPerBlock: Get<u32>;

        type WeightInfo: crate::weights::WeightInfo;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// 不可修改条款 manifest(ADR-027 §6.1,L3 创世锚)。
    /// `article_numbers` 与 `article_hashes` 平行:条号 + 该条规范 SCALE 的 blake2_256 摘要。
    /// 仅 `genesis_build` 写入,创世后永不可变;节点启动据此与二进制清单/创世条文交叉校验。
    #[derive(Clone, PartialEq, Eq, Encode, Decode, MaxEncodedLen, TypeInfo, RuntimeDebug)]
    pub struct ImmutableManifest {
        pub article_numbers: BoundedVec<u32, ConstU32<{ MAX_IMMUTABLE_ARTICLES }>>,
        pub article_hashes: BoundedVec<[u8; 32], ConstU32<{ MAX_IMMUTABLE_ARTICLES }>>,
    }

    /// 法律自增 ID。
    #[pallet::storage]
    pub type NextLawId<T> = StorageValue<_, u64, ValueQuery>;

    /// 法律主表:law_id → Law。
    #[pallet::storage]
    pub type Laws<T: Config> = StorageMap<_, Blake2_128Concat, u64, Law<T>, OptionQuery>;

    /// 法律全版本历史:(law_id, version) → LawVersion。
    #[pallet::storage]
    pub type LawVersions<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        u64,
        Blake2_128Concat,
        u32,
        LawVersion<T>,
        OptionQuery,
    >;

    /// 列表索引:(tier, scope_code) → [law_id]。供客户端按层级/行政区列出法律。
    #[pallet::storage]
    pub type LawsByScope<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        Tier,
        Blake2_128Concat,
        u32,
        BoundedVec<u64, <T as Config>::MaxLawsPerScope>,
        ValueQuery,
    >;

    /// 生效调度:effective_block → [(law_id, version)]。on_initialize 到点翻 Effective。
    #[pallet::storage]
    pub type PendingActivation<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        BlockNumberFor<T>,
        BoundedVec<(u64, u32), <T as Config>::MaxActivationsPerBlock>,
        ValueQuery,
    >;

    /// 不可修改条款 manifest(创世冻结,无 setter,见 [`ImmutableManifest`])。
    #[pallet::storage]
    pub type ConstitutionImmutableManifest<T: Config> =
        StorageValue<_, ImmutableManifest, OptionQuery>;

    /// 创世配置:注入内置公民宪法作为 `tier=宪法`、`law_id=0` 的链上法律(宪法唯一真源)。
    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        /// 宪法归属立法机构院序列(默认 [国家立法院]);为空则不注入宪法。
        pub constitution_houses: HousesOf<T>,
    }

    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            let nlg_account = T::AccountId::decode(&mut &CHINA_LF[0].main_account[..])
                .expect("国家立法院 main_account 必须解码为 AccountId");
            let constitution_houses = BoundedVec::try_from(sp_runtime::sp_std::vec![(
                NATIONAL_LEGISLATURE_CODE,
                nlg_account
            )])
            .expect("constitution houses within bound");
            Self {
                constitution_houses,
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            if self.constitution_houses.is_empty() {
                return;
            }
            let chapters = ChaptersOf::<T>::decode(&mut &CONSTITUTION_SCALE[..])
                .expect("内置 constitution.scale 必须解码为 ChaptersOf");

            // L3 创世锚:逐条断言不可修改条款存在(缺即 panic,烤不出非法创世),
            // 并把条号 + 内容摘要冻结进链上 manifest,供节点启动期交叉校验(ADR-027 §6.1)。
            let mut article_numbers =
                BoundedVec::<u32, ConstU32<{ MAX_IMMUTABLE_ARTICLES }>>::new();
            let mut article_hashes =
                BoundedVec::<[u8; 32], ConstU32<{ MAX_IMMUTABLE_ARTICLES }>>::new();
            for &n in IMMUTABLE_CONSTITUTION_ARTICLES.iter() {
                let article = Pallet::<T>::find_article(&chapters, n)
                    .unwrap_or_else(|| panic!("不可修改条款第 {n} 条必须存在于创世宪法"));
                article_numbers
                    .try_push(n)
                    .expect("MAX_IMMUTABLE_ARTICLES 必须 >= 不可修改条款数");
                article_hashes
                    .try_push(sp_io::hashing::blake2_256(&article.encode()))
                    .expect("MAX_IMMUTABLE_ARTICLES 必须 >= 不可修改条款数");
            }
            ConstitutionImmutableManifest::<T>::put(ImmutableManifest {
                article_numbers,
                article_hashes,
            });

            let title = BoundedVec::try_from("公民宪法".as_bytes().to_vec())
                .expect("constitution title within bound");
            let title_en = BoundedVec::try_from("Citizen Constitution".as_bytes().to_vec())
                .expect("constitution title_en within bound");
            let now = frame_system::Pallet::<T>::block_number();
            let law_id = NextLawId::<T>::mutate(|n| {
                let id = *n;
                *n = n.saturating_add(1);
                id
            });
            let version = 1u32;
            let lv = LawVersion::<T> {
                law_id,
                version,
                title,
                title_en: Some(title_en),
                content_hash: Pallet::<T>::hash_chapters(&chapters),
                chapters,
                vote_type: VoteType::Special,
                proposal_id: 0,
                published_at: now,
                effective_at: now,
            };
            LawVersions::<T>::insert(law_id, version, lv);
            let law = Law::<T> {
                law_id,
                tier: Tier::Constitution,
                scope_code: 0,
                houses: self.constitution_houses.clone(),
                current_version: version,
                status: LawStatus::Effective,
            };
            Laws::<T>::insert(law_id, law);
            let _ = LawsByScope::<T>::try_mutate(Tier::Constitution, 0, |v| v.try_push(law_id));
        }
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// 立法/修法/废法提案已创建并进入投票。
        LawProposalCreated {
            proposal_id: u64,
            action: LawAction,
            law_id: Option<u64>,
            proposer: T::AccountId,
        },
        /// 提案被否决。
        LawProposalRejected { proposal_id: u64 },
        /// 新法已立(待生效)。
        LawEnacted { law_id: u64, version: u32 },
        /// 法律已修订(新版本待生效)。
        LawAmended { law_id: u64, version: u32 },
        /// 法律已废止。
        LawRepealed { law_id: u64 },
        /// 法律版本已生效。
        LawEffective { law_id: u64, version: u32 },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// 标题为空
        EmptyTitle,
        /// 法律全文(章节)为空
        EmptyChapters,
        /// 院序列为空
        EmptyHouses,
        /// 发起人不是该立法机构的现任议员/委员(admins)
        NotLegislator,
        /// 宪法修改的表决类型不合法(只能特别案或重要案)
        InvalidVoteTypeForConstitution,
        /// 命中宪法不可修改条款(第 1/2/3/17/19/23/33/41 条)
        ImmutableArticleViolation,
        /// 宪法不可整体废止
        CannotRepealConstitution,
        /// 宪法唯一真源 = 创世 law_id=0,不可经立法入口新立第二部宪法
        CannotEnactConstitution,
        /// 法律不存在
        LawNotFound,
        /// 法律版本不存在
        LawVersionNotFound,
        /// 法律已废止,不能再修改
        LawAlreadyRepealed,
        /// 立法投票引擎建提案失败
        VoteEngineCreateFailed,
        /// votingengine 提案载荷缺失或解码失败
        ProposalPayloadInvalid,
        /// 该 (tier, scope) 下法律数量超上限
        TooManyLawsInScope,
        /// 同一生效区块的待激活法律数量超上限
        TooManyActivations,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        /// 到生效区块把对应法律版本从 Pending 翻为 Effective。
        fn on_initialize(now: BlockNumberFor<T>) -> Weight {
            let due = PendingActivation::<T>::take(now);
            let n = due.len() as u64;
            for (law_id, version) in due.into_iter() {
                Self::set_effective(law_id, version);
            }
            T::DbWeight::get().reads_writes(n.saturating_add(1), n.saturating_add(1))
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 立法(新法):立法机构议员/委员发起,走立法投票。
        #[pallet::call_index(0)]
        #[pallet::weight(<T as Config>::WeightInfo::propose_enact_law())]
        pub fn propose_enact_law(
            origin: OriginFor<T>,
            tier: Tier,
            scope_code: u32,
            houses: HousesOf<T>,
            vote_type: VoteType,
            title: TitleOf<T>,
            title_en: Option<TitleOf<T>>,
            chapters: ChaptersOf<T>,
            effective_at: BlockNumberFor<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            // 宪法唯一真源 = 创世注入的 law_id=0,立法入口永不能新立第二部宪法(ADR-027 §6.1)。
            ensure!(
                tier != Tier::Constitution,
                Error::<T>::CannotEnactConstitution
            );
            ensure!(!title.is_empty(), Error::<T>::EmptyTitle);
            ensure!(!chapters.is_empty(), Error::<T>::EmptyChapters);
            Self::ensure_legislator(&houses, &who)?;
            Self::ensure_tier_vote_type(tier, vote_type)?;

            let summary = LawProposalSummary::<T> {
                action: LawAction::Enact,
                law_id: 0,
                tier,
                scope_code,
                houses: houses.clone(),
                vote_type,
                title,
                title_en,
                content_hash: Self::hash_chapters(&chapters),
                effective_at,
            };
            let proposal_id =
                Self::dispatch_to_engine(&who, &houses, vote_type, &summary, &chapters)?;
            Self::deposit_event(Event::<T>::LawProposalCreated {
                proposal_id,
                action: LawAction::Enact,
                law_id: None,
                proposer: who,
            });
            Ok(())
        }

        /// 修法:针对既有法律提交新版本(整部全文快照),走立法投票。
        #[pallet::call_index(1)]
        #[pallet::weight(<T as Config>::WeightInfo::propose_amend_law())]
        pub fn propose_amend_law(
            origin: OriginFor<T>,
            law_id: u64,
            vote_type: VoteType,
            title: TitleOf<T>,
            title_en: Option<TitleOf<T>>,
            chapters: ChaptersOf<T>,
            effective_at: BlockNumberFor<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(!title.is_empty(), Error::<T>::EmptyTitle);
            ensure!(!chapters.is_empty(), Error::<T>::EmptyChapters);
            let law = Laws::<T>::get(law_id).ok_or(Error::<T>::LawNotFound)?;
            ensure!(
                law.status != LawStatus::Repealed,
                Error::<T>::LawAlreadyRepealed
            );
            Self::ensure_legislator(&law.houses, &who)?;
            Self::ensure_tier_vote_type(law.tier, vote_type)?;
            if law.tier == Tier::Constitution {
                Self::ensure_immutable_preserved(law_id, law.current_version, &chapters)?;
            }

            let summary = LawProposalSummary::<T> {
                action: LawAction::Amend,
                law_id,
                tier: law.tier,
                scope_code: law.scope_code,
                houses: law.houses.clone(),
                vote_type,
                title,
                title_en,
                content_hash: Self::hash_chapters(&chapters),
                effective_at,
            };
            let proposal_id =
                Self::dispatch_to_engine(&who, &law.houses, vote_type, &summary, &chapters)?;
            Self::deposit_event(Event::<T>::LawProposalCreated {
                proposal_id,
                action: LawAction::Amend,
                law_id: Some(law_id),
                proposer: who,
            });
            Ok(())
        }

        /// 废法:废止既有法律,走立法投票。宪法不可整体废止。
        #[pallet::call_index(2)]
        #[pallet::weight(<T as Config>::WeightInfo::propose_repeal_law())]
        pub fn propose_repeal_law(
            origin: OriginFor<T>,
            law_id: u64,
            vote_type: VoteType,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let law = Laws::<T>::get(law_id).ok_or(Error::<T>::LawNotFound)?;
            ensure!(
                law.status != LawStatus::Repealed,
                Error::<T>::LawAlreadyRepealed
            );
            ensure!(
                law.tier != Tier::Constitution,
                Error::<T>::CannotRepealConstitution
            );
            Self::ensure_legislator(&law.houses, &who)?;
            Self::ensure_tier_vote_type(law.tier, vote_type)?;

            let summary = LawProposalSummary::<T> {
                action: LawAction::Repeal,
                law_id,
                tier: law.tier,
                scope_code: law.scope_code,
                houses: law.houses.clone(),
                vote_type,
                title: Default::default(),
                title_en: None,
                content_hash: [0u8; 32],
                effective_at: Default::default(),
            };
            let empty: ChaptersOf<T> = Default::default();
            let proposal_id =
                Self::dispatch_to_engine(&who, &law.houses, vote_type, &summary, &empty)?;
            Self::deposit_event(Event::<T>::LawProposalCreated {
                proposal_id,
                action: LawAction::Repeal,
                law_id: Some(law_id),
                proposer: who,
            });
            Ok(())
        }
    }

    // ──────────────── 内部 helper:校验 / 编排 / 执行器 / 查询 ────────────────
    impl<T: Config> Pallet<T> {
        /// 校验院序列非空,且发起人是发起院(houses[0])的现任管理员(议员/委员)。
        fn ensure_legislator(houses: &HousesOf<T>, who: &T::AccountId) -> DispatchResult {
            let (code, body) = houses.first().ok_or(Error::<T>::EmptyHouses)?;
            ensure!(
                <T as votingengine::Config>::InternalAdminProvider::is_internal_admin(
                    *code,
                    body.clone(),
                    who
                ),
                Error::<T>::NotLegislator
            );
            Ok(())
        }

        /// 宪法修改只能走特别案或重要案(宪法第十九条)。
        fn ensure_tier_vote_type(tier: Tier, vt: VoteType) -> DispatchResult {
            if tier == Tier::Constitution {
                ensure!(
                    matches!(vt, VoteType::Special | VoteType::Important),
                    Error::<T>::InvalidVoteTypeForConstitution
                );
            }
            Ok(())
        }

        /// 在章>节>条嵌套结构里按条号查找条文。
        fn find_article(chapters: &ChaptersOf<T>, number: u32) -> Option<&Article<T>> {
            chapters
                .iter()
                .flat_map(|c| c.sections.iter())
                .flat_map(|s| s.articles.iter())
                .find(|a| a.number == number)
        }

        /// 宪法不可修改条款必须与当前版本逐字保持一致(增/改/删任一即违规)。
        /// 遍历 章>节>条 按条号比对。
        fn ensure_immutable_preserved(
            law_id: u64,
            current_version: u32,
            new_chapters: &ChaptersOf<T>,
        ) -> DispatchResult {
            let current = LawVersions::<T>::get(law_id, current_version)
                .ok_or(Error::<T>::LawVersionNotFound)?;
            for &n in IMMUTABLE_CONSTITUTION_ARTICLES.iter() {
                let cur = Self::find_article(&current.chapters, n);
                let new = Self::find_article(new_chapters, n);
                ensure!(cur == new, Error::<T>::ImmutableArticleViolation);
            }
            Ok(())
        }

        /// 规范化全文哈希(blake2_256(章节条款 SCALE))。
        pub fn hash_chapters(chapters: &ChaptersOf<T>) -> [u8; 32] {
            sp_io::hashing::blake2_256(&chapters.encode())
        }

        /// 编码载荷并调立法投票引擎建提案,返回真实提案 ID。
        /// 院序列(houses)由提案携带,发起人资格与各院表决全部归属立法投票引擎。
        fn dispatch_to_engine(
            who: &T::AccountId,
            houses: &HousesOf<T>,
            vote_type: VoteType,
            summary: &LawProposalSummary<T>,
            chapters: &ChaptersOf<T>,
        ) -> Result<u64, DispatchError> {
            let mut data = sp_runtime::sp_std::vec::Vec::from(MODULE_TAG);
            data.extend_from_slice(&summary.encode());
            let object = chapters.encode();
            let proposal_id = T::LegislationVoteEngine::create_legislation_proposal(
                who.clone(),
                houses.clone().into_inner(),
                vote_type.as_u8(),
                MODULE_TAG,
                data,
                object,
            )
            .map_err(|_| Error::<T>::VoteEngineCreateFailed)?;
            Ok(proposal_id)
        }

        /// 把某法律版本置为生效;若不是当前版本则忽略。
        fn set_effective(law_id: u64, version: u32) {
            Laws::<T>::mutate(law_id, |maybe| {
                if let Some(law) = maybe {
                    if law.current_version == version && law.status == LawStatus::Pending {
                        law.status = LawStatus::Effective;
                        Self::deposit_event(Event::<T>::LawEffective { law_id, version });
                    }
                }
            });
        }

        /// 到点即生效,否则排入 PendingActivation。
        fn activate_or_schedule(
            law_id: u64,
            version: u32,
            effective_at: BlockNumberFor<T>,
        ) -> DispatchResult {
            let now = frame_system::Pallet::<T>::block_number();
            if effective_at <= now {
                Self::set_effective(law_id, version);
            } else {
                PendingActivation::<T>::try_mutate(effective_at, |v| v.try_push((law_id, version)))
                    .map_err(|_| Error::<T>::TooManyActivations)?;
            }
            Ok(())
        }

        /// 投票通过/否决回调的内部写入逻辑(由 legislation-vote 投票终态经核心回调触发)。
        pub fn apply_legislation_vote_result(
            proposal_id: u64,
            approved: bool,
        ) -> Result<ProposalExecutionOutcome, DispatchError> {
            if !votingengine::Pallet::<T>::is_proposal_owner(proposal_id, MODULE_TAG) {
                return Ok(ProposalExecutionOutcome::Ignored);
            }
            if !approved {
                Self::deposit_event(Event::<T>::LawProposalRejected { proposal_id });
                return Ok(ProposalExecutionOutcome::Executed);
            }
            let summary = Self::load_summary(proposal_id)?;
            let chapters = Self::load_chapters(proposal_id)?;
            let now = frame_system::Pallet::<T>::block_number();
            Self::write_law_version(proposal_id, summary, chapters, now)?;
            Ok(ProposalExecutionOutcome::Executed)
        }

        /// 从 votingengine ProposalData 读回并解码本模块提案摘要(先校验 MODULE_TAG 前缀)。
        fn load_summary(proposal_id: u64) -> Result<LawProposalSummary<T>, DispatchError> {
            let raw = votingengine::Pallet::<T>::get_proposal_data(proposal_id)
                .ok_or(Error::<T>::ProposalPayloadInvalid)?;
            let tag = MODULE_TAG;
            if raw.len() < tag.len() || &raw[..tag.len()] != tag {
                return Err(Error::<T>::ProposalPayloadInvalid.into());
            }
            LawProposalSummary::<T>::decode(&mut &raw[tag.len()..])
                .map_err(|_| Error::<T>::ProposalPayloadInvalid.into())
        }

        /// 从 votingengine ProposalObject 读回并解码法律全文(章>节>条>款)。
        fn load_chapters(proposal_id: u64) -> Result<ChaptersOf<T>, DispatchError> {
            let raw = votingengine::Pallet::<T>::get_proposal_object(proposal_id)
                .ok_or(Error::<T>::ProposalPayloadInvalid)?;
            ChaptersOf::<T>::decode(&mut &raw[..])
                .map_err(|_| Error::<T>::ProposalPayloadInvalid.into())
        }

        /// 把通过的提案写入法律存储(立法新增 / 修法升版 / 废法置废)。
        pub fn write_law_version(
            proposal_id: u64,
            summary: LawProposalSummary<T>,
            chapters: ChaptersOf<T>,
            now: BlockNumberFor<T>,
        ) -> DispatchResult {
            match summary.action {
                LawAction::Enact => {
                    let law_id = NextLawId::<T>::mutate(|n| {
                        let id = *n;
                        *n = n.saturating_add(1);
                        id
                    });
                    let version = 1u32;
                    let lv = LawVersion::<T> {
                        law_id,
                        version,
                        title: summary.title,
                        title_en: summary.title_en,
                        chapters,
                        content_hash: summary.content_hash,
                        vote_type: summary.vote_type,
                        proposal_id,
                        published_at: now,
                        effective_at: summary.effective_at,
                    };
                    LawVersions::<T>::insert(law_id, version, lv);
                    let law = Law::<T> {
                        law_id,
                        tier: summary.tier,
                        scope_code: summary.scope_code,
                        houses: summary.houses,
                        current_version: version,
                        status: LawStatus::Pending,
                    };
                    Laws::<T>::insert(law_id, law);
                    LawsByScope::<T>::try_mutate(summary.tier, summary.scope_code, |v| {
                        v.try_push(law_id)
                    })
                    .map_err(|_| Error::<T>::TooManyLawsInScope)?;
                    Self::deposit_event(Event::<T>::LawEnacted { law_id, version });
                    Self::activate_or_schedule(law_id, version, summary.effective_at)?;
                }
                LawAction::Amend => {
                    let mut law = Laws::<T>::get(summary.law_id).ok_or(Error::<T>::LawNotFound)?;
                    let version = law.current_version.saturating_add(1);
                    let lv = LawVersion::<T> {
                        law_id: summary.law_id,
                        version,
                        title: summary.title,
                        title_en: summary.title_en,
                        chapters,
                        content_hash: summary.content_hash,
                        vote_type: summary.vote_type,
                        proposal_id,
                        published_at: now,
                        effective_at: summary.effective_at,
                    };
                    LawVersions::<T>::insert(summary.law_id, version, lv);
                    law.current_version = version;
                    law.status = LawStatus::Pending;
                    Laws::<T>::insert(summary.law_id, law);
                    Self::deposit_event(Event::<T>::LawAmended {
                        law_id: summary.law_id,
                        version,
                    });
                    Self::activate_or_schedule(summary.law_id, version, summary.effective_at)?;
                }
                LawAction::Repeal => {
                    Laws::<T>::mutate(summary.law_id, |maybe| {
                        if let Some(law) = maybe {
                            law.status = LawStatus::Repealed;
                        }
                    });
                    Self::deposit_event(Event::<T>::LawRepealed {
                        law_id: summary.law_id,
                    });
                }
            }
            Ok(())
        }

        // ───────── 查询(供 runtime API 调用)─────────
        /// 读取法律主体。
        pub fn get_law(law_id: u64) -> Option<Law<T>> {
            Laws::<T>::get(law_id)
        }

        /// 读取法律指定版本。
        pub fn get_law_version(law_id: u64, version: u32) -> Option<LawVersion<T>> {
            LawVersions::<T>::get(law_id, version)
        }

        /// 列出某层级 + 行政区下的法律 ID。
        pub fn list_laws(tier: Tier, scope_code: u32) -> sp_runtime::sp_std::vec::Vec<u64> {
            LawsByScope::<T>::get(tier, scope_code).into_inner()
        }
    }
}

/// 立法投票终态回调接入:投票引擎在立法提案达终态时按 kind 广播到此,
/// 由本业务壳认领并写入法律(runtime 装配 `votingengine::Config::LegislationVoteResultCallback = LegislationYuan`)。
impl<T: pallet::Config> votingengine::LegislationVoteResultCallback for pallet::Pallet<T> {
    fn on_legislation_vote_finalized(
        vote_proposal_id: u64,
        approved: bool,
    ) -> Result<votingengine::ProposalExecutionOutcome, sp_runtime::DispatchError> {
        pallet::Pallet::<T>::apply_legislation_vote_result(vote_proposal_id, approved)
    }
}

#[cfg(test)]
mod tests;
