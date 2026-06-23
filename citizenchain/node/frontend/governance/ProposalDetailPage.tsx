// 提案详情页：提案元数据 + 业务详情模块挂载 + 投票进度 + 管理员投票状态列表。
// 从提案列表和机构页面进入时行为一致：自动检测管理员钱包权限。
import { useEffect, useState, useCallback, useRef } from 'react';
import { sanitizeError } from '../core/tauri';
import { hexToSs58 } from '../shared/ss58';
import { DuoqianTransferProposalDetailSection } from '../transaction/duoqian-transfer/ProposalDetailSection';
import { adminsChangeApi } from './admins_change/api';
import { governanceApi as api } from './api';
import type { ProposalFullInfo, AdminWalletMatch, UserVoteStatus, InstitutionDetail } from './types';
import { VoteSigningFlow } from './VoteSigningFlow';

type Props = {
  proposalId: number;
  adminWallets: AdminWalletMatch[];
  cidNumber?: string;
  onBack: () => void;
};

function institutionHexToCidNumber(hex: string): string {
  const clean = hex.startsWith('0x') ? hex.slice(2) : hex;
  const bytes: number[] = [];
  for (let i = 0; i < clean.length; i += 2) {
    bytes.push(parseInt(clean.substring(i, i + 2), 16));
  }
  let end = bytes.length;
  while (end > 0 && bytes[end - 1] === 0) end--;
  return new TextDecoder().decode(new Uint8Array(bytes.slice(0, end)));
}

