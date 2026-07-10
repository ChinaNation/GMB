//! 宪法节点能力统一入口(ADR-027)。
//!
//! 宪法已迁入链上立法院模块(`legislation-yuan`,`law_id=0`、`tier=宪法`,创世注入),
//! 唯一真源 = 链上法律。本文件统一承载节点端两件事:
//!
//! 1. **渲染**(展示):从链上结构化宪法(章>节>条>款 + 中英双语)重建《公民宪法》HTML,
//!    复用原 CSS 外壳,供桌面端「公民宪法」tab 显示(`constitution_getDocument` RPC)。
//! 2. **不可修改条款守卫**(L2 共识层):宪法第 1/2/3/17/19/24/34/42 条为「不可修改条款」,
//!    本守卫在区块导入时逐块校验这些条文与**创世(block#0)逐字一致**,违者拒块。
//!    执法逻辑在 runtime 之外的节点二进制里,清单(`primitives::IMMUTABLE_CONSTITUTION_ARTICLES`)
//!    编译进二进制 —— 故 setCode / migration / 改清单常量都改不动这些条文;唯一修改路径 =
//!    改创世(创世哈希变 = 新链)或改节点二进制(硬分叉),即「只能重新创世」。详见 ADR-027 §7。

use std::collections::BTreeMap;
use std::sync::Arc;

use codec::{Decode, Encode};
use sc_client_api::backend::{Backend as _, TrieCacheContext};
use sc_client_api::StorageProvider;
use sc_consensus::{
    BlockCheckParams, BlockImport, BlockImportParams, ImportResult, ImportedState, StateAction,
    StorageChanges,
};
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
const LAW_STATUS_REPEALED: u8 = 2;
/// 表决类型「特别案」的 wire 值(`legislation-yuan::VoteType::Special.as_u8()`)。
/// 由 legislation-yuan 测试 `enum_discriminants_match_node_guard` 交叉钉死,防漂移。
/// 用途:核心章(第一章总则)条款改动必须记录为特别案(宪法第十九条 node 背书)。
const LEG_VOTE_SPECIAL: u8 = 4;

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

/// 只解码 `LawVersion` 前缀(law_id..vote_type);其后字段(proposal_id..effective_at)顺序解码到
/// vote_type 即停。chapters 用于条文比对,vote_type 用于核心章档位背书(第十九条)。
#[derive(Decode)]
#[allow(dead_code)] // law_id/version/title/title_en/content_hash 仅占位保持字段序。
struct MLawVersionHead {
    law_id: u64,
    version: u32,
    title: Vec<u8>,
    title_en: Option<Vec<u8>>,
    chapters: Vec<MChapter>,
    content_hash: [u8; 32],
    vote_type: u8,
}

/// 链上 `LawVersionLabel` 镜像。版本号排序仍来自 `LawVersion.version`,本结构只负责展示名。
#[derive(Decode)]
struct MLawVersionLabel {
    title: Vec<u8>,
    title_en: Option<Vec<u8>>,
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
    effective_version: Option<u32>,
    latest_version: u32,
    pending_version: Option<u32>,
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
const VERSION_LABEL_MARKER: &str = "<!--CONSTITUTION_VERSION_LABEL-->";
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

/// 当前**生效中**的版本号(供桌面端展示,避免提前显示待生效版)。
pub fn effective_version_of_law(law_scale: &[u8]) -> Result<u32, String> {
    let law = decode_law_head(law_scale).map_err(|e| format!("宪法 Law 解码失败:{e:?}"))?;
    law.effective_version
        .ok_or_else(|| "宪法尚无生效版本".to_string())
}

/// 解码链上不可修改条款 manifest,供桌面端展示徽章。
pub fn immutable_article_numbers(manifest_scale: &[u8]) -> Result<Vec<u32>, String> {
    let manifest = MImmutableManifest::decode(&mut &manifest_scale[..])
        .map_err(|e| format!("宪法不可修改条款 manifest 解码失败:{e}"))?;
    Ok(manifest.article_numbers)
}

/// 把链上结构化宪法 SCALE 字节(`LawVersion` 编码)重建为完整 HTML 文档。
pub fn render_constitution_html(
    law_version_scale: &[u8],
    immutable_article_numbers: &[u32],
    law_version_label_scale: Option<&[u8]>,
) -> Result<String, String> {
    let law = MLawVersionHead::decode(&mut &law_version_scale[..])
        .map_err(|e| format!("宪法 LawVersion 解码失败:{e}"))?;
    let version_label = law_version_label_scale
        .map(|raw| {
            MLawVersionLabel::decode(&mut &raw[..])
                .map_err(|e| format!("宪法 LawVersionLabel 解码失败:{e}"))
        })
        .transpose()?;
    let version_label_html = render_version_label(law.version, version_label.as_ref());

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
                let (immutable_badge_cn, immutable_badge_en) = if immutable_article_numbers
                    .contains(&article.number)
                {
                    (
                            "<span class=\"immutable-badge immutable-badge-cn\">不可修改条款</span>",
                            "<span class=\"immutable-badge immutable-badge-en\">Immutable Clause</span>",
                        )
                } else {
                    ("", "")
                };
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
                    "<article id=\"article-{an}\" class=\"block article-block\">\n  <h3 class=\"article-title\">\n    <span class=\"cn heading-cn\">{a_cn}{immutable_badge_cn}</span>\n    <span class=\"en heading-en\">{a_en}{immutable_badge_en}</span>\n  </h3>\n\n{paragraphs}</article>\n\n",
                    an = article.number,
                ));
            }
        }
    }

    // 把目录与正文填入外壳两个占位标记 —— 整页结构只在 constitution_shell.html 一处维护。
    Ok(SHELL
        .replace(VERSION_LABEL_MARKER, &version_label_html)
        .replace(TOC_MARKER, &toc)
        .replace(CONTENT_MARKER, &content))
}

