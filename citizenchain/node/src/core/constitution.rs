//! 宪法节点能力统一入口(ADR-027)。
//!
//! 宪法已迁入链上立法院模块(`legislation-yuan`,`law_id=0`、`tier=宪法`,创世注入),
//! 唯一真源 = 链上法律。本文件统一承载节点端两件事:
//!
//! 1. **渲染**(展示):从链上结构化宪法(章>节>条>款 + 中英双语)重建《公民宪法》HTML,
//!    复用原 CSS 外壳,供桌面端「公民宪法」tab 显示(`constitution_getDocument` RPC)。
//! 2. **不可修改条款守卫**(L2 共识层):宪法第 1/2/3/17/19/23/33/41 条为「不可修改条款」,
//!    本守卫在区块导入时逐块校验这些条文与**创世(block#0)逐字一致**,违者拒块。
//!    执法逻辑在 runtime 之外的节点二进制里,清单(`primitives::IMMUTABLE_CONSTITUTION_ARTICLES`)
//!    编译进二进制 —— 故 setCode / migration / 改清单常量都改不动这些条文;唯一修改路径 =
//!    改创世(创世哈希变 = 新链)或改节点二进制(硬分叉),即「只能重新创世」。详见 ADR-027 §7。

use std::collections::BTreeMap;
use std::sync::Arc;

use codec::{Decode, Encode};
use sc_client_api::backend::{Backend as _, TrieCacheContext};
use sc_client_api::StorageProvider;
use sc_consensus::{BlockCheckParams, BlockImport, BlockImportParams, ImportResult};
use sp_api::{ApiExt, Core, ProvideRuntimeApi};
use sp_blockchain::HeaderBackend;
use sp_consensus::Error as ConsensusError;
use sp_runtime::traits::{Block as BlockT, Header as HeaderT};
use sp_storage::StorageKey;

use primitives::count_const::IMMUTABLE_CONSTITUTION_ARTICLES;

use citizenchain::opaque::Block;

use crate::core::service::{FullBackend, FullClient};

/// 宪法在立法院模块固定为 `law_id = 0`(创世注入)。
pub const CONSTITUTION_LAW_ID: u64 = 0;
/// 创世注入的宪法版本号,作为不可修改条款的内容基准来源。
const GENESIS_CONSTITUTION_VERSION: u32 = 1;

/// Law 元数据判别值(SCALE 变体索引,与 `legislation-yuan::types` 声明序一致:
/// `Tier{Constitution=0,..}`、`LawStatus{Pending=0,Effective=1,Repealed=2}`)。
/// 由 legislation-yuan 测试 `enum_discriminants_match_node_guard` 交叉钉死,防漂移。
const TIER_CONSTITUTION: u8 = 0;
const LAW_STATUS_PENDING: u8 = 0;
const LAW_STATUS_REPEALED: u8 = 2;

// ───────── 链上结构镜像 ─────────
// 字段序必须与 legislation-yuan 链端 `LawVersion / Chapter / Section / Article / Clause` 一致。
// SCALE 按声明序顺序解码,解到所需字段即停 —— 尾部字段无需镜像。Encode 仅用于不可修改条款的
// 规范字节比对(同一逻辑内容 → 同一字节)。

#[derive(Encode, Decode, PartialEq)]
#[allow(dead_code)] // 款号 number 不参与渲染/比对(text 已含「第N款」前缀),仅占位保持字段序。
struct MClause {
    number: u32,
    text: Vec<u8>,
    text_en: Option<Vec<u8>>,
}

#[derive(Encode, Decode, PartialEq)]
struct MArticle {
    number: u32,
    title: Vec<u8>,
    title_en: Option<Vec<u8>>,
    body: Vec<u8>,
    body_en: Option<Vec<u8>>,
    clauses: Vec<MClause>,
}

#[derive(Encode, Decode, PartialEq)]
struct MSection {
    number: u32,
    title: Vec<u8>,
    title_en: Option<Vec<u8>>,
    articles: Vec<MArticle>,
}

#[derive(Encode, Decode, PartialEq)]
struct MChapter {
    number: u32,
    title: Vec<u8>,
    title_en: Option<Vec<u8>>,
    sections: Vec<MSection>,
}

/// 只解码 `LawVersion` 前缀(law_id..chapters);其后字段顺序解码到 chapters 即停。
#[derive(Decode)]
#[allow(dead_code)] // law_id/version/title/title_en 仅占位保持字段序,只用 chapters。
struct MLawVersionHead {
    law_id: u64,
    version: u32,
    title: Vec<u8>,
    title_en: Option<Vec<u8>>,
    chapters: Vec<MChapter>,
}

/// 解码 `Law`(到 status 即停)。houses = `Vec<(InstitutionCode=[u8;4], AccountId=[u8;32])>`,
/// 与链端 `HousesOf` 一致;tier/status 为枚举变体索引(u8)。守卫据此校验宪法元数据不变式。
#[derive(Decode)]
#[allow(dead_code)] // law_id 占位保持字段序。
struct MLawHead {
    law_id: u64,
    tier: u8,
    scope_code: u32,
    houses: Vec<([u8; 4], [u8; 32])>,
    current_version: u32,
    status: u8,
}

/// 解码宪法 `Law` 记录(失败 → `LawDecodeFailed`)。
fn decode_law_head(law_scale: &[u8]) -> Result<MLawHead, GuardError> {
    MLawHead::decode(&mut &law_scale[..]).map_err(|_| GuardError::LawDecodeFailed)
}