export function ProposalDetailPage({ proposalId, adminWallets: externalAdminWallets, cidNumber: externalCidNumber, onBack }: Props) {
  const [info, setInfo] = useState<ProposalFullInfo | null>(null);
  const [institution, setInstitution] = useState<InstitutionDetail | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [votingWallet, setVotingWallet] = useState<AdminWalletMatch | null>(null);
  const [voteStatuses, setVoteStatuses] = useState<Record<string, UserVoteStatus>>({});
  const [detectedAdminWallets, setDetectedAdminWallets] = useState<AdminWalletMatch[]>([]);
  const [resolvedCidNumber, setResolvedCidNumber] = useState<string | undefined>(externalCidNumber);
  // 投票中（已提交但未确认上链）的钱包 pubkey → 提交时间
  const [pendingVotes, setPendingVotes] = useState<Map<string, number>>(new Map());
  // 双层 ID v1:展示号反查值,链上 ProposalDisplayId[id] 拉取
  const [displayMeta, setDisplayMeta] = useState<import('./types').ProposalDisplayMeta | null>(
    null,
  );
  const pollTimerRef = useRef<ReturnType<typeof setInterval> | null>(null);
  // 超过 5 分钟未确认的投票视为丢失
  const PENDING_TIMEOUT_MS = 5 * 60 * 1000;

  const adminWallets = externalAdminWallets.length > 0 ? externalAdminWallets : detectedAdminWallets;
  const cidNumber = resolvedCidNumber;

  const fetchVoteStatuses = useCallback(async (
    pid: number, admins: string[], sid: string | undefined,
  ) => {
    const results: Record<string, UserVoteStatus> = {};
    await Promise.all(
      admins.map(async (pubkey) => {
        try {
          const vs = await api.checkVoteStatus(pid, pubkey, sid);
          results[pubkey.toLowerCase()] = vs;
        } catch (_) {}
      }),
    );
    setVoteStatuses(results);
    return results;
  }, []);

  const refreshData = useCallback(async (inst?: InstitutionDetail | null, sid?: string) => {
    const curInst = inst ?? institution;
    const curSid = sid ?? cidNumber;
    try {
      const d = await api.getProposalDetail(proposalId);
      setInfo(d);
    } catch (_) {}
    if (curInst && curSid) {
      const statuses = await fetchVoteStatuses(proposalId, curInst.admins.map(a => a.pubkeyHex), curSid);
      // 已确认上链的投票或超时的投票，从 pending 中移除
      setPendingVotes((prev) => {
        const next = new Map(prev);
        const now = Date.now();
        for (const [pk, submittedAt] of prev) {
          const vs = statuses[pk];
          const voted = vs && (vs.internalVote != null || vs.jointVote != null);
          const timedOut = now - submittedAt > PENDING_TIMEOUT_MS;
          if (voted || timedOut) next.delete(pk);
        }
        return next;
      });
    }
  }, [proposalId, institution, cidNumber, fetchVoteStatuses]);

  useEffect(() => {
    setLoading(true);
    const fetchAll = async () => {
      const [d, dm] = await Promise.all([
        api.getProposalDetail(proposalId),
        api.getProposalDisplay(proposalId).catch(() => null),
      ]);
      setInfo(d);
      setDisplayMeta(dm ?? null);
      let sid = externalCidNumber;
      if (!sid && d.meta.institutionHex) {
        sid = institutionHexToCidNumber(d.meta.institutionHex);
        setResolvedCidNumber(sid);
      }
      let wallets = externalAdminWallets;
      if (sid) {
        try {
          const inst = await api.getInstitutionDetail(sid);
          setInstitution(inst);
          if (externalAdminWallets.length === 0) {
            const activated = await adminsChangeApi.getActivatedAdmins(sid, {
              cidNumber: sid,
            });
            wallets = activated.map(a => ({ address: hexToSs58(a.pubkeyHex), pubkeyHex: a.pubkeyHex, name: '' }));
            setDetectedAdminWallets(wallets);
          }
          await fetchVoteStatuses(proposalId, inst.admins.map(a => a.pubkeyHex), sid);
        } catch (_) {}
      }
    };
    fetchAll()
      .catch((e) => setError(sanitizeError(e)))
      .finally(() => setLoading(false));
  }, [proposalId]);

  // 有 pending 投票时轮询刷新
  useEffect(() => {
    if (pendingVotes.size > 0) {
      pollTimerRef.current = setInterval(() => refreshData(), 5000);
    }
    return () => {
      if (pollTimerRef.current) { clearInterval(pollTimerRef.current); pollTimerRef.current = null; }
    };
  }, [pendingVotes.size, refreshData]);

  // 投票提交成功回调：标记为 pending，关闭弹窗
  const handleVoteSuccess = useCallback((txHash: string) => {
    if (votingWallet) {
      setPendingVotes((prev) => new Map(prev).set(votingWallet.pubkeyHex.toLowerCase(), Date.now()));
    }
    setVotingWallet(null);
    // 立即刷新一次
    refreshData();
  }, [votingWallet, refreshData]);

  if (loading) {
    return <div className="governance-section"><p>加载提案详情…</p></div>;
  }
  if (error) {
    return (
      <div className="governance-section">
        <button className="back-button" onClick={onBack}>← 返回</button>
        <div className="error">{error}</div>
      </div>
    );
  }
  if (!info) return null;

  const { meta } = info;
  const displayId = formatProposalId(meta.proposalId, displayMeta);
  const displayStatus = { code: meta.status, label: statusLabel(meta.status) };

  return (
    <div className="governance-section">
      <button className="back-button" onClick={onBack}>← 返回</button>
      <h2>提案 {displayId}</h2>
      {info.cidFullName && (
        <p className="proposal-institution-name">{info.cidFullName}</p>
      )}

      {votingWallet && (
        <VoteSigningFlow
          proposalId={meta.proposalId}
          proposalKind={meta.kind}
          adminWallets={[votingWallet]}
          cidNumber={cidNumber}
          onClose={() => setVotingWallet(null)}
          onSuccess={handleVoteSuccess}
        />
      )}

      {/* 元数据卡片 */}
      <div className="institution-detail-grid">
        <div className="metric-card">
          <div className="metric-label">提案类型</div>
          <div className="metric-value">{kindLabel(meta.kind)}</div>
        </div>
        {meta.kind === 1 && (
          <div className="metric-card">
            <div className="metric-label">当前阶段</div>
            <div className="metric-value">{stageLabel(meta.stage)}</div>
          </div>
        )}
        <div className="metric-card">
          <div className="metric-label">状态</div>
          <div className={`metric-value status-text-${displayStatus.code}`}>
            {displayStatus.label}
          </div>
        </div>
        {meta.internalOrg != null && (
          <div className="metric-card">
            <div className="metric-label">机构类型</div>
            <div className="metric-value">{orgTypeLabel(meta.internalOrg)}</div>
          </div>
        )}
      </div>

      <DuoqianTransferProposalDetailSection info={info} />

      {/* 协议升级提案详情 */}
      {info.runtimeUpgradeDetail && (
        <div className="institution-info-section">
          <h3>协议升级详情</h3>
          <div className="proposal-detail-table">
            <div className="detail-row">
              <span className="detail-label">原因</span>
              <span className="detail-value">{info.runtimeUpgradeDetail.reason}</span>
            </div>
            <div className="detail-row">
              <span className="detail-label">代码哈希</span>
              <code className="detail-value">0x{info.runtimeUpgradeDetail.codeHashHex}</code>
            </div>
            <div className="detail-row">
              <span className="detail-label">提案人</span>
              <code className="detail-value">{hexToSs58(info.runtimeUpgradeDetail.proposerHex)}</code>
            </div>
          </div>
        </div>
      )}

      {/* 投票进度 */}
      <div className="institution-info-section">
        <h3>投票进度</h3>
        {info.internalTally && (
          <VoteTallyBar title="内部投票" yes={info.internalTally.yes} no={info.internalTally.no} />
        )}
        {info.jointTally && (
          <VoteTallyBar title="联合投票" yes={info.jointTally.yes} no={info.jointTally.no} threshold={105} />
        )}
        {info.citizenTally && (
          <VoteTallyBar title="公民投票" yes={info.citizenTally.yes} no={info.citizenTally.no} />
        )}
        {!info.internalTally && !info.jointTally && !info.citizenTally && (
          <p className="no-data">暂无投票数据</p>
        )}
      </div>

      {/* 管理员投票状态列表 */}
      {institution && (
        <div className="institution-info-section">
          <h3>管理员投票状态（{institution.admins.length} 人）</h3>
          <div className="admin-grid">
            {institution.admins.map((admin, i) => {
              const pubkey = admin.pubkeyHex;
              const pk = pubkey.toLowerCase();
              const myWallet = adminWallets.find(w => w.pubkeyHex.toLowerCase() === pk);
              const vs = voteStatuses[pk];
              const voted = meta.kind === 1 ? vs?.jointVote : vs?.internalVote;
              const hasVoted = voted != null;
              const isPending = pendingVotes.has(pk);
              const canVote = myWallet && meta.status === 0 && !hasVoted && !isPending;

              return (
                <div key={pubkey} className={`metric-card admin-card ${hasVoted ? (voted ? 'admin-card-voted-yes' : 'admin-card-voted-no') : ''}`}>
                  <span className="admin-card-index">{i + 1}</span>
                  <code className="admin-card-address">{hexToSs58(pubkey)}</code>
                  <div className="admin-card-actions">
                    {canVote ? (
                      <button className="vote-button-inline" onClick={() => setVotingWallet(myWallet)}>
                        投票
                      </button>
                    ) : isPending && !hasVoted ? (
                      <span className="vote-result-tag vote-pending-tag">投票中</span>
                    ) : hasVoted ? (
                      <span className={`vote-result-tag ${voted ? 'vote-yes-tag' : 'vote-no-tag'}`}>
                        {voted ? '赞成' : '反对'}
                      </span>
                    ) : (
                      <span className="vote-result-tag vote-none-tag">未投票</span>
                    )}
                  </div>
                </div>
              );
            })}
          </div>
        </div>
      )}
    </div>
  );
}