fn render_version_label(version: u32, label: Option<&MLawVersionLabel>) -> String {
    let fallback = format!("v{version}");
    let (cn, en) = if let Some(label) = label {
        let cn = text(&label.title);
        let en = opt_text(&label.title_en);
        (
            if cn.trim().is_empty() {
                fallback.clone()
            } else {
                cn
            },
            if en.trim().is_empty() {
                fallback.clone()
            } else {
                en
            },
        )
    } else {
        (fallback.clone(), fallback)
    };
    format!(
        "<span class=\"doc-version-cn\">{}</span><span class=\"doc-version-en\">{}</span>",
        esc(&cn),
        esc(&en),
    )
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

    /// `LegislationYuan::LawVersionLabels[law_id][version]` 的完整存储 key。
    pub fn law_version_label(law_id: u64, version: u32) -> Vec<u8> {
        let mut k = map_prefix(b"LawVersionLabels");
        k.extend_from_slice(&blake2_128_concat(&law_id.encode()));
        k.extend_from_slice(&blake2_128_concat(&version.encode()));
        k
    }

    /// `LegislationYuan::ConstitutionImmutableManifest`(StorageValue,无 key hash)的完整 key。
    pub fn manifest() -> Vec<u8> {
        map_prefix(b"ConstitutionImmutableManifest")
    }

    /// `LegislationYuan::ConstitutionAmendmentProof[version]`(Blake2_128Concat)的完整 key。
    /// 核心修宪的永久公投凭据 `(eligible, yes, no)`,供守卫背书(第十九条,ADR-027 §6.3)。
    pub fn constitution_amendment_proof(version: u32) -> Vec<u8> {
        let mut k = map_prefix(b"ConstitutionAmendmentProof");
        k.extend_from_slice(&blake2_128_concat(&version.encode()));
        k
    }

    /// `LegislationYuan::ConstitutionGuardVoteProof[version]`(Blake2_128Concat)的完整 key。
    /// 修宪的永久护宪大法官终审凭据(赞成票数),供守卫背书(第21条,ADR-027 §6.3)。
    pub fn constitution_guard_vote_proof(version: u32) -> Vec<u8> {
        let mut k = map_prefix(b"ConstitutionGuardVoteProof");
        k.extend_from_slice(&blake2_128_concat(&version.encode()));
        k
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
    /// 核心章(第一章总则,创世口径)某条被修改/删除/移出核心章,但该版本未记录为特别案表决
    /// (违反宪法第十九条:核心章条款修改须走特别案 + 强制公投)。参数为条号。
    CoreClauseNotSpecial(u32),
    /// 核心章条款有改动的版本缺失强制公投凭据 `ConstitutionAmendmentProof[version]`
    /// (第十九条:核心修宪须经公投)。参数为版本号。
    CoreClauseReferendumMissing(u32),
    /// 核心章条款有改动的版本,其公投凭据未达通过口径(≥70% 参与 + ≥70% 赞成)。参数为版本号。
    CoreClauseReferendumNotPassed(u32),
    /// 修宪版本(v>创世)缺失护宪大法官终审凭据 `ConstitutionGuardVoteProof[version]`
    /// (第21条:一切修宪须经护宪终审)。参数为版本号。
    GuardReviewMissing(u32),
    /// 修宪版本的护宪终审凭据未达通过口径(4 名及以上赞成)。参数为版本号。
    GuardReviewNotPassed(u32),
}

/// 不可修改基准(创世/block#0):
/// - `articles`:不可修改条款(第 1/2/3/17/19/24/34/42 条)的规范 SCALE 字节;
/// - `core_articles`:核心章(第一章总则)**非禁改**条款的创世规范字节(条号→字节);
/// - `houses`:宪法可修订机构。
///
/// `core_articles` 用于第十九条章→档位的 node 背书:任一核心条款相对创世被
/// 修改/删除/移出核心章,则承载它的版本必须记录为特别案表决(见 [`check_core_chapter_tier`])。
pub struct ImmutableReference {
    articles: BTreeMap<u32, Vec<u8>>,
    core_articles: BTreeMap<u32, Vec<u8>>,
    houses: Vec<([u8; 4], [u8; 32])>,
}

impl ImmutableReference {
    /// 从一个 RAW 存储读取闭包(应指向 block#0 创世状态)派生基准:
    /// 读 `Laws[0]` 取 `houses`,读创世版本 `LawVersions[0][1]` 取不可修改条款与核心章条款规范字节。
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

        // 不可修改条款基准:逐字冻结。
        let mut articles = BTreeMap::new();
        for &n in IMMUTABLE_CONSTITUTION_ARTICLES.iter() {
            let article =
                find_article(&head.chapters, n).ok_or(GuardError::ImmutableArticleMissing(n))?;
            articles.insert(n, article.encode());
        }
        // 核心章基准:创世第一章总则里的**非禁改**条款(可经特别案修订,故存字节供 diff)。
        let mut core_articles = BTreeMap::new();
        if let Some(core_chapter) = head.chapters.first() {
            for article in core_chapter.sections.iter().flat_map(|s| s.articles.iter()) {
                if !IMMUTABLE_CONSTITUTION_ARTICLES.contains(&article.number) {
                    core_articles.insert(article.number, article.encode());
                }
            }
        }
        Ok(Self {
            articles,
            core_articles,
            houses: law.houses,
        })
    }
}

