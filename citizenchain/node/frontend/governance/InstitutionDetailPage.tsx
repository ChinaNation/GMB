// 机构详情页：显示机构基本信息、余额、管理员列表、活跃提案列表。
// 如果用户导入的冷钱包匹配该机构管理员，显示管理员标记和投票入口。
import { useEffect, useState } from 'react';
import { api, sanitizeError } from '../api';
import { formatBalance, hexToSs58 } from '../format';
import type { InstitutionDetail, ProposalListItem, AdminWalletMatch } from './governance-types';

type Props = {
  shenfenId: string;
  onBack: () => void;
  onSelectProposal?: (proposalId: number, adminWallets: AdminWalletMatch[], shenfenId: string) => void;
  onCreateProposal?: (shenfenId: string, orgType: number, institutionName: string, duoqianAddress: string, adminWallets: AdminWalletMatch[]) => void;
  onCreateRuntimeUpgrade?: (adminWallets: AdminWalletMatch[]) => void;
  /** 隐藏返回按钮（用于直接作为 Tab 内容显示时）。 */
  hideBackButton?: boolean;
};

export function InstitutionDetailPage({ shenfenId, onBack, onSelectProposal, onCreateProposal, onCreateRuntimeUpgrade, hideBackButton }: Props) {
  const [detail, setDetail] = useState<InstitutionDetail | null>(null);
  const [proposals, setProposals] = useState<ProposalListItem[]>([]);
  const [adminWallets, setAdminWallets] = useState<AdminWalletMatch[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  const isAdmin = adminWallets.length > 0;

  useEffect(() => {
    setLoading(true);
    Promise.all([
      api.getInstitutionDetail(shenfenId),
      api.getInstitutionProposals(shenfenId).catch(() => [] as ProposalListItem[]),
      api.checkAdminWallets(shenfenId).catch(() => [] as AdminWalletMatch[]),
    ])
      .then(([d, p, aw]) => {
        setDetail(d);
        setProposals(p);
        setAdminWallets(aw);
        setError(null);
      })
      .catch((e) => setError(sanitizeError(e)))
      .finally(() => setLoading(false));
  }, [shenfenId]);

  if (loading) {
    return <div className="governance-section"><p>加载中…</p></div>;
  }

  if (error) {
    return (
      <div className="governance-section">
        <button className="back-button" onClick={onBack}>← 返回</button>
        <div className="error">{error}</div>
      </div>
    );
  }

  if (!detail) return null;

  return (
    <div className="governance-section">
      {!hideBackButton && (
        <button className="back-button" onClick={onBack}>← 返回机构列表</button>
      )}

      <div className="institution-title-row">
        <h2>{detail.name}</h2>
        {isAdmin && (
          <span className="admin-badge">管理员</span>
        )}
      </div>

      {detail.warning && <div className="warning">{detail.warning}</div>}

      <div className="institution-detail-grid">
        <div className="metric-card">
          <div className="metric-label">机构类型 <code className="metric-label-id">{detail.shenfenId}</code></div>
          <div className="metric-value">{detail.orgTypeLabel}</div>
        </div>
        <div className="metric-card">
          <div className="metric-label">多签余额 <code className="metric-label-id">{hexToSs58(detail.duoqianAddress)}</code></div>
          <div className="metric-value">
            {detail.balanceFen != null
              ? formatBalance(detail.balanceFen)
              : '—'}
          </div>
        </div>
        <div className="metric-card">
          <div className="metric-label">内部投票阈值</div>
          <div className="metric-value">{detail.internalThreshold} 票</div>
        </div>
        <div className="metric-card">
          <div className="metric-label">联合投票权重</div>
          <div className="metric-value">{detail.jointVoteWeight}</div>
        </div>
      </div>

      {/* 提案类型入口（管理员可见） */}
      {isAdmin && detail && (
        <div className="institution-info-section">
          <h3>发起提案</h3>
          <div className="proposal-type-grid">
            <button
              className="proposal-type-button"
              onClick={() => onCreateProposal?.(
                shenfenId, detail.orgType, detail.name, detail.duoqianAddress, adminWallets
              )}
            >转账</button>
            <button className="proposal-type-button" disabled title="即将上线">换管理员</button>
            <button className="proposal-type-button" disabled title="即将上线">决议销毁</button>
            {detail.orgType === 0 && (
              <>
                <button className="proposal-type-button" disabled title="即将上线">决议发行</button>
                <button className="proposal-type-button" disabled title="即将上线">验证密钥</button>
                <button
                  className="proposal-type-button"
                  onClick={() => onCreateRuntimeUpgrade?.(adminWallets)}
                >状态升级</button>
              </>
            )}
          </div>
        </div>
      )}

      {/* 活跃提案列表 */}
      <div className="institution-info-section">
        <h3>活跃提案（{proposals.length}）</h3>
        {proposals.length === 0 ? (
          <p className="no-data">暂无活跃提案</p>
        ) : (
          <div className="proposal-list">
            {proposals.map((item) => (
              <div
                key={item.proposalId}
                className={`proposal-card ${isAdmin ? 'clickable' : ''}`}
                onClick={() => {
                  if (isAdmin) {
                    onSelectProposal?.(item.proposalId, adminWallets, shenfenId);
                  }
                }}
              >
                <div className="proposal-card-header">
                  <span className="proposal-id">{item.displayId}</span>
                  <div className="proposal-card-header-right">
                    {isAdmin && item.status === 0 && (
                      <span className="vote-entry-badge">可投票</span>
                    )}
                    <span className={`proposal-status status-${item.status}`}>
                      {item.statusLabel}
                    </span>
                  </div>
                </div>
                <div className="proposal-card-body">
                  <div className="proposal-card-tags">
                    <span className="proposal-tag">{item.kindLabel}</span>
                  </div>
                  <div className="proposal-summary">{item.summary}</div>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      <div className="institution-info-section">
        <h3>管理员列表（{detail.admins.length} 人）</h3>
        {detail.admins.length === 0 ? (
          <p className="no-data">暂无数据（需节点运行后查询链上数据）</p>
        ) : (
          <div className="admin-list">
            {detail.admins.map((pubkey, i) => {
              const isMyWallet = adminWallets.some(
                w => w.pubkeyHex.toLowerCase() === pubkey.toLowerCase()
              );
              return (
                <div key={pubkey} className={`admin-item ${isMyWallet ? 'my-wallet' : ''}`}>
                  <span className="admin-index">{i + 1}.</span>
                  <code className="admin-pubkey">{hexToSs58(pubkey)}</code>
                  {isMyWallet && <span className="my-wallet-tag">我的钱包</span>}
                </div>
              );
            })}
          </div>
        )}
      </div>
    </div>
  );
}
