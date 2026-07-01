// 中文注释:立法与表决前端类型,camelCase 逐字镜像后端 DTO
// (onchina/src/domains/legislation/law/model.rs 与 chain_read_proposal.rs);枚举数值与链端对齐。

/** 提案类型(可扩展维度;本轮仅 law 实现,personnel/budget 预留)。 */
export type ProposalCategory = 'law' | 'personnel' | 'budget';

/** 立法动作(对应 propose_enact/amend/repeal_law)。 */
export type LawActionInput = 'enact' | 'amend' | 'repeal';

/** 法律层级(对齐链 Tier::as_u8)。 */
export const LAW_TIER = {
  CONSTITUTION: 0,
  NATIONAL: 1,
  PROVINCIAL: 2,
  MUNICIPAL: 3,
} as const;

/** 表决类型(对齐链 VoteType::as_u8)。 */
export const VOTE_TYPE = {
  REGULAR: 0,
  REGULAR_EDUCATION: 1,
  MAJOR: 2,
  MAJOR_EDUCATION: 3,
  SPECIAL: 4,
} as const;

/** 法律状态(对齐链 LawStatus)。 */
export const LAW_STATUS = {
  PENDING: 0,
  EFFECTIVE: 1,
  REPEALED: 2,
} as const;

/** 提案阶段(对齐链 STAGE_LEG_*)。 */
export const LEG_STAGE = {
  HOUSE: 10,
  REFERENDUM: 11,
  SIGN: 12,
  OVERRIDE: 13,
  GUARD: 14,
} as const;

/** 款(最末层正文)。 */
export interface LawClause {
  number: number;
  text: string;
  textEn?: string | null;
}

/** 条(目录 + 正文 + 款列表)。 */
export interface LawArticle {
  number: number;
  title: string;
  titleEn?: string | null;
  body: string;
  bodyEn?: string | null;
  clauses: LawClause[];
}

/** 节(目录 + 条列表)。 */
export interface LawSection {
  number: number;
  title: string;
  titleEn?: string | null;
  articles: LawArticle[];
}

/** 章(目录 + 节列表)。 */
export interface LawChapter {
  number: number;
  title: string;
  titleEn?: string | null;
  sections: LawSection[];
}

/** 机构 + 账户引用(code=去尾 \0 的机构码,accountHex=0x 小写)。 */
export interface HouseRef {
  code: string;
  accountHex: string;
}

/** 法律只读视图(Law 主体 + 当前版本全文)。 */
export interface LawView {
  lawId: number;
  version: number;
  tier: number;
  scopeCode: number;
  status: number;
  voteType: number;
  title: string;
  titleEn?: string | null;
  contentHash: string;
  proposalId: number;
  publishedAt: number;
  effectiveAt: number;
  houses: HouseRef[];
  chapters: LawChapter[];
}

/** 发起法律案请求体(houses/executive/legislature 由后端按宪法路由解析,前端不传)。 */
export interface ProposeLawInput {
  lawAction: LawActionInput;
  tier: number;
  scopeCode: number;
  voteType: number;
  title: string;
  titleEn?: string | null;
  /** 正文:章>节>条>款(立法/修法携带;废法为空)。 */
  chapters: LawChapter[];
  effectiveAt: number;
  /** 修法/废法目标法律 ID;立法为 null。 */
  lawId?: number | null;
}

/** 计票(院内 u32 / 公投 u64,前端统一 number)。 */
export interface VoteTally {
  yes: number;
  no: number;
}

/** 提案进度只读投影(不含计票判定,只搬运链上事实)。 */
export interface LegProposalState {
  proposalId: number;
  kind: number;
  stage: number;
  status: number;
  voteType: number;
  currentHouse: number;
  referendumRequired: boolean;
  needsGuard: boolean;
  houses: HouseRef[];
  startBlock: number;
  endBlock: number;
  houseTally: VoteTally;
  referendumTally: VoteTally;
}