/// 纯判定:给定一个指向**目标区块后置状态**的 RAW 读取闭包,校验宪法全部不变式:
/// ① Law 元数据(tier=Constitution、scope=0、status≠Repealed、houses=创世);
/// ② 层级唯一性(`LawsByScope[宪法][0] == [0]`);③ 不可修改条款逐字 == 创世基准;
/// ④ 核心章(第一章总则)条款改动须记录为特别案表决(第十九条,见 [`check_core_chapter_tier`])
///    且挂一份过公投口径的永久凭据(见 [`check_core_referendum_proof`]);
/// ⑤ 一切修宪版本(v>创世)须挂一份过 4/7 口径的护宪大法官终审凭据(第21条,见 [`check_guard_review_proof`])。
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
    // 生效版本必须存在；待生效版本若存在也必须立即接受同一套不可修改条款约束。
    let effective_version = law.effective_version.ok_or(GuardError::LawVersionMissing)?;
    check_immutable_version(&read_raw, reference, effective_version)?;
    if let Some(pending_version) = law.pending_version {
        check_immutable_version(&read_raw, reference, pending_version)?;
    }
    Ok(())
}

/// 校验指定宪法版本里的不可修改条款是否仍与创世基准逐字一致。
fn check_immutable_version<F>(
    read_raw: &F,
    reference: &ImmutableReference,
    version: u32,
) -> Result<(), GuardError>
where
    F: Fn(&[u8]) -> Option<Vec<u8>>,
{
    let version_bytes = read_raw(&storage_key::law_version(CONSTITUTION_LAW_ID, version))
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
    // 核心章条款(第一章总则非禁改条)改动:① 必须记录为特别案;② 必须挂合格公投凭据(宪法第十九条 node 背书)。
    let core_changed = check_core_chapter_tier(&head, reference)?;
    if core_changed {
        check_core_referendum_proof(read_raw, version)?;
    }
    // 一切修宪版本(v > 创世版本)都须挂合格护宪大法官终审凭据(宪法第21条 node 背书,含一般章重要案)。
    if version > GENESIS_CONSTITUTION_VERSION {
        check_guard_review_proof(read_raw, version)?;
    }
    Ok(())
}

/// 修宪版本(生效/待生效,`version > 创世`)必须挂一份**通过口径**的永久护宪终审凭据
/// `ConstitutionGuardVoteProof[version]`(第21条 node 背书:一切修宪须经护宪大法官 4/7 终审,含一般章)。
/// 凭据是 legislation-yuan **永久存储**(不受 votingengine 90 天清理影响),故可对任意修宪版本随时校验。
/// 口径复用 `primitives::constitution::guard_review_passed`(与链端结算共用单源)。
fn check_guard_review_proof<F>(read_raw: &F, version: u32) -> Result<(), GuardError>
where
    F: Fn(&[u8]) -> Option<Vec<u8>>,
{
    let bytes = read_raw(&storage_key::constitution_guard_vote_proof(version))
        .ok_or(GuardError::GuardReviewMissing(version))?;
    let approve =
        u32::decode(&mut &bytes[..]).map_err(|_| GuardError::GuardReviewMissing(version))?;
    if !primitives::constitution::guard_review_passed(approve) {
        return Err(GuardError::GuardReviewNotPassed(version));
    }
    Ok(())
}

