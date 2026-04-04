// 提案详情页：提案元数据 + 业务详情（转账/升级）+ 投票进度 + 管理员投票状态列表。
// 从提案列表和机构页面进入时行为一致：自动检测管理员钱包权限。
import { useEffect, useState, useCallback, useRef } from 'react';
import { api, sanitizeError } from '../api';
import { formatBalance, hexToSs58 } from '../format';
import type { ProposalFullInfo, AdminWalletMatch, UserVoteStatus, InstitutionDetail } from './governance-types';
import { VoteSigningFlow } from './VoteSigningFlow';

type Props = {
  proposalId: number;
  adminWallets: AdminWalletMatch[];
  shenfenId?: string;
  onBack: () => void;
};

function institutionHexToShenfenId(hex: string): string {
  const clean = hex.startsWith('0x') ? hex.slice(2) : hex;
  const bytes: number[] = [];
  for (let i = 0; i < clean.length; i += 2) {
    bytes.push(parseInt(clean.substring(i, i + 2), 16));
  }
  let end = bytes.length;
  while (end > 0 && bytes[end - 1] === 0) end--;
  return new TextDecoder().decode(new Uint8Array(bytes.slice(0, end)));
}

export function ProposalDetailPage({ proposalId, adminWallets: externalAdminWallets, shenfenId: externalShenfenId, onBack }: Props) {
  const [info, setInfo] = useState<ProposalFullInfo | null>(null);
  const [institution, setInstitution] = useState<InstitutionDetail | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [votingWallet, setVotingWallet] = useState<AdminWalletMatch | null>(null);
  const [voteStatuses, setVoteStatuses] = useState<Record<string, UserVoteStatus>>({});
  const [detectedAdminWallets, setDetectedAdminWallets] = useState<AdminWalletMatch[]>([]);
  const [resolvedShenfenId, setResolvedShenfenId] = useState<string | undefined>(externalShenfenId);
  // 投票中（已提交但未确认上链）的钱包 pubkey → 提交时间
  const [pendingVotes, setPendingVotes] = useState<Map<string, number>>(new Map());
  const pollTimerRef = useRef<ReturnType<typeof setInterval> | null>(null);
  // 超过 5 分钟未确认的投票视为丢失
  const PENDING_TIMEOUT_MS = 5 * 60 * 1000;

  const adminWallets = externalAdminWallets.length > 0 ? externalAdminWallets : detectedAdminWallets;
  const shenfenId = resolvedShenfenId;

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
    const curSid = sid ?? shenfenId;
    try {
      const d = await api.getProposalDetail(proposalId);
      setInfo(d);
    } catch (_) {}
    if (curInst && curSid) {
      const statuses = await fetchVoteStatuses(proposalId, curInst.admins, curSid);
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
  }, [proposalId, institution, shenfenId, fetchVoteStatuses]);

  useEffect(() => {
    setLoading(true);
    const fetchAll = async () => {
      const d = await api.getProposalDetail(proposalId);
      setInfo(d);
      let sid = externalShenfenId;
      if (!sid && d.meta.institutionHex) {
        sid = institutionHexToShenfenId(d.meta.institutionHex);
        setResolvedShenfenId(sid);
      }
      let wallets = externalAdminWallets;
      if (externalAdminWallets.length === 0 && sid) {
        try {
          const activated = await api.getActivatedAdmins(sid);
          wallets = activated.map(a => ({ address: hexToSs58(a.pubkeyHex), pubkeyHex: a.pubkeyHex, name: '' }));
          setDetectedAdminWallets(wallets);
        } catch (_) {}
      }
      if (sid) {
        try {
          const inst = await api.getInstitutionDetail(sid);
          setInstitution(inst);
          await fetchVoteStatuses(proposalId, inst.admins, sid);
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
  const displayId = formatProposalId(meta.proposalId);

  return (
    <div className="governance-section">
      <button className="back-button" onClick={onBack}>← 返回</button>
      <h2>提案 {displayId}</h2>
      {info.institutionName && (
        <p className="proposal-institution-name">{info.institutionName}</p>
      )}

      {votingWallet && (
        <VoteSigningFlow
          proposalId={meta.proposalId}
          proposalKind={meta.kind}
          adminWallets={[votingWallet]}
          shenfenId={shenfenId}
          useRateVote={!!info.feeRateDetail}
          useSafetyFundVote={!!info.safetyFundDetail}
          useSweepVote={!!info.sweepDetail}
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
          <div className={`metric-value status-text-${meta.status}`}>
            {statusLabel(meta.status)}
          </div>
        </div>
        {meta.internalOrg != null && (
          <div className="metric-card">
            <div className="metric-label">机构类型</div>
            <div className="metric-value">{orgTypeLabel(meta.internalOrg)}</div>
          </div>
        )}
      </div>

      {/* 转账提案详情 */}
      {info.transferDetail && (
        <div className="institution-info-section">
          <h3>转账详情</h3>
          <div className="proposal-detail-table">
            <div className="detail-row">
              <span className="detail-label">金额</span>
              <span className="detail-value">
                {formatBalance(info.transferDetail.amountFen)}
              </span>
            </div>
            <div className="detail-row">
              <span className="detail-label">收款人</span>
              <code className="detail-value">{hexToSs58(info.transferDetail.beneficiaryHex)}</code>
            </div>
            <div className="detail-row">
              <span className="detail-label">备注</span>
              <span className="detail-value">{info.transferDetail.remark || '—'}</span>
            </div>
            <div className="detail-row">
              <span className="detail-label">提案人</span>
              <code className="detail-value">{hexToSs58(info.transferDetail.proposerHex)}</code>
            </div>
          </div>
        </div>
      )}

      {/* Runtime 升级提案详情 */}
      {info.runtimeUpgradeDetail && (
        <div className="institution-info-section">
          <h3>运行时升级详情</h3>
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
              <span className="detail-label">代码已上传</span>
              <span className="detail-value">
                {info.runtimeUpgradeDetail.hasCode ? '是' : '否'}
              </span>
            </div>
            <div className="detail-row">
              <span className="detail-label">提案人</span>
              <code className="detail-value">{hexToSs58(info.runtimeUpgradeDetail.proposerHex)}</code>
            </div>
          </div>
        </div>
      )}

      {/* 费率设置提案详情 */}
      {info.feeRateDetail && (
        <div className="institution-info-section">
          <h3>费率设置详情</h3>
          <div className="proposal-detail-table">
            <div className="detail-row">
              <span className="detail-label">新费率</span>
              <span className="detail-value">{info.feeRateDetail.newRateBp} bp ({(info.feeRateDetail.newRateBp / 100).toFixed(2)}%)</span>
            </div>
          </div>
        </div>
      )}

      {/* 安全基金转账提案详情 */}
      {info.safetyFundDetail && (
        <div className="institution-info-section">
          <h3>安全基金转账详情</h3>
          <div className="proposal-detail-table">
            <div className="detail-row">
              <span className="detail-label">收款地址</span>
              <code className="detail-value">{hexToSs58(info.safetyFundDetail.beneficiaryHex)}</code>
            </div>
            <div className="detail-row">
              <span className="detail-label">金额</span>
              <span className="detail-value">{formatBalance(parseInt(info.safetyFundDetail.amountFen))}</span>
            </div>
            {info.safetyFundDetail.remark && (
              <div className="detail-row">
                <span className="detail-label">备注</span>
                <span className="detail-value">{info.safetyFundDetail.remark}</span>
              </div>
            )}
          </div>
        </div>
      )}

      {/* 手续费划转提案详情 */}
      {info.sweepDetail && (
        <div className="institution-info-section">
          <h3>手续费划转详情</h3>
          <div className="proposal-detail-table">
            <div className="detail-row">
              <span className="detail-label">金额</span>
              <span className="detail-value">{formatBalance(parseInt(info.sweepDetail.amountFen))}</span>
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
            {institution.admins.map((pubkey, i) => {
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

function formatProposalId(id: number): string {
  const year = Math.floor(id / 1_000_000);
  const counter = id % 1_000_000;
  return `${year}#${counter}`;
}
function kindLabel(kind: number): string {
  return kind === 0 ? '内部投票' : kind === 1 ? '联合投票' : '未知';
}
function stageLabel(stage: number): string {
  switch (stage) { case 0: return '内部阶段'; case 1: return '联合阶段'; case 2: return '公民阶段'; default: return '未知'; }
}
function statusLabel(status: number): string {
  switch (status) { case 0: return '投票中'; case 1: return '已通过'; case 2: return '已否决'; case 3: return '已执行'; default: return '未知'; }
}
function orgTypeLabel(orgType: number): string {
  switch (orgType) { case 0: return '国储会'; case 1: return '省储会'; case 2: return '省储行'; default: return '未知'; }
}
