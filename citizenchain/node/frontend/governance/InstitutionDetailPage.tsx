// 机构详情页：机构信息 → 管理员入口 → 发起提案 → 提案列表（分页）。
// 管理员列表折叠到独立的 AdminListPage，点击入口卡片右箭头进入。
import { useEffect, useState, useCallback } from 'react';
import { api, sanitizeError } from '../api';
import { formatBalance, hexToSs58 } from '../format';
import type {
  ActivatedAdmin,
  AdminWalletMatch,
  InstitutionDetail,
  ProposalListItem,
} from './governance-types';

type Props = {
  shenfenId: string;
  onBack: () => void;
  onOpenAdminList?: () => void;
  onSelectProposal?: (proposalId: number, adminWallets: AdminWalletMatch[], shenfenId: string) => void;
  onCreateProposal?: (shenfenId: string, orgType: number, institutionName: string, duoqianAddress: string, adminWallets: AdminWalletMatch[]) => void;
  onCreateRuntimeUpgrade?: (adminWallets: AdminWalletMatch[]) => void;
  onCreateFeeRate?: (shenfenId: string, institutionName: string, adminWallets: AdminWalletMatch[]) => void;
  onCreateSafetyFund?: (adminWallets: AdminWalletMatch[]) => void;
  onCreateSweep?: (shenfenId: string, institutionName: string, adminWallets: AdminWalletMatch[]) => void;
  /** 隐藏返回按钮（用于直接作为 Tab 内容显示时）。 */
  hideBackButton?: boolean;
};