/// 核心章(第一章总则,创世口径)条款的档位背书(宪法第十九条 node 侧强制)。
/// 对每个创世核心条款:若其相对创世基准被**修改/删除/移出核心章**,则本版本必须记录为
/// 特别案表决(`vote_type == LEG_VOTE_SPECIAL`),否则拒块;核心条款未变则不约束档位
/// (一般章条款可走重要案)。仅盯创世核心集且按条号定位,故不受章节重排影响。
///
/// 返回 `true` 表示本版本相对创世核心基准**有改动**(调用方据此再校验公投凭据)。
/// 本函数盯的是版本**记录的** `vote_type`(使 setCode 无法静默降级为重要案);
/// 「公投是否真的通过」由 [`check_core_referendum_proof`] 读永久凭据补上。
fn check_core_chapter_tier(
    head: &MLawVersionHead,
    reference: &ImmutableReference,
) -> Result<bool, GuardError> {
    // 当前版本核心章(chapters[0])的条号集,用于判定核心条款是否被移出核心章。
    let core_chapter_now: Vec<u32> = head
        .chapters
        .first()
        .into_iter()
        .flat_map(|c| c.sections.iter())
        .flat_map(|s| s.articles.iter())
        .map(|a| a.number)
        .collect();
    let mut core_changed = false;
    for (&n, baseline) in reference.core_articles.iter() {
        let current = find_article(&head.chapters, n);
        let content_same = current.map(|a| a.encode()).as_deref() == Some(baseline.as_slice());
        let in_core_chapter = core_chapter_now.contains(&n);
        // 修改(内容变)/删除(找不到)/移出核心章(不在 chapters[0])任一 → 有改动,且须特别案。
        if !content_same || !in_core_chapter {
            core_changed = true;
            if head.vote_type != LEG_VOTE_SPECIAL {
                return Err(GuardError::CoreClauseNotSpecial(n));
            }
        }
    }
    Ok(core_changed)
}

/// 核心章有改动的版本必须挂一份**通过口径**的强制公投凭据 `ConstitutionAmendmentProof[version]`
/// (第十九条 node 背书的公投凭据层:不止记录 `vote_type=Special`,还须有过公投口径的计票)。
/// 凭据是 legislation-yuan **永久存储**(不受 votingengine 90 天清理影响),故可对生效/待生效版本随时校验,
/// 无转移块检测。口径复用 `primitives::constitution::referendum_passed`(与链端结算共用单源)。
fn check_core_referendum_proof<F>(read_raw: &F, version: u32) -> Result<(), GuardError>
where
    F: Fn(&[u8]) -> Option<Vec<u8>>,
{
    let bytes = read_raw(&storage_key::constitution_amendment_proof(version))
        .ok_or(GuardError::CoreClauseReferendumMissing(version))?;
    let (eligible, yes, no) = <(u64, u64, u64)>::decode(&mut &bytes[..])
        .map_err(|_| GuardError::CoreClauseReferendumMissing(version))?;
    if !primitives::constitution::referendum_passed(eligible, yes, no) {
        return Err(GuardError::CoreClauseReferendumNotPassed(version));
    }
    Ok(())
}

/// 是否必须跑完整宪法不变式校验。普通块只要触及立法院存储或 `:code` runtime 升级,
/// 就不能走快路径;其余块按归纳假设跳过。
fn needs_full_invariant_check(delta: &BTreeMap<Vec<u8>, Option<Vec<u8>>>) -> bool {
    let prefix = storage_key::pallet_prefix();
    delta.keys().any(|k| k.starts_with(&prefix))
        || delta.contains_key(sp_storage::well_known_keys::CODE)
}

/// 从 warp/状态导入的完整下载态中抽出立法院模块键,在提交前执行同一套宪法不变式校验。
fn check_imported_state_key_values<'a, I>(
    pairs: I,
    reference: &ImmutableReference,
) -> Result<(), String>
where
    I: IntoIterator<Item = &'a (Vec<u8>, Vec<u8>)>,
{
    let prefix = storage_key::pallet_prefix();
    let mut map: BTreeMap<Vec<u8>, Vec<u8>> = BTreeMap::new();
    for (key, value) in pairs {
        if key.starts_with(&prefix) {
            map.insert(key.clone(), value.clone());
        }
    }
    check_immutable_articles(|key| map.get(key).cloned(), reference).map_err(|e| format!("{e:?}"))
}