/// 在 章>节>条 嵌套结构里按条号查找条文。
fn find_article<'a>(chapters: &'a [MChapter], number: u32) -> Option<&'a MArticle> {
    chapters
        .iter()
        .flat_map(|c| c.sections.iter())
        .flat_map(|s| s.articles.iter())
        .find(|a| a.number == number)
}

// ═════════════════════════════════════════════════════════════════════════
// 一、渲染:链上结构化宪法 → HTML(供桌面端「公民宪法」tab)
// ═════════════════════════════════════════════════════════════════════════

/// 完整 HTML 外壳模板(原《公民宪法》页的 head/style/封面/目录与正文容器全保留),
/// 内含两个占位标记,渲染时按链上结构化宪法替换:
///   `<!--CONSTITUTION_TOC-->`     → 目录项(`toc-item`)
///   `<!--CONSTITUTION_CONTENT-->` → 正文块(章/节/条/款)
const SHELL: &str = include_str!("constitution_shell.html");
const TOC_MARKER: &str = "<!--CONSTITUTION_TOC-->";
const CONTENT_MARKER: &str = "<!--CONSTITUTION_CONTENT-->";

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

fn opt_text(bytes: &Option<Vec<u8>>) -> String {
    bytes.as_ref().map(|b| text(b)).unwrap_or_default()
}