export function InstitutionDetailPage({ shenfenId, onBack, onOpenAdminList, onSelectProposal, onCreateProposal, onCreateRuntimeUpgrade, onCreateFeeRate, onCreateSafetyFund, onCreateSweep, hideBackButton }: Props) {
  const [detail, setDetail] = useState<InstitutionDetail | null>(null);
  const [proposals, setProposals] = useState<ProposalListItem[]>([]);
  const [proposalHasMore, setProposalHasMore] = useState(false);
  const [proposalNextStartId, setProposalNextStartId] = useState<number | null>(null);
  const [loadingMoreProposals, setLoadingMoreProposals] = useState(false);
  const [activatedAdmins, setActivatedAdmins] = useState<ActivatedAdmin[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  const PROPOSAL_PAGE_SIZE = 10;
  const isAdmin = activatedAdmins.length > 0;

  // 将已激活管理员转换为 AdminWalletMatch 格式（兼容现有提案/投票组件）
  const adminWallets: AdminWalletMatch[] = activatedAdmins.map(a => ({
    address: hexToSs58(a.pubkeyHex),
    pubkeyHex: a.pubkeyHex,
    name: '',
  }));

  useEffect(() => {
    setLoading(true);
    Promise.all([
      api.getInstitutionDetail(shenfenId),
      api.getNextProposalId().catch(() => 0),
      api.getActivatedAdmins(shenfenId).catch(() => [] as ActivatedAdmin[]),
    ])
      .then(async ([d, nextId, aa]) => {
        setDetail(d);
        setActivatedAdmins(aa);
        // 加载第一页提案
        if (nextId > 0) {
          try {
            const page = await api.getInstitutionProposalPage(shenfenId, nextId - 1, PROPOSAL_PAGE_SIZE);
            setProposals(page.items);
            setProposalHasMore(page.hasMore);
            if (page.items.length > 0) {
              const lastId = page.items[page.items.length - 1].proposalId;
              setProposalNextStartId(lastId > 0 ? lastId - 1 : null);
            }
          } catch (_) {
            setProposals([]);
          }
        }
        setError(null);
      })
      .catch((e) => setError(sanitizeError(e)))
      .finally(() => setLoading(false));
  }, [shenfenId]);

  // 加载更多提案
  const loadMoreProposals = useCallback(() => {
    if (loadingMoreProposals || proposalNextStartId == null || !proposalHasMore) return;
    setLoadingMoreProposals(true);
    api.getInstitutionProposalPage(shenfenId, proposalNextStartId, PROPOSAL_PAGE_SIZE)
      .then((page) => {
        setProposals(prev => [...prev, ...page.items]);
        setProposalHasMore(page.hasMore);
        if (page.items.length > 0) {
          const lastId = page.items[page.items.length - 1].proposalId;
          setProposalNextStartId(lastId > 0 ? lastId - 1 : null);
        } else {
          setProposalHasMore(false);
        }
      })
      .catch(() => setProposalHasMore(false))
      .finally(() => setLoadingMoreProposals(false));
  }, [shenfenId, loadingMoreProposals, proposalNextStartId, proposalHasMore]);

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

  const activatedCount = activatedAdmins.length;

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

      {/* 机构信息卡片 */}
      <div className="institution-detail-grid">
        <div className="metric-card">
          <div className="metric-label">机构类型 <code className="metric-label-id">{detail.shenfenId}</code></div>
          <div className="metric-value">{detail.orgTypeLabel}</div>
        </div>
        <div className="metric-card">
          <div className="metric-label">机构主账户 <code className="metric-label-id">{hexToSs58(detail.duoqianAddress)}</code></div>
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
        {detail.orgType === 2 && detail.stakingAddress && (
          <div className="metric-card">
            <div className="metric-label">永久质押账户 <code className="metric-label-id">{hexToSs58(detail.stakingAddress)}</code></div>
            <div className="metric-value">
              {detail.stakingBalanceFen != null
                ? formatBalance(detail.stakingBalanceFen)
                : '—'}
            </div>
          </div>
        )}
        {detail.orgType === 2 && detail.feeAddress && (
          <div className="metric-card">
            <div className="metric-label">费用账户 <code className="metric-label-id">{hexToSs58(detail.feeAddress)}</code></div>
            <div className="metric-value">
              {detail.feeBalanceFen != null
                ? formatBalance(detail.feeBalanceFen)
                : '—'}
            </div>
          </div>
        )}
        {detail.orgType === 1 && detail.cbFeeAddress && (
          <div className="metric-card">
            <div className="metric-label">费用账户 <code className="metric-label-id">{hexToSs58(detail.cbFeeAddress)}</code></div>
            <div className="metric-value">
              {detail.cbFeeBalanceFen != null
                ? formatBalance(detail.cbFeeBalanceFen)
                : '—'}
            </div>
          </div>
        )}
        {detail.nrcFeeAddress && (
          <div className="metric-card">
            <div className="metric-label">费用账户 <code className="metric-label-id">{hexToSs58(detail.nrcFeeAddress)}</code></div>
            <div className="metric-value">
              {detail.nrcFeeBalanceFen != null
                ? formatBalance(detail.nrcFeeBalanceFen)
                : '—'}
            </div>
          </div>
        )}
        {detail.nrcAnquanAddress && (
          <div className="metric-card">
            <div className="metric-label">安全基金账户 <code className="metric-label-id">{hexToSs58(detail.nrcAnquanAddress)}</code></div>
            <div className="metric-value">
              {detail.nrcAnquanBalanceFen != null
                ? formatBalance(detail.nrcAnquanBalanceFen)
                : '—'}
            </div>
          </div>
        )}
      </div>

      {/* 管理员入口卡片（折叠，点击进入管理员列表页） */}
      <div className="institution-info-section">
        <div
          className="metric-card admin-entry-card"
          onClick={onOpenAdminList}
          role="button"
          tabIndex={0}
          onKeyDown={(e) => e.key === 'Enter' && onOpenAdminList?.()}
        >
          <div className="admin-entry-left">
            <div className="admin-entry-title">管理员列表（{detail.admins.length} 人）</div>
            {activatedCount > 0 && (
              <div className="admin-entry-activated">已激活 {activatedCount} 人</div>
            )}
          </div>
          <span className="admin-entry-arrow">→</span>
        </div>
      </div>

      {/* 提案类型入口（始终显示，未激活时灰色禁用） */}
      <div className="institution-info-section">
        <h3>发起提案</h3>
        <div className="proposal-type-grid">
          <button
            className="proposal-type-button"
            disabled={!isAdmin}
            onClick={() => isAdmin && onCreateProposal?.(
              shenfenId, detail.orgType, detail.name, detail.duoqianAddress, adminWallets
            )}
          >转账</button>
          <button className="proposal-type-button" disabled title="即将上线">换管理员</button>
          <button className="proposal-type-button" disabled title="即将上线">决议销毁</button>
          {detail.orgType === 2 && (
            <button
              className="proposal-type-button"
              disabled={!isAdmin}
              onClick={() => isAdmin && onCreateFeeRate?.(shenfenId, detail.name, adminWallets)}
            >费率设置</button>
          )}
          {(detail.orgType === 0 || detail.orgType === 2) && (
            <button
              className="proposal-type-button"
              disabled={!isAdmin}
              onClick={() => isAdmin && onCreateSweep?.(shenfenId, detail.name, adminWallets)}
            >手续费划转</button>
          )}
          {detail.orgType === 0 && (
            <button
              className="proposal-type-button"
              disabled={!isAdmin}
              onClick={() => isAdmin && onCreateSafetyFund?.(adminWallets)}
            >安全基金转账</button>
          )}
          {(detail.orgType === 0 || detail.orgType === 1) && (
            <>
              <button className="proposal-type-button" disabled title="即将上线">决议发行</button>
              <button className="proposal-type-button" disabled title="即将上线">验证密钥</button>
              <button
                className="proposal-type-button"
                disabled={!isAdmin}
                onClick={() => isAdmin && onCreateRuntimeUpgrade?.(adminWallets)}
              >状态升级</button>
            </>
          )}
        </div>
        {!isAdmin && (
          <p className="no-data">激活管理员后可操作提案功能</p>
        )}
      </div>

      {/* 提案列表（分页） */}
      <div className="institution-info-section">
        <h3>提案列表{proposals.length > 0 ? `（${proposals.length}${proposalHasMore ? '+' : ''}）` : ''}</h3>
        {proposals.length === 0 ? (
          <p className="no-data">暂无提案</p>
        ) : (
          <div className="proposal-list">
            {proposals.map((item) => (
              <div
                key={item.proposalId}
                className="proposal-card clickable"
                onClick={() => {
                  onSelectProposal?.(item.proposalId, adminWallets, shenfenId);
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
                    {item.kind === 1 && (
                      <span className="proposal-tag">{item.stageLabel}</span>
                    )}
                  </div>
                  <div className="proposal-summary">{item.summary}</div>
                </div>
              </div>
            ))}
          </div>
        )}
        {proposalHasMore && (
          <button
            className="load-more-button"
            onClick={loadMoreProposals}
            disabled={loadingMoreProposals}
          >
            {loadingMoreProposals ? '加载中…' : '加载更多'}
          </button>
        )}
      </div>
    </div>
  );
}