function VoteTallyBar({ title, yes, no, threshold }: {
  title: string; yes: number; no: number; threshold?: number;
}) {
  const total = yes + no;
  const yesPercent = total > 0 ? Math.round((yes / total) * 100) : 0;
  return (
    <div className="vote-tally">
      <div className="vote-tally-header">
        <span className="vote-tally-title">{title}</span>
        <span className="vote-tally-counts">
          赞成 {yes} / 反对 {no}
          {threshold != null && <span className="vote-threshold">（通过线 {threshold}）</span>}
        </span>
      </div>
      <div className="vote-tally-bar">
        <div className="vote-tally-bar-yes" style={{ width: total > 0 ? `${yesPercent}%` : '0%' }} />
      </div>
    </div>
  );
}

/// 提案展示号格式化(双层 ID v1):`2026000123` 风格(年份 + 6 位补零序号)。
/// 主键 `proposalId` 与展示号解耦,展示号由 `getProposalDisplay` 反查得到。
/// `meta=null` 时(链上未写入)fallback 到 `#<id>` 形式。
export function formatProposalId(
  id: number,
  meta?: import('./types').ProposalDisplayMeta | null,
): string {
  if (meta == null) return `#${id}`;
  const seq = String(meta.seqInYear).padStart(6, '0');
  return `${meta.year}${seq}`;
}
function kindLabel(kind: number): string {
  return kind === 0 ? '内部投票' : kind === 1 ? '联合投票' : '未知';
}
function stageLabel(stage: number): string {
  switch (stage) { case 0: return '内部阶段'; case 1: return '联合阶段'; case 2: return '公民阶段'; default: return '未知'; }
}
function statusLabel(status: number): string {
  switch (status) { case 0: return '投票中'; case 1: return '已通过'; case 2: return '已否决'; case 3: return '已执行'; case 4: return '执行失败'; default: return '未知'; }
}
function orgTypeLabel(orgType: number): string {
  switch (orgType) { case 0: return '国储会'; case 1: return '省储会'; case 2: return '省储行'; default: return '未知'; }
}