/// HTML 文本转义(纵深防御:链上文本为治理立法产出,非用户任意输入,但仍转义防外壳破坏)。
fn esc(raw: &str) -> String {
    raw.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

/// 当前**生效中**的版本号(供桌面端展示,避免提前显示待生效版):
/// `status == Pending` 时现行版 = `current_version - 1`(v1 为创世恒生效,不回退);否则 = `current_version`。
pub fn effective_version_of_law(law_scale: &[u8]) -> Result<u32, String> {
    let law = decode_law_head(law_scale).map_err(|e| format!("宪法 Law 解码失败:{e:?}"))?;
    let v = if law.status == LAW_STATUS_PENDING && law.current_version > 1 {
        law.current_version - 1
    } else {
        law.current_version
    };
    Ok(v)
}

/// 把链上结构化宪法 SCALE 字节(`LawVersion` 编码)重建为完整 HTML 文档。
pub fn render_constitution_html(law_version_scale: &[u8]) -> Result<String, String> {
    let law = MLawVersionHead::decode(&mut &law_version_scale[..])
        .map_err(|e| format!("宪法 LawVersion 解码失败:{e}"))?;

    let mut toc = String::new();
    let mut content = String::new();

    for chapter in &law.chapters {
        let (c_cn, c_en) = (
            esc(&text(&chapter.title)),
            esc(&opt_text(&chapter.title_en)),
        );
        toc.push_str(&format!(
            "        <a class=\"toc-item toc-level-1\" href=\"#chapter-{n}\"><span class=\"toc-cn\">{c_cn}</span><span class=\"toc-en\">{c_en}</span></a>\n",
            n = chapter.number,
        ));
        content.push_str(&format!(
            "<section id=\"chapter-{n}\" class=\"block chapter-block\">\n  <h1 class=\"chapter-title\">\n    <span class=\"cn heading-cn\">{c_cn}</span>\n    <span class=\"en heading-en\">{c_en}</span>\n  </h1>\n</section>\n\n",
            n = chapter.number,
        ));

        for section in &chapter.sections {
            let (s_cn, s_en) = (
                esc(&text(&section.title)),
                esc(&opt_text(&section.title_en)),
            );
            toc.push_str(&format!(
                "        <a class=\"toc-item toc-level-2\" href=\"#chapter-{cn}-section-{sn}\"><span class=\"toc-cn\">{s_cn}</span><span class=\"toc-en\">{s_en}</span></a>\n",
                cn = chapter.number, sn = section.number,
            ));
            content.push_str(&format!(
                "<section id=\"chapter-{cn}-section-{sn}\" class=\"block section-block\">\n  <h2 class=\"section-title\">\n    <span class=\"cn heading-cn\">{s_cn}</span>\n    <span class=\"en heading-en\">{s_en}</span>\n  </h2>\n</section>\n\n",
                cn = chapter.number, sn = section.number,
            ));

            for article in &section.articles {
                let (a_cn, a_en) = (
                    esc(&text(&article.title)),
                    esc(&opt_text(&article.title_en)),
                );
                toc.push_str(&format!(
                    "        <a class=\"toc-item toc-level-3\" href=\"#article-{an}\"><span class=\"toc-cn\">{a_cn}</span><span class=\"toc-en\">{a_en}</span></a>\n",
                    an = article.number,
                ));

                // 首段为条正文(必填),其后每段为款(可空)。
                let mut paragraphs = format!(
                    "  <p class=\"article-paragraph\">\n    <span class=\"cn body-cn\">{b_cn}</span>\n    <span class=\"en body-en\">{b_en}</span>\n  </p>\n",
                    b_cn = esc(&text(&article.body)),
                    b_en = esc(&opt_text(&article.body_en)),
                );
                for clause in &article.clauses {
                    paragraphs.push_str(&format!(
                        "  <p class=\"article-paragraph\">\n    <span class=\"cn body-cn\">{k_cn}</span>\n    <span class=\"en body-en\">{k_en}</span>\n  </p>\n",
                        k_cn = esc(&text(&clause.text)),
                        k_en = esc(&opt_text(&clause.text_en)),
                    ));
                }

                content.push_str(&format!(
                    "<article id=\"article-{an}\" class=\"block article-block\">\n  <h3 class=\"article-title\">\n    <span class=\"cn heading-cn\">{a_cn}</span>\n    <span class=\"en heading-en\">{a_en}</span>\n  </h3>\n\n{paragraphs}</article>\n\n",
                    an = article.number,
                ));
            }
        }
    }

    // 把目录与正文填入外壳两个占位标记 —— 整页结构只在 constitution_shell.html 一处维护。
    Ok(SHELL
        .replace(TOC_MARKER, &toc)
        .replace(CONTENT_MARKER, &content))
}

// ═════════════════════════════════════════════════════════════════════════
// 二、不可修改条款守卫(L2 共识层)
// ═════════════════════════════════════════════════════════════════════════

/// 立法院模块在 `construct_runtime` 中的 pallet 名(twox128 前缀据此推导)。
/// 硬编码,绝不读链上 metadata —— metadata 属可升级 runtime,会被恶意升级伪造。
const PALLET_NAME: &[u8] = b"LegislationYuan";

/// 守卫推导出的宪法存储 RAW key(`Laws` / `LawVersions`),硬编码 hasher 与链端一致:
/// `Laws: StorageMap<Blake2_128Concat, u64, ..>`、
/// `LawVersions: StorageDoubleMap<Blake2_128Concat u64, Blake2_128Concat u32, ..>`。
pub mod storage_key {
    use super::PALLET_NAME;
    use codec::Encode;
    use sp_core::hashing::{blake2_128, twox_128};

    fn map_prefix(storage: &[u8]) -> Vec<u8> {
        let mut k = Vec::with_capacity(32);
        k.extend_from_slice(&twox_128(PALLET_NAME));
        k.extend_from_slice(&twox_128(storage));
        k
    }

    fn blake2_128_concat(encoded: &[u8]) -> Vec<u8> {
        let mut out = blake2_128(encoded).to_vec();
        out.extend_from_slice(encoded);
        out
    }

    /// `LegislationYuan::Laws[law_id]` 的完整存储 key。
    pub fn law(law_id: u64) -> Vec<u8> {
        let mut k = map_prefix(b"Laws");
        k.extend_from_slice(&blake2_128_concat(&law_id.encode()));
        k
    }

    /// `LegislationYuan::LawVersions[law_id][version]` 的完整存储 key。
    pub fn law_version(law_id: u64, version: u32) -> Vec<u8> {
        let mut k = map_prefix(b"LawVersions");
        k.extend_from_slice(&blake2_128_concat(&law_id.encode()));
        k.extend_from_slice(&blake2_128_concat(&version.encode()));
        k
    }

    /// `LegislationYuan::ConstitutionImmutableManifest`(StorageValue,无 key hash)的完整 key。
    pub fn manifest() -> Vec<u8> {
        map_prefix(b"ConstitutionImmutableManifest")
    }

    /// `LegislationYuan::LawsByScope[Tier::Constitution][0]` 的完整 key(宪法层级唯一性校验)。
    /// Tier::Constitution 编码为单字节变体索引 `[0]`,scope_code = 0。
    pub fn laws_by_scope_constitution() -> Vec<u8> {
        let mut k = map_prefix(b"LawsByScope");
        k.extend_from_slice(&blake2_128_concat(&[super::TIER_CONSTITUTION]));
        k.extend_from_slice(&blake2_128_concat(&0u32.encode()));
        k
    }

    /// 立法院模块存储的公共前缀(twox128(pallet)),用于快速判断区块是否动过宪法相关存储。
    pub fn pallet_prefix() -> [u8; 16] {
        twox_128(PALLET_NAME)
    }
}

/// 链上不可修改条款 manifest 镜像(与 `legislation-yuan::ImmutableManifest` 字段序一致)。
#[derive(Decode)]
struct MImmutableManifest {
    article_numbers: Vec<u32>,
    article_hashes: Vec<[u8; 32]>,
}

/// 启动期交叉校验:创世 manifest 的清单必须 == 节点二进制单源,且逐条摘要 == 基准条文摘要。
/// 任一不符 → 返回 `Err`(节点应拒绝启动)。纯函数,便于单测。
fn verify_manifest(manifest_bytes: &[u8], reference: &ImmutableReference) -> Result<(), String> {
    let manifest = MImmutableManifest::decode(&mut &manifest_bytes[..])
        .map_err(|e| format!("manifest 解码失败:{e}"))?;

    // 1. 清单一致(双锚:创世 manifest ↔ 节点二进制常量)。
    let mut on_chain = manifest.article_numbers.clone();
    on_chain.sort_unstable();
    let mut binary: Vec<u32> = IMMUTABLE_CONSTITUTION_ARTICLES.to_vec();
    binary.sort_unstable();
    if on_chain != binary {
        return Err(format!(
            "创世 manifest 清单 {on_chain:?} 与节点二进制 {binary:?} 不一致"
        ));
    }

    // 2. 逐条摘要 == 节点从 block#0 派生的条文摘要(防 manifest 谎报)。
    for (number, reference_bytes) in reference.articles.iter() {
        let idx = manifest
            .article_numbers
            .iter()
            .position(|x| x == number)
            .ok_or_else(|| format!("manifest 缺第 {number} 条"))?;
        let want = sp_core::blake2_256(reference_bytes);
        if manifest.article_hashes.get(idx) != Some(&want) {
            return Err(format!("manifest 第 {number} 条摘要与创世条文不符"));
        }
    }
    Ok(())
}

/// 不可修改条款守卫的判定失败原因(全部一律拒块/拒启,fail-safe 方向恒为「拒绝」)。
#[derive(Debug, PartialEq)]
pub enum GuardError {
    /// 宪法 `Law(0)` 在目标状态缺失(存储被改名/删除)。
    ConstitutionLawMissing,
    /// `Law` 解码失败。
    LawDecodeFailed,
    /// 当前版本 `LawVersion` 缺失。
    LawVersionMissing,
    /// `LawVersion` 解码失败。
    VersionDecodeFailed,
    /// 某不可修改条款在状态中缺失(被删/改号)。
    ImmutableArticleMissing(u32),
    /// 基准中缺该不可修改条款(创世派生异常)。
    ReferenceMissing(u32),
    /// 某不可修改条款内容被改动(与创世基准不一致)。
    ImmutableArticleMutated(u32),
    /// 宪法 `tier` 被改(不再是 Constitution)。
    ConstitutionTierChanged,
    /// 宪法 `scope_code` 被改(不再是全国 0)。
    ConstitutionScopeChanged,
    /// 宪法被置为 Repealed(违反不可废止)。
    ConstitutionRepealed,
    /// 宪法可修订机构(`houses`)被改(与创世不一致)。
    ConstitutionHousesChanged,
    /// 宪法层级唯一性被破坏(`LawsByScope[宪法][0]` 不再恰为 `[0]`:多出第二部宪法或 law_id=0 被隐藏)。
    ConstitutionNotUnique,
}

/// 不可修改基准(创世/block#0):不可修改条款的规范 SCALE 字节 + 可修订机构 `houses`。
pub struct ImmutableReference {
    articles: BTreeMap<u32, Vec<u8>>,
    houses: Vec<([u8; 4], [u8; 32])>,
}

impl ImmutableReference {
    /// 从一个 RAW 存储读取闭包(应指向 block#0 创世状态)派生基准:
    /// 读 `Laws[0]` 取 `houses`,读创世版本 `LawVersions[0][1]` 取不可修改条款规范字节。
    /// 任一缺失/不合法 → 返回错误(创世不合法,调用方应拒绝启动)。
    pub fn from_raw_reader<F>(read_raw: F) -> Result<Self, GuardError>
    where
        F: Fn(&[u8]) -> Option<Vec<u8>>,
    {
        let law_bytes = read_raw(&storage_key::law(CONSTITUTION_LAW_ID))
            .ok_or(GuardError::ConstitutionLawMissing)?;
        let law = decode_law_head(&law_bytes)?;

        let version_bytes = read_raw(&storage_key::law_version(
            CONSTITUTION_LAW_ID,
            GENESIS_CONSTITUTION_VERSION,
        ))
        .ok_or(GuardError::LawVersionMissing)?;
        let head = MLawVersionHead::decode(&mut &version_bytes[..])
            .map_err(|_| GuardError::VersionDecodeFailed)?;

        let mut articles = BTreeMap::new();
        for &n in IMMUTABLE_CONSTITUTION_ARTICLES.iter() {
            let article =
                find_article(&head.chapters, n).ok_or(GuardError::ImmutableArticleMissing(n))?;
            articles.insert(n, article.encode());
        }
        Ok(Self {
            articles,
            houses: law.houses,
        })
    }
}

/// 纯判定:给定一个指向**目标区块后置状态**的 RAW 读取闭包,校验宪法全部不变式:
/// ① Law 元数据(tier=Constitution、scope=0、status≠Repealed、houses=创世);
/// ② 层级唯一性(`LawsByScope[宪法][0] == [0]`);③ 不可修改条款逐字 == 创世基准。
/// 任一缺失/解码失败/不一致 → 返回 `Err`(拒块)。
pub fn check_immutable_articles<F>(
    read_raw: F,
    reference: &ImmutableReference,
) -> Result<(), GuardError>
where
    F: Fn(&[u8]) -> Option<Vec<u8>>,
{
    // ── ① Law 元数据不变式 ──
    let law_bytes = read_raw(&storage_key::law(CONSTITUTION_LAW_ID))
        .ok_or(GuardError::ConstitutionLawMissing)?;
    let law = decode_law_head(&law_bytes)?;
    if law.tier != TIER_CONSTITUTION {
        return Err(GuardError::ConstitutionTierChanged);
    }
    if law.scope_code != 0 {
        return Err(GuardError::ConstitutionScopeChanged);
    }
    if law.status == LAW_STATUS_REPEALED {
        return Err(GuardError::ConstitutionRepealed);
    }
    // status 允许 Pending(0)/Effective(1):合法修宪的待生效窗口 status=Pending,不可误杀。
    if law.houses != reference.houses {
        return Err(GuardError::ConstitutionHousesChanged);
    }

    // ── ② 层级唯一性:LawsByScope[宪法][0] 必须恰为 [0] ──
    let scope_bytes = read_raw(&storage_key::laws_by_scope_constitution())
        .ok_or(GuardError::ConstitutionNotUnique)?;
    let scope_list =
        Vec::<u64>::decode(&mut &scope_bytes[..]).map_err(|_| GuardError::ConstitutionNotUnique)?;
    if scope_list != [CONSTITUTION_LAW_ID] {
        return Err(GuardError::ConstitutionNotUnique);
    }

    // ── ③ 不可修改条款逐字一致 ──
    let version_bytes = read_raw(&storage_key::law_version(
        CONSTITUTION_LAW_ID,
        law.current_version,
    ))
    .ok_or(GuardError::LawVersionMissing)?;
    let head = MLawVersionHead::decode(&mut &version_bytes[..])
        .map_err(|_| GuardError::VersionDecodeFailed)?;
    for &n in IMMUTABLE_CONSTITUTION_ARTICLES.iter() {
        let current =
            find_article(&head.chapters, n).ok_or(GuardError::ImmutableArticleMissing(n))?;
        let baseline = reference
            .articles
            .get(&n)
            .ok_or(GuardError::ReferenceMissing(n))?;
        if &current.encode() != baseline {
            return Err(GuardError::ImmutableArticleMutated(n));
        }
    }
    Ok(())
}

/// 区块导入守卫:包住内层 `BlockImport`(PoW),在区块进入规范链之前校验不可修改条款。
///
/// 判定路径:对携带 body 的普通区块,先用 runtime API 在**父状态**上只读执行该区块得到后置存储变更,
/// 仅当变更触及立法院模块存储时,据「变更 ∪ 父状态」重建宪法相关 RAW 值并比对基准;
/// 命中违规 → 返回 `Ok(KnownBad)`(内层永不被调用,区块不入库、不成为最佳块);
/// 校验通过 → 原样委派内层正常导入(只读执行不改提交路径,故安全)。
pub struct ConstitutionGuard<I> {
    inner: I,
    client: Arc<FullClient>,
    backend: Arc<FullBackend>,
    reference: ImmutableReference,
}

impl<I> ConstitutionGuard<I> {
    /// 装配守卫:从创世(block#0)状态派生不可修改条款基准。基准缺失即返回错误(应拒绝启动)。
    pub fn new(
        inner: I,
        client: Arc<FullClient>,
        backend: Arc<FullBackend>,
    ) -> Result<Self, String> {
        let genesis_hash = client.info().genesis_hash;
        // 基准从 block#0(创世)状态 RAW 读取:创世哈希为之背书,改它即换链。
        let reference = ImmutableReference::from_raw_reader(|key| {
            client
                .storage(genesis_hash, &StorageKey(key.to_vec()))
                .ok()
                .flatten()
                .map(|data| data.0)
        })
        .map_err(|e| format!("宪法守卫:创世不可修改条款基准派生失败:{e:?}"))?;

        // L3 启动交叉校验:创世 manifest ↔ 二进制清单 ↔ 创世条文三者一致,否则拒绝启动。
        let manifest_bytes = client
            .storage(genesis_hash, &StorageKey(storage_key::manifest()))
            .ok()
            .flatten()
            .map(|data| data.0)
            .ok_or_else(|| "宪法守卫:创世缺不可修改条款 manifest".to_string())?;
        verify_manifest(&manifest_bytes, &reference)
            .map_err(|e| format!("宪法守卫:启动交叉校验失败:{e}"))?;

        Ok(Self {
            inner,
            client,
            backend,
            reference,
        })
    }

    /// 校验某已提交区块状态(`hash`)的宪法全部不变式 —— 供 warp/状态导入块用。
    /// `Ok(())` = 合规;`Err` = 违规(拒块)。直接 RAW 读该状态,不走 runtime API。
    fn verify_committed_state(&self, hash: <Block as BlockT>::Hash) -> Result<(), String> {
        let read = |key: &[u8]| -> Option<Vec<u8>> {
            self.client
                .storage(hash, &StorageKey(key.to_vec()))
                .ok()
                .flatten()
                .map(|data| data.0)
        };
        check_immutable_articles(read, &self.reference).map_err(|e| format!("{e:?}"))
    }

    /// 计算普通(执行型)区块后置状态是否违反宪法不变式。warp/状态导入块不走此路径(见 `import_block`)。
    /// `Ok(true)` = 确认违规(拒块);`Ok(false)` = 合规;`Err` = 无法判定(放行内层,见模块注释)。
    fn detect_violation(&self, params: &BlockImportParams<Block>) -> Result<bool, String> {
        let body = match &params.body {
            Some(b) => b.clone(),
            None => return Ok(false), // 无 body 且非状态导入,不经执行改宪法,跳过
        };

        let parent_hash = *params.header.parent_hash();
        let block = Block::new(params.header.clone(), body);

        // 在父状态上只读执行该区块(不提交),取后置存储变更。
        let api = self.client.runtime_api();
        api.execute_block(parent_hash, block.into())
            .map_err(|e| format!("只读执行区块失败:{e}"))?;
        let parent_state = self
            .backend
            .state_at(parent_hash, TrieCacheContext::Untrusted)
            .map_err(|e| format!("取父状态失败:{e}"))?;
        let changes = api
            .into_storage_changes(&parent_state, parent_hash)
            .map_err(|e| format!("提取存储变更失败:{e}"))?;

        // 快路径:本块是否动过立法院模块存储?未动则(归纳)不可修改条款不变,合规。
        let prefix = storage_key::pallet_prefix();
        let delta: BTreeMap<Vec<u8>, Option<Vec<u8>>> =
            changes.main_storage_changes.into_iter().collect();
        if !delta.keys().any(|k| k.starts_with(&prefix)) {
            return Ok(false);
        }

        // 后置状态读取器:命中变更取变更值(Some=改、None=删),否则回落父状态(已提交)。
        let read_post = |key: &[u8]| -> Option<Vec<u8>> {
            match delta.get(key) {
                Some(value) => value.clone(),
                None => self
                    .client
                    .storage(parent_hash, &StorageKey(key.to_vec()))
                    .ok()
                    .flatten()
                    .map(|data| data.0),
            }
        };

        match check_immutable_articles(read_post, &self.reference) {
            Ok(()) => Ok(false),
            Err(reason) => {
                log::error!(
                    target: "constitution-guard",
                    "拒绝区块 #{} ({:?}):不可修改条款被改动 —— {:?}",
                    params.header.number(),
                    params.post_hash(),
                    reason,
                );
                Ok(true)
            }
        }
    }
}

#[async_trait::async_trait]
impl<I> BlockImport<Block> for ConstitutionGuard<I>
where
    I: BlockImport<Block, Error = ConsensusError> + Send + Sync,
{
    type Error = ConsensusError;

    async fn check_block(
        &self,
        block: BlockCheckParams<Block>,
    ) -> Result<ImportResult, Self::Error> {
        self.inner.check_block(block).await
    }

    async fn import_block(
        &self,
        params: BlockImportParams<Block>,
    ) -> Result<ImportResult, Self::Error> {
        // warp/状态同步块:不经执行,直接导入下载来的状态 → 先让内层导入,再校验导入后的
        // 宪法不变式(防新节点 warp 到已篡改不可修改条款的状态)。违规则拒(KnownBad),
        // warp 目标被否,节点换 peer / 落回全量同步。
        if params.with_state() {
            let post_hash = params.post_hash();
            let result = self.inner.import_block(params).await?;
            return match self.verify_committed_state(post_hash) {
                Ok(()) => Ok(result),
                Err(reason) => {
                    log::error!(
                        target: "constitution-guard",
                        "拒绝 warp/状态导入 ({post_hash:?}):宪法不变式被破坏 —— {reason}",
                    );
                    Ok(ImportResult::KnownBad)
                }
            };
        }

        // 普通(执行型)块:执行前判定,违规 KnownBad(内层永不被调用)。
        match self.detect_violation(&params) {
            Ok(true) => Ok(ImportResult::KnownBad),
            Ok(false) => self.inner.import_block(params).await,
            // 守卫自身执行/取数出错不等于「确认违规」:同一 STF 决定论下内层会一致处理,
            // 故放行内层而非据自身故障拒块(避免守卫 bug 误停全链);异常已记日志待排查。
            Err(why) => {
                log::warn!(target: "constitution-guard", "守卫判定未完成,放行内层:{why}");
                self.inner.import_block(params).await
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- 测试夹具:构造一份 LawVersion 字节(可指定某条文内容)----
    fn article_bytes(number: u32, body: &str) -> MArticle {
        MArticle {
            number,
            title: format!("第{number}条").into_bytes(),
            title_en: Some(format!("Article {number}").into_bytes()),
            body: body.as_bytes().to_vec(),
            body_en: Some("EN".as_bytes().to_vec()),
            clauses: Vec::new(),
        }
    }

    /// 构造 LawVersion(version, 给定条文)的 SCALE 字节 + 哑尾(模拟链端完整编码)。
    fn law_version_scale(version: u32, articles: Vec<MArticle>) -> Vec<u8> {
        let chapter = MChapter {
            number: 1,
            title: "第一章".as_bytes().to_vec(),
            title_en: Some("Chapter I".as_bytes().to_vec()),
            sections: vec![MSection {
                number: 1,
                title: "第一节".as_bytes().to_vec(),
                title_en: Some("Section 1".as_bytes().to_vec()),
                articles,
            }],
        };
        let mut bytes = Vec::new();
        CONSTITUTION_LAW_ID.encode_to(&mut bytes); // law_id
        version.encode_to(&mut bytes); // version
        "公民宪法".as_bytes().to_vec().encode_to(&mut bytes); // title
        Option::<Vec<u8>>::None.encode_to(&mut bytes); // title_en
        vec![chapter].encode_to(&mut bytes); // chapters
        [0u8; 32].encode_to(&mut bytes); // content_hash(哑尾)
        0u8.encode_to(&mut bytes); // vote_type
        0u64.encode_to(&mut bytes); // proposal_id
        0u32.encode_to(&mut bytes); // published_at
        0u32.encode_to(&mut bytes); // effective_at
        bytes
    }

    /// 构造 Law(current_version) 的 SCALE 字节 + 哑尾。
    /// Law(0) 字节:tier=Constitution、scope=0、给定 houses/current_version/status。
    fn law_scale_full(
        current_version: u32,
        status: u8,
        houses: Vec<([u8; 4], [u8; 32])>,
    ) -> Vec<u8> {
        let mut bytes = Vec::new();
        CONSTITUTION_LAW_ID.encode_to(&mut bytes); // law_id
        TIER_CONSTITUTION.encode_to(&mut bytes); // tier = Constitution(0)
        0u32.encode_to(&mut bytes); // scope_code = 0
        houses.encode_to(&mut bytes); // houses
        current_version.encode_to(&mut bytes); // current_version
        status.encode_to(&mut bytes); // status
        bytes
    }

    /// 默认合法 Law(0):Effective、空 houses。
    fn law_scale(current_version: u32) -> Vec<u8> {
        law_scale_full(current_version, 1 /* Effective */, Vec::new())
    }

    fn laws_by_scope_entry(list: Vec<u64>) -> (Vec<u8>, Vec<u8>) {
        (storage_key::laws_by_scope_constitution(), list.encode())
    }

    /// 一份完整合法当前态:Laws[0] + LawVersions[0][version] + LawsByScope[宪法][0]=[0]。
    fn valid_current_state(version: u32, articles: Vec<MArticle>) -> Vec<(Vec<u8>, Vec<u8>)> {
        vec![
            (storage_key::law(CONSTITUTION_LAW_ID), law_scale(version)),
            (
                storage_key::law_version(CONSTITUTION_LAW_ID, version),
                law_version_scale(version, articles),
            ),
            laws_by_scope_entry(vec![CONSTITUTION_LAW_ID]),
        ]
    }

    /// 用一组 (key,value) 建一个 RAW 读取闭包。
    fn reader(entries: Vec<(Vec<u8>, Vec<u8>)>) -> impl Fn(&[u8]) -> Option<Vec<u8>> {
        let map: BTreeMap<Vec<u8>, Vec<u8>> = entries.into_iter().collect();
        move |k: &[u8]| map.get(k).cloned()
    }

    // 取全部不可修改条号 + 几条可变条文,组成一份"创世"状态。
    fn genesis_articles() -> Vec<MArticle> {
        let mut arts: Vec<MArticle> = IMMUTABLE_CONSTITUTION_ARTICLES
            .iter()
            .map(|&n| article_bytes(n, &format!("不可修改条款 {n} 原文")))
            .collect();
        arts.push(article_bytes(5, "可变条款原文")); // 一条可变条
        arts
    }

    fn genesis_state() -> Vec<(Vec<u8>, Vec<u8>)> {
        valid_current_state(1, genesis_articles())
    }

    /// 不可修改条款原样 + 可变条改动的新版本条文(version=2 用)。
    fn amended_articles(immutable_body: impl Fn(u32) -> String) -> Vec<MArticle> {
        let mut arts: Vec<MArticle> = IMMUTABLE_CONSTITUTION_ARTICLES
            .iter()
            .map(|&n| article_bytes(n, &immutable_body(n)))
            .collect();
        arts.push(article_bytes(5, "可变条款原文"));
        arts
    }

    fn immutable_intact(n: u32) -> String {
        format!("不可修改条款 {n} 原文")
    }

    #[test]
    fn key_derivation_is_stable_and_distinct() {
        // 同输入稳定、不同输入相异、含正确前缀。
        assert_eq!(storage_key::law(0), storage_key::law(0));
        assert_ne!(storage_key::law(0), storage_key::law(1));
        assert_ne!(
            storage_key::law_version(0, 1),
            storage_key::law_version(0, 2)
        );
        assert!(storage_key::law(0).starts_with(&storage_key::pallet_prefix()));
        assert!(storage_key::law_version(0, 1).starts_with(&storage_key::pallet_prefix()));
    }

    #[test]
    fn reference_derives_all_immutable_articles() {
        let r = ImmutableReference::from_raw_reader(reader(genesis_state())).expect("应能派生");
        for &n in IMMUTABLE_CONSTITUTION_ARTICLES.iter() {
            assert!(r.articles.contains_key(&n), "基准缺条号 {n}");
        }
    }

    #[test]
    fn passes_when_immutable_unchanged_even_if_mutable_changed() {
        let reference = ImmutableReference::from_raw_reader(reader(genesis_state())).unwrap();
        // 新版本:不可修改条款原样,只改了可变条 5,bump current_version=2。
        let mut arts = amended_articles(immutable_intact);
        arts[IMMUTABLE_CONSTITUTION_ARTICLES.len()] = article_bytes(5, "可变条款已被合法修改");
        let state = valid_current_state(2, arts);
        assert_eq!(check_immutable_articles(reader(state), &reference), Ok(()));
    }

    #[test]
    fn rejects_when_an_immutable_article_mutated() {
        let reference = ImmutableReference::from_raw_reader(reader(genesis_state())).unwrap();
        let arts = amended_articles(|n| {
            if n == 1 {
                "第一条被篡改".to_string()
            } else {
                immutable_intact(n)
            }
        });
        let state = valid_current_state(2, arts);
        assert_eq!(
            check_immutable_articles(reader(state), &reference),
            Err(GuardError::ImmutableArticleMutated(1))
        );
    }

    #[test]
    fn rejects_when_an_immutable_article_deleted() {
        let reference = ImmutableReference::from_raw_reader(reader(genesis_state())).unwrap();
        // 删掉第 17 条。
        let arts: Vec<MArticle> = IMMUTABLE_CONSTITUTION_ARTICLES
            .iter()
            .filter(|&&n| n != 17)
            .map(|&n| article_bytes(n, &immutable_intact(n)))
            .collect();
        let state = valid_current_state(2, arts);
        assert_eq!(
            check_immutable_articles(reader(state), &reference),
            Err(GuardError::ImmutableArticleMissing(17))
        );
    }

    #[test]
    fn rejects_when_constitution_storage_missing() {
        let reference = ImmutableReference::from_raw_reader(reader(genesis_state())).unwrap();
        assert_eq!(
            check_immutable_articles(reader(vec![]), &reference),
            Err(GuardError::ConstitutionLawMissing)
        );
    }

    // ── H1 元数据 / houses / 唯一性不变式 ──

    /// 在合法当前态基础上,替换某个 key 的值后跑校验。
    fn check_with_override(version: u32, key: Vec<u8>, value: Vec<u8>) -> Result<(), GuardError> {
        let reference = ImmutableReference::from_raw_reader(reader(genesis_state())).unwrap();
        let mut state = valid_current_state(version, amended_articles(immutable_intact));
        state.retain(|(k, _)| k != &key);
        state.push((key, value));
        check_immutable_articles(reader(state), &reference)
    }

    #[test]
    fn rejects_when_tier_changed() {
        // tier 改为 National(1),其余合法。
        let mut law = Vec::new();
        CONSTITUTION_LAW_ID.encode_to(&mut law);
        1u8.encode_to(&mut law); // tier = National
        0u32.encode_to(&mut law);
        Vec::<([u8; 4], [u8; 32])>::new().encode_to(&mut law);
        2u32.encode_to(&mut law);
        1u8.encode_to(&mut law); // Effective
        assert_eq!(
            check_with_override(2, storage_key::law(CONSTITUTION_LAW_ID), law),
            Err(GuardError::ConstitutionTierChanged)
        );
    }

    #[test]
    fn rejects_when_repealed() {
        let law = law_scale_full(2, LAW_STATUS_REPEALED, Vec::new());
        assert_eq!(
            check_with_override(2, storage_key::law(CONSTITUTION_LAW_ID), law),
            Err(GuardError::ConstitutionRepealed)
        );
    }

    #[test]
    fn allows_pending_status_during_amendment() {
        // status=Pending(0) 不应被拒(合法修宪窗口)。
        let law = law_scale_full(2, LAW_STATUS_PENDING, Vec::new());
        assert_eq!(
            check_with_override(2, storage_key::law(CONSTITUTION_LAW_ID), law),
            Ok(())
        );
    }

    #[test]
    fn rejects_when_houses_changed() {
        // houses 改为非空(与创世空 houses 不一致)。
        let law = law_scale_full(2, 1, vec![(*b"NLG\0", [9u8; 32])]);
        assert_eq!(
            check_with_override(2, storage_key::law(CONSTITUTION_LAW_ID), law),
            Err(GuardError::ConstitutionHousesChanged)
        );
    }

    #[test]
    fn rejects_when_constitution_not_unique() {
        // LawsByScope[宪法][0] 多出第二部宪法 law_id=1。
        assert_eq!(
            check_with_override(
                2,
                storage_key::laws_by_scope_constitution(),
                vec![0u64, 1u64].encode()
            ),
            Err(GuardError::ConstitutionNotUnique)
        );
    }

    #[test]
    fn rejects_when_constitution_hidden_from_scope_list() {
        // LawsByScope[宪法][0] 被清空(law_id=0 被隐藏)。
        assert_eq!(
            check_with_override(
                2,
                storage_key::laws_by_scope_constitution(),
                Vec::<u64>::new().encode()
            ),
            Err(GuardError::ConstitutionNotUnique)
        );
    }

    #[test]
    fn effective_version_skips_pending() {
        // Effective → current;Pending → current-1;v1 不回退。
        assert_eq!(
            effective_version_of_law(&law_scale_full(3, 1, vec![])).unwrap(),
            3
        );
        assert_eq!(
            effective_version_of_law(&law_scale_full(3, 0, vec![])).unwrap(),
            2
        );
        assert_eq!(
            effective_version_of_law(&law_scale_full(1, 0, vec![])).unwrap(),
            1
        );
    }

    // ---- manifest 交叉校验 ----
    fn manifest_scale(numbers: Vec<u32>, hashes: Vec<[u8; 32]>) -> Vec<u8> {
        let mut bytes = Vec::new();
        numbers.encode_to(&mut bytes);
        hashes.encode_to(&mut bytes);
        bytes
    }

    fn correct_manifest(reference: &ImmutableReference) -> Vec<u8> {
        let numbers: Vec<u32> = IMMUTABLE_CONSTITUTION_ARTICLES.to_vec();
        let hashes: Vec<[u8; 32]> = numbers
            .iter()
            .map(|n| sp_core::blake2_256(&reference.articles[n]))
            .collect();
        manifest_scale(numbers, hashes)
    }

    #[test]
    fn manifest_passes_when_list_and_hashes_consistent() {
        let reference = ImmutableReference::from_raw_reader(reader(genesis_state())).unwrap();
        assert!(verify_manifest(&correct_manifest(&reference), &reference).is_ok());
    }

    #[test]
    fn manifest_rejects_wrong_list() {
        let reference = ImmutableReference::from_raw_reader(reader(genesis_state())).unwrap();
        // 清单缺条 → 与二进制不一致。
        let bad = manifest_scale(vec![1, 2, 3], vec![[0u8; 32]; 3]);
        assert!(verify_manifest(&bad, &reference).is_err());
    }

    #[test]
    fn manifest_rejects_tampered_hash() {
        let reference = ImmutableReference::from_raw_reader(reader(genesis_state())).unwrap();
        let numbers: Vec<u32> = IMMUTABLE_CONSTITUTION_ARTICLES.to_vec();
        let mut hashes: Vec<[u8; 32]> = numbers
            .iter()
            .map(|n| sp_core::blake2_256(&reference.articles[n]))
            .collect();
        hashes[0] = [9u8; 32]; // 谎报第一条摘要
        assert!(verify_manifest(&manifest_scale(numbers, hashes), &reference).is_err());
    }

    #[test]
    fn render_rebuilds_expected_anchors() {
        let scale = law_version_scale(1, vec![article_bytes(1, "正文")]);
        let html = render_constitution_html(&scale).expect("应能重建");
        assert!(html.starts_with("<!DOCTYPE html>"));
        assert!(html.trim_end().ends_with("</html>"));
        assert!(html.contains("href=\"#article-1\""));
        assert!(html.contains("id=\"article-1\" class=\"block article-block\""));
    }
}
