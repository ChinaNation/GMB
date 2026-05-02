// 清算行机构详情页(链上 duoqian-manage::Institutions[sfid_id] 已存在时展示)。
//
// 风格参考 governance/InstitutionDetailPage 的卡片栅格 + 折叠子页入口,
// 但数据源全部走 chain/duoqian-manage,通过 offchainApi.fetchInstitutionDetail 获取。
//
// 顶部按钮根据本机是否已声明清算行节点切换:
//   - 未声明 → "声明本机为清算行节点" → declare-node
//   - 已声明 → "查看节点详情(端点/管理员/操作)" → 传统 detail 页(节点 RPC 端点 + 端点更新/注销)

import { useEffect, useState } from 'react';
import { sanitizeError } from '../../core/tauri';
import { offchainApi } from '../api';
import type {
  ClearingBankNodeOnChainInfo,
  InstitutionDetail,
  InstitutionProposalItem,
} from '../types';

type Props = {
  sfidId: string;
  onBack: () => void;
  onOpenOtherAccounts: (detail: InstitutionDetail) => void;
  onOpenAdminList: (detail: InstitutionDetail) => void;
  onDeclareNode: (sfidId: string, institutionName: string) => void;
};

const PROPOSAL_PAGE_SIZE = 10;

export function ClearingBankInstitutionDetailPage({
  sfidId,
  onBack,
  onOpenOtherAccounts,
  onOpenAdminList,
  onDeclareNode,
}: Props) {
  const [detail, setDetail] = useState<InstitutionDetail | null>(null);
  const [nodeInfo, setNodeInfo] = useState<ClearingBankNodeOnChainInfo | null>(null);
  const [proposals, setProposals] = useState<InstitutionProposalItem[]>([]);
  const [proposalHasMore, setProposalHasMore] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    let cancelled = false;
    setLoading(true);
    setError(null);
    Promise.all([
      offchainApi.fetchInstitutionDetail(sfidId),
      offchainApi.queryClearingBankNodeInfo(sfidId).catch(() => null),
      offchainApi
        .fetchInstitutionProposals(sfidId, 0, PROPOSAL_PAGE_SIZE)
        .catch(() => ({ items: [], hasMore: false })),
    ])
      .then(([d, n, page]) => {
        if (cancelled) return;
        setDetail(d);
        setNodeInfo(n);
        setProposals(page.items);
        setProposalHasMore(page.hasMore);
      })
      .catch((e) => {
        if (!cancelled) setError(sanitizeError(e));
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });
    return () => {
      cancelled = true;
    };
  }, [sfidId]);

  if (loading) {
    return (
      <>
        <button className="back-button" onClick={onBack}>← 返回</button>
        <p>加载中…</p>
      </>
    );
  }

  if (error) {
    return (
      <>
        <button className="back-button" onClick={onBack}>← 返回</button>
        <div className="error">{error}</div>
      </>
    );
  }

  if (!detail) {
    return (
      <>
        <button className="back-button" onClick={onBack}>← 返回</button>
        <p className="no-data">未找到机构详情</p>
      </>
    );
  }

  return (
    <>
      <button className="back-button" onClick={onBack}>← 返回</button>

      <div className="institution-title-row">
        <h2>{detail.institutionName}</h2>
        {detail.status === 'Pending' && (
          <span className="status-badge status-pending" style={{ marginLeft: 8 }}>
            创建提案投票中
          </span>
        )}
        {!nodeInfo && detail.status === 'Active' && (
          <button
            className="primary-button"
            style={{ marginLeft: 'auto' }}
            onClick={() => onDeclareNode(sfidId, detail.institutionName)}
          >
            声明本机为清算行节点 →
          </button>
        )}
      </div>

      {/* 已声明节点的对外端点信息(只读展示;后续端点更新/注销可作 follow-up) */}
      {nodeInfo && (
        <div className="node-info-panel metric-card">
          <h3>清算行节点(本机已声明)</h3>
          <dl>
            <dt>PeerId</dt>
            <dd><code>{nodeInfo.peerId}</code></dd>
            <dt>RPC 端点</dt>
            <dd>{nodeInfo.rpcDomain}:{nodeInfo.rpcPort}</dd>
            <dt>注册区块</dt>
            <dd>#{nodeInfo.registeredAt}</dd>
            <dt>声明账户</dt>
            <dd><code>{nodeInfo.registeredBySs58}</code></dd>
          </dl>
        </div>
      )}

      {/* 机构信息卡片栅格 */}
      <div className="institution-detail-grid">
        <div className="metric-card">
          <div className="metric-label">
            机构身份ID <code className="metric-label-id">{detail.sfidId}</code>
          </div>
          <div className="metric-value">{detail.sfidId}</div>
        </div>

        <div className="metric-card">
          <div className="metric-label">
            主账户 <code className="metric-label-id">{detail.mainAccount.addressSs58}</code>
          </div>
          <div className="metric-value">{detail.mainAccount.balanceText} 元</div>
        </div>

        <div className="metric-card">
          <div className="metric-label">内部投票阈值</div>
          <div className="metric-value">
            {detail.threshold} / {detail.adminCount} 票
          </div>
        </div>

        <div className="metric-card">
          <div className="metric-label">
            费用账户 <code className="metric-label-id">{detail.feeAccount.addressSs58}</code>
          </div>
          <div className="metric-value">{detail.feeAccount.balanceText} 元</div>
        </div>
      </div>

      {/* 其他账户列表(折叠卡片入口) */}
      <div className="institution-info-section">
        <div
          className="metric-card admin-entry-card"
          role="button"
          tabIndex={0}
          onClick={() => onOpenOtherAccounts(detail)}
          onKeyDown={(e) => e.key === 'Enter' && onOpenOtherAccounts(detail)}
        >
          <div className="admin-entry-left">
            <div className="admin-entry-title">
              其他账户列表（{detail.otherAccounts.length} 个）
            </div>
          </div>
          <span className="admin-entry-arrow">→</span>
        </div>
      </div>

      {/* 管理员列表(折叠卡片入口) */}
      <div className="institution-info-section">
        <div
          className="metric-card admin-entry-card"
          role="button"
          tabIndex={0}
          onClick={() => onOpenAdminList(detail)}
          onKeyDown={(e) => e.key === 'Enter' && onOpenAdminList(detail)}
        >
          <div className="admin-entry-left">
            <div className="admin-entry-title">
              管理员列表（{detail.duoqianAdminsSs58.length} 人）
            </div>
          </div>
          <span className="admin-entry-arrow">→</span>
        </div>
      </div>

      {/* 发起提案按钮组(占位:duoqian-manage 各类提案 follow-up) */}
      <div className="institution-info-section">
        <h3>发起提案</h3>
        <div className="proposal-type-grid">
          <button className="proposal-type-button" disabled title="即将上线">转账</button>
          <button className="proposal-type-button" disabled title="即将上线">换管理员</button>
          <button className="proposal-type-button" disabled title="即将上线">关闭多签</button>
          <button className="proposal-type-button" disabled title="即将上线">手续费划转</button>
        </div>
        <p className="no-data">机构内提案类型即将上线(本任务先做创建机构主流程)</p>
      </div>

      {/* 提案列表(分页占位,full scan 留 follow-up) */}
      <div className="institution-info-section">
        <h3>
          提案列表
          {proposals.length > 0 ? `（${proposals.length}${proposalHasMore ? '+' : ''}）` : ''}
        </h3>
        {proposals.length === 0 ? (
          <p className="no-data">暂无提案</p>
        ) : (
          <div className="proposal-list">
            {proposals.map((item) => (
              <div key={item.proposalId} className="proposal-card">
                <div className="proposal-card-header">
                  <span className="proposal-id">#{item.proposalId}</span>
                  <span className="proposal-status">{item.statusLabel}</span>
                </div>
                <div className="proposal-card-body">
                  <span className="proposal-tag">{item.kindLabel}</span>
                  <div className="proposal-summary">{item.summary}</div>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </>
  );
}