/// 从 warp/状态导入的完整下载态中抽出立法院模块键,在提交前执行同一套宪法不变式校验。
fn check_imported_state_immutable(
    imported: &ImportedState<Block>,
    reference: &ImmutableReference,
) -> Result<(), String> {
    check_imported_state_key_values(
        imported
            .state
            .0
            .iter()
            .flat_map(|level| level.key_values.iter()),
        reference,
    )
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
        .map_err(|e| format!("护宪守卫:创世不可修改条款基准派生失败:{e:?}"))?;

        // L3 启动交叉校验:创世 manifest ↔ 二进制清单 ↔ 创世条文三者一致,否则拒绝启动。
        let manifest_bytes = client
            .storage(genesis_hash, &StorageKey(storage_key::manifest()))
            .ok()
            .flatten()
            .map(|data| data.0)
            .ok_or_else(|| "护宪守卫:创世缺不可修改条款 manifest".to_string())?;
        verify_manifest(&manifest_bytes, &reference)
            .map_err(|e| format!("护宪守卫:启动交叉校验失败:{e}"))?;

        Ok(Self {
            inner,
            client,
            backend,
            reference,
        })
    }

    /// **提交前**校验 warp/状态导入块携带的下载态宪法不变式(P1:vendored GRANDPA 在 `inner.import_block`
    /// 内即把状态置 finalized 落库,post-import 拒块无法回滚,故必须在调用 inner **之前**校验)。
    /// 从 `params.state_action` 的 `ImportedState` 抽出立法院模块前缀键(仅几 KB),据此跑全套不变式。
    /// `Ok(())` = 合规(可提交);`Err` = 违规或无法抽取(拒绝,fail-closed —— 不调用 inner,什么都不落库)。
    fn verify_imported_state(&self, params: &BlockImportParams<Block>) -> Result<(), String> {
        let imported = match &params.state_action {
            StateAction::ApplyChanges(StorageChanges::Import(imported)) => imported,
            // warp 状态形态非预期 → 无法 pre-commit 校验、post-commit 又不可回滚 → 拒绝。
            _ => return Err("warp 状态非 ApplyChanges(Import) 形态,拒绝(无法提交前校验)".into()),
        };
        check_imported_state_immutable(imported, &self.reference)
    }

    /// 计算普通(执行型)区块后置状态是否违反宪法不变式。warp/状态导入块不走此路径(见 `import_block`)。
    /// `Ok(true)` = 确认违规(拒块);`Ok(false)` = 合规;`Err` = 无法判定(`import_block` fail-closed 拒块)。
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

        // 快路径:本块既未动立法院模块存储、也未升级 runtime(`:code`)→ 归纳不可修改条款不变,合规。
        // runtime 升级(setCode)是高危块:即便 delta 未触立法院前缀也强制走全量不变式校验(P2/P3)。
        let delta: BTreeMap<Vec<u8>, Option<Vec<u8>>> =
            changes.main_storage_changes.into_iter().collect();
        if !needs_full_invariant_check(&delta) {
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
        // warp/状态同步块:vendored GRANDPA 在 inner 内即把状态置 finalized 落库,无法事后回滚,
        // 故必须**提交前**校验下载态的宪法不变式(P1)。违规/无法抽取 → KnownBad(不调用 inner,什么都不落库)。
        if params.with_state() {
            return match self.verify_imported_state(&params) {
                Ok(()) => self.inner.import_block(params).await,
                Err(reason) => {
                    log::error!(
                        target: "constitution-guard",
                        "拒绝 warp/状态导入 ({:?}):宪法不变式校验未通过 —— {reason}",
                        params.post_hash(),
                    );
                    Ok(ImportResult::KnownBad)
                }
            };
        }

        // 普通(执行型)块:执行前判定,违规 KnownBad(内层永不被调用)。
        match self.detect_violation(&params) {
            Ok(true) => Ok(ImportResult::KnownBad),
            Ok(false) => self.inner.import_block(params).await,
            // P2 fail-closed:守卫自身执行/取数失败(无法读父状态/无法执行/无法取变更)→ 拒块,
            // 不放行未经校验的块。代价是守卫机器 bug 可能误停链,但对「不可修改条款永不可破坏」
            // 这是刻意的安全优先取舍;宪法读取/解码/比对失败本就在 detect_violation 内 fail-closed。
            Err(why) => {
                log::error!(
                    target: "constitution-guard",
                    "守卫判定失败,fail-closed 拒块 ({:?}):{why}",
                    params.post_hash(),
                );
                Ok(ImportResult::KnownBad)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LAW_STATUS_PENDING: u8 = 0;

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

    /// 构造 LawVersion(version, 条文, vote_type)的 SCALE 字节 + 哑尾(模拟链端完整编码)。
    /// 单章夹具:全部条文置于第一章(核心章),便于测核心章档位背书。
    fn law_version_scale_vt(version: u32, articles: Vec<MArticle>, vote_type: u8) -> Vec<u8> {
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
        vote_type.encode_to(&mut bytes); // vote_type
        0u64.encode_to(&mut bytes); // proposal_id
        0u64.encode_to(&mut bytes); // published_at
        0u64.encode_to(&mut bytes); // effective_at
        bytes
    }

    /// 默认版本编码:vote_type = 特别案(与创世宪法 `VoteType::Special` 一致)。
    fn law_version_scale(version: u32, articles: Vec<MArticle>) -> Vec<u8> {
        law_version_scale_vt(version, articles, LEG_VOTE_SPECIAL)
    }

    fn law_version_label_scale(title: &str, title_en: Option<&str>) -> Vec<u8> {
        let mut bytes = Vec::new();
        title.as_bytes().to_vec().encode_to(&mut bytes);
        title_en
            .map(|s| s.as_bytes().to_vec())
            .encode_to(&mut bytes);
        bytes
    }

    /// 构造 Law 的 SCALE 字节 + 哑尾。
    /// Law(0) 字节:tier=Constitution、scope=0、给定 houses/显式版本指针/status。
    fn law_scale_with_versions(
        effective_version: Option<u32>,
        latest_version: u32,
        pending_version: Option<u32>,
        status: u8,
        houses: Vec<([u8; 4], [u8; 32])>,
    ) -> Vec<u8> {
        let mut bytes = Vec::new();
        CONSTITUTION_LAW_ID.encode_to(&mut bytes); // law_id
        TIER_CONSTITUTION.encode_to(&mut bytes); // tier = Constitution(0)
        0u32.encode_to(&mut bytes); // scope_code = 0
        houses.encode_to(&mut bytes); // houses
        effective_version.encode_to(&mut bytes); // effective_version
        latest_version.encode_to(&mut bytes); // latest_version
        pending_version.encode_to(&mut bytes); // pending_version
        status.encode_to(&mut bytes); // status
        bytes
    }

    fn law_scale_full(
        latest_version: u32,
        status: u8,
        houses: Vec<([u8; 4], [u8; 32])>,
    ) -> Vec<u8> {
        let (effective_version, pending_version) = if status == LAW_STATUS_PENDING {
            (
                (latest_version > 1).then_some(latest_version - 1),
                Some(latest_version),
            )
        } else {
            (Some(latest_version), None)
        };
        law_scale_with_versions(
            effective_version,
            latest_version,
            pending_version,
            status,
            houses,
        )
    }

    /// 默认合法 Law(0):Effective、空 houses。
    fn law_scale(latest_version: u32) -> Vec<u8> {
        law_scale_full(latest_version, 1 /* Effective */, Vec::new())
    }

    fn laws_by_scope_entry(list: Vec<u64>) -> (Vec<u8>, Vec<u8>) {
        (storage_key::laws_by_scope_constitution(), list.encode())
    }

    /// 一条核心修宪永久公投凭据:`ConstitutionAmendmentProof[version] = (eligible, yes, no)`。
    fn amendment_proof_entry(version: u32, eligible: u64, yes: u64, no: u64) -> (Vec<u8>, Vec<u8>) {
        (
            storage_key::constitution_amendment_proof(version),
            (eligible, yes, no).encode(),
        )
    }

    /// 一条修宪护宪终审凭据:`ConstitutionGuardVoteProof[version] = approve`。
    fn guard_proof_entry(version: u32, approve: u32) -> (Vec<u8>, Vec<u8>) {
        (
            storage_key::constitution_guard_vote_proof(version),
            approve.encode(),
        )
    }

    /// 一份完整合法当前态:Laws[0] + LawVersions[0][version] + LawsByScope[宪法][0]=[0]。
    fn valid_current_state(version: u32, articles: Vec<MArticle>) -> Vec<(Vec<u8>, Vec<u8>)> {
        valid_current_state_vt(version, articles, LEG_VOTE_SPECIAL)
    }

    /// 同 `valid_current_state`,但显式指定生效版本记录的 `vote_type`(测核心章档位背书)。
    /// 修宪版本(v>创世)自动挂一份通过口径(4/7)的护宪终审凭据,使其为一份**合法**修宪态;
    /// 需测护宪凭据缺失/不合格的用例请手工构造(不经本 helper)。
    fn valid_current_state_vt(
        version: u32,
        articles: Vec<MArticle>,
        vote_type: u8,
    ) -> Vec<(Vec<u8>, Vec<u8>)> {
        let mut entries = vec![
            (storage_key::law(CONSTITUTION_LAW_ID), law_scale(version)),
            (
                storage_key::law_version(CONSTITUTION_LAW_ID, version),
                law_version_scale_vt(version, articles, vote_type),
            ),
            laws_by_scope_entry(vec![CONSTITUTION_LAW_ID]),
        ];
        if version > GENESIS_CONSTITUTION_VERSION {
            entries.push(guard_proof_entry(version, 4));
        }
        entries
    }

    /// 一份合法待生效态:v1 仍生效、v2 待生效。
    fn valid_pending_state(pending_articles: Vec<MArticle>) -> Vec<(Vec<u8>, Vec<u8>)> {
        vec![
            (
                storage_key::law(CONSTITUTION_LAW_ID),
                law_scale_with_versions(Some(1), 2, Some(2), LAW_STATUS_PENDING, Vec::new()),
            ),
            (
                storage_key::law_version(CONSTITUTION_LAW_ID, 1),
                law_version_scale(1, genesis_articles()),
            ),
            (
                storage_key::law_version(CONSTITUTION_LAW_ID, 2),
                law_version_scale(2, pending_articles),
            ),
            laws_by_scope_entry(vec![CONSTITUTION_LAW_ID]),
            // 待生效版本 v2 是一次修宪 → 挂通过口径的护宪终审凭据(第21条)。
            guard_proof_entry(2, 4),
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
        assert_ne!(
            storage_key::law_version(0, 1),
            storage_key::law_version_label(0, 1)
        );
        assert!(storage_key::law(0).starts_with(&storage_key::pallet_prefix()));
        assert!(storage_key::law_version(0, 1).starts_with(&storage_key::pallet_prefix()));
        assert!(storage_key::law_version_label(0, 1).starts_with(&storage_key::pallet_prefix()));
    }

    #[test]
    fn full_check_required_for_legislation_or_runtime_code_delta() {
        let mut delta = BTreeMap::new();
        delta.insert(b"OtherPalletKey".to_vec(), Some(vec![1]));
        assert!(!needs_full_invariant_check(&delta));

        let mut legislation_key = storage_key::pallet_prefix().to_vec();
        legislation_key.extend_from_slice(b"SomeStorage");
        delta.insert(legislation_key, Some(vec![2]));
        assert!(needs_full_invariant_check(&delta));

        let mut runtime_delta = BTreeMap::new();
        runtime_delta.insert(sp_storage::well_known_keys::CODE.to_vec(), Some(vec![3]));
        assert!(needs_full_invariant_check(&runtime_delta));
    }

    #[test]
    fn imported_state_precheck_passes_valid_state() {
        let reference = ImmutableReference::from_raw_reader(reader(genesis_state())).unwrap();
        let entries = valid_current_state(2, amended_articles(immutable_intact));
        assert_eq!(
            check_imported_state_key_values(entries.iter(), &reference),
            Ok(())
        );
    }

    #[test]
    fn imported_state_precheck_rejects_mutated_immutable_article() {
        let reference = ImmutableReference::from_raw_reader(reader(genesis_state())).unwrap();
        let articles = amended_articles(|n| {
            if n == 1 {
                "第一条被 warp 态篡改".to_string()
            } else {
                immutable_intact(n)
            }
        });
        let entries = valid_current_state(2, articles);
        assert_eq!(
            check_imported_state_key_values(entries.iter(), &reference),
            Err("ImmutableArticleMutated(1)".to_string())
        );
    }

    #[test]
    fn imported_state_precheck_rejects_missing_constitution_keys() {
        let reference = ImmutableReference::from_raw_reader(reader(genesis_state())).unwrap();
        let entries: Vec<(Vec<u8>, Vec<u8>)> = Vec::new();
        assert_eq!(
            check_imported_state_key_values(entries.iter(), &reference),
            Err("ConstitutionLawMissing".to_string())
        );
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
        // 新版本:不可修改条款原样,只改了可变条 5(核心章),特别案 + 挂通过公投凭据,bump latest_version=2。
        let mut arts = amended_articles(immutable_intact);
        arts[IMMUTABLE_CONSTITUTION_ARTICLES.len()] = article_bytes(5, "可变条款已被合法修改");
        let mut state = valid_current_state(2, arts);
        state.push(amendment_proof_entry(2, 100, 80, 5));
        assert_eq!(check_immutable_articles(reader(state), &reference), Ok(()));
    }

    // ── 第十九条章→档位:核心章条款改动须记录为特别案 ──

    #[test]
    fn reference_derives_core_articles_excluding_immutable() {
        let r = ImmutableReference::from_raw_reader(reader(genesis_state())).unwrap();
        // 创世含核心章非禁改第 5 条 → 应入核心基准;禁改条不得混入核心基准。
        assert!(r.core_articles.contains_key(&5), "核心章基准缺第 5 条");
        for &n in IMMUTABLE_CONSTITUTION_ARTICLES.iter() {
            assert!(
                !r.core_articles.contains_key(&n),
                "核心章基准不应含禁改条 {n}"
            );
        }
    }

    #[test]
    fn rejects_core_clause_change_without_special() {
        let reference = ImmutableReference::from_raw_reader(reader(genesis_state())).unwrap();
        // 改核心章第 5 条但版本记录为重要案(Major=2)→ 违反第十九条,拒块。
        let mut arts = amended_articles(immutable_intact);
        arts[IMMUTABLE_CONSTITUTION_ARTICLES.len()] = article_bytes(5, "核心条被改但走重要案");
        let state = valid_current_state_vt(2, arts, 2 /* Major */);
        assert_eq!(
            check_immutable_articles(reader(state), &reference),
            Err(GuardError::CoreClauseNotSpecial(5))
        );
    }

    #[test]
    fn allows_core_clause_change_with_special() {
        let reference = ImmutableReference::from_raw_reader(reader(genesis_state())).unwrap();
        // 改核心章第 5 条 + 特别案 + 挂通过公投凭据 → 合法。
        let mut arts = amended_articles(immutable_intact);
        arts[IMMUTABLE_CONSTITUTION_ARTICLES.len()] = article_bytes(5, "核心条经特别案修改");
        let mut state = valid_current_state_vt(2, arts, LEG_VOTE_SPECIAL);
        state.push(amendment_proof_entry(2, 100, 80, 5));
        assert_eq!(check_immutable_articles(reader(state), &reference), Ok(()));
    }

    #[test]
    fn rejects_core_clause_change_without_referendum_proof() {
        let reference = ImmutableReference::from_raw_reader(reader(genesis_state())).unwrap();
        // 核心章第 5 条改动 + 特别案,但缺永久公投凭据 → 拒。
        let mut arts = amended_articles(immutable_intact);
        arts[IMMUTABLE_CONSTITUTION_ARTICLES.len()] = article_bytes(5, "核心条改但无公投凭据");
        let state = valid_current_state_vt(2, arts, LEG_VOTE_SPECIAL); // 不挂 proof
        assert_eq!(
            check_immutable_articles(reader(state), &reference),
            Err(GuardError::CoreClauseReferendumMissing(2))
        );
    }

    #[test]
    fn rejects_core_clause_change_with_failing_referendum() {
        let reference = ImmutableReference::from_raw_reader(reader(genesis_state())).unwrap();
        // 核心章第 5 条改动 + 特别案 + 公投未过口径(参与 45% <70%)→ 拒。
        let mut arts = amended_articles(immutable_intact);
        arts[IMMUTABLE_CONSTITUTION_ARTICLES.len()] = article_bytes(5, "核心条改但公投未过");
        let mut state = valid_current_state_vt(2, arts, LEG_VOTE_SPECIAL);
        state.push(amendment_proof_entry(2, 100, 40, 5));
        assert_eq!(
            check_immutable_articles(reader(state), &reference),
            Err(GuardError::CoreClauseReferendumNotPassed(2))
        );
    }

    #[test]
    fn allows_unchanged_core_clause_with_non_special() {
        let reference = ImmutableReference::from_raw_reader(reader(genesis_state())).unwrap();
        // 核心章第 5 条原样不动,版本记录为重要案 → 核心章未变,不约束档位 → 合法。
        let state =
            valid_current_state_vt(2, amended_articles(immutable_intact), 2 /* Major */);
        assert_eq!(check_immutable_articles(reader(state), &reference), Ok(()));
    }

    // ── 第21条:一切修宪须挂通过口径(4/7)的护宪大法官终审凭据 ──

    #[test]
    fn rejects_amendment_without_guard_proof() {
        let reference = ImmutableReference::from_raw_reader(reader(genesis_state())).unwrap();
        // v2 修宪(核心条款不变、免公投),但无护宪终审凭据 → 拒。
        let state = vec![
            (storage_key::law(CONSTITUTION_LAW_ID), law_scale(2)),
            (
                storage_key::law_version(CONSTITUTION_LAW_ID, 2),
                law_version_scale_vt(2, amended_articles(immutable_intact), 2 /* Major */),
            ),
            laws_by_scope_entry(vec![CONSTITUTION_LAW_ID]),
        ];
        assert_eq!(
            check_immutable_articles(reader(state), &reference),
            Err(GuardError::GuardReviewMissing(2))
        );
    }

    #[test]
    fn rejects_amendment_with_failing_guard_review() {
        let reference = ImmutableReference::from_raw_reader(reader(genesis_state())).unwrap();
        // v2 修宪 + 护宪终审仅 3/7 赞成(<4)→ 拒。
        let state = vec![
            (storage_key::law(CONSTITUTION_LAW_ID), law_scale(2)),
            (
                storage_key::law_version(CONSTITUTION_LAW_ID, 2),
                law_version_scale_vt(2, amended_articles(immutable_intact), 2 /* Major */),
            ),
            laws_by_scope_entry(vec![CONSTITUTION_LAW_ID]),
            guard_proof_entry(2, 3),
        ];
        assert_eq!(
            check_immutable_articles(reader(state), &reference),
            Err(GuardError::GuardReviewNotPassed(2))
        );
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
        Some(2u32).encode_to(&mut law);
        2u32.encode_to(&mut law);
        Option::<u32>::None.encode_to(&mut law);
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
        let reference = ImmutableReference::from_raw_reader(reader(genesis_state())).unwrap();
        let state = valid_pending_state(amended_articles(immutable_intact));
        assert_eq!(check_immutable_articles(reader(state), &reference), Ok(()));
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
    fn effective_version_uses_explicit_pointer() {
        // Effective 和 Pending 都只读显式 effective_version;新法尚无生效版时返回错误。
        assert_eq!(
            effective_version_of_law(&law_scale_full(3, 1, vec![])).unwrap(),
            3
        );
        assert_eq!(
            effective_version_of_law(&law_scale_full(3, 0, vec![])).unwrap(),
            2
        );
        assert!(
            effective_version_of_law(&law_scale_full(1, 0, vec![])).is_err(),
            "新法待生效且尚无 effective_version 时不能再推断"
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
        let html = render_constitution_html(&scale, &[1], None).expect("应能重建");
        assert!(html.starts_with("<!DOCTYPE html>"));
        assert!(html.trim_end().ends_with("</html>"));
        assert!(html.contains(
            "<span class=\"doc-version-cn\">v1</span><span class=\"doc-version-en\">v1</span>"
        ));
        assert!(html.contains("href=\"#article-1\""));
        assert!(html.contains("id=\"article-1\" class=\"block article-block\""));
        assert!(html.contains(
            "<span class=\"cn heading-cn\">第1条<span class=\"immutable-badge immutable-badge-cn\">不可修改条款</span></span>"
        ));
        assert!(html.contains(
            "<span class=\"en heading-en\">Article 1<span class=\"immutable-badge immutable-badge-en\">Immutable Clause</span></span>"
        ));
    }

    #[test]
    fn render_uses_chain_law_version_label() {
        let scale = law_version_scale(1, vec![article_bytes(1, "正文")]);
        let label = law_version_label_scale("创世版", Some("Genesis Edition"));
        let html = render_constitution_html(&scale, &[], Some(&label)).expect("应能重建");
        assert!(html.contains(
            "<span class=\"doc-version-cn\">创世版</span><span class=\"doc-version-en\">Genesis Edition</span>"
        ));
    }
}
