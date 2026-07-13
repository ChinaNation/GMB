// 立法与表决前端 API(对接 onchina /api/v1/legislation/*)。
// 发起/表决返回扫码上链 sign_request(字符串),由冷签弹窗渲染成 QR 交 CitizenApp/CitizenWallet 提交;
// 读法律/提案进度直读链投影。通用 http 走 utils/http.ts,本模块不另造请求封装。

import type { AdminAuth } from '../auth/types';
import { adminRequest } from '../utils/http';
import type {
  LawView,
  LegProposalState,
  ProposableCandidate,
  ProposeLawInput,
} from './types';

/** GET 本机构可发起的提案候选(category×tier×voteTypes)。 */
export async function getProposable(auth: AdminAuth): Promise<ProposableCandidate[]> {
  return adminRequest<ProposableCandidate[]>('/api/v1/legislation/proposable', auth);
}

/** GET 本级已生效/在册法律列表(按层级 + 行政区码)。 */
export async function listLaws(
  auth: AdminAuth,
  tier: number,
  scopeCode: number,
): Promise<LawView[]> {
  return adminRequest<LawView[]>(
    `/api/v1/legislation/laws?tier=${tier}&scope_code=${scopeCode}`,
    auth,
  );
}

/** GET 本节点绑定机构层级/辖区的法律(会话派生 scope,前端不传码)。 */
export async function listMyLaws(auth: AdminAuth): Promise<LawView[]> {
  return adminRequest<LawView[]>('/api/v1/legislation/laws/mine', auth);
}

/** GET 单部法律办理端展示版本全文。 */
export async function getLaw(auth: AdminAuth, lawId: number): Promise<LawView> {
  return adminRequest<LawView>(`/api/v1/legislation/laws/${lawId}`, auth);
}

/** GET 提案进度只读投影。 */
export async function getProposalState(
  auth: AdminAuth,
  proposalId: number,
): Promise<LegProposalState> {
  return adminRequest<LegProposalState>(`/api/v1/legislation/proposals/${proposalId}`, auth);
}

/** POST 发起法律案,返回扫码上链 sign_request。 */
export async function proposeLegislation(
  auth: AdminAuth,
  input: ProposeLawInput,
): Promise<string> {
  return adminRequest<string>('/api/v1/legislation/propose', auth, {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify(input),
  });
}

/** POST 当前代表机构表决，返回扫码上链 sign_request。 */
export async function castRepresentativeVote(
  auth: AdminAuth,
  proposalId: number,
  approve: boolean,
): Promise<string> {
  return adminRequest<string>('/api/v1/legislation/representative-vote', auth, {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify({ proposalId, approve }),
  });
}
