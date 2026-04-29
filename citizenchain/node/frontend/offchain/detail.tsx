// 清算行机构详情页:
//  - 基础信息卡(机构名/sfid_id/主账户/费用账户余额)
//  - 节点信息长卡 node.tsx(端点更新/注销入口)
//  - 管理员列表入口 → admin.tsx("解密"按钮)
//  - 提案按钮:转账 / 手续费划转启用;换管理员 / 费率设置 disabled "即将上线"
//  - 提案列表(分页,复用 InstitutionDetailPage 的查询 API)
//
// 与 governance/InstitutionDetailPage 的关系:
//  - 业务上同根(都是 SFID 机构 + 多签),但本页关注清算行特有视图(节点信息 +
//    解密管理员列表),所以单独维护一个 detail 页,不复用 InstitutionDetailPage。
//  - 读取 institution detail 仍走 api.getInstitutionDetail 复用后端实现。

import { useCallback, useEffect, useState } from 'react';
import { api, sanitizeError } from '../api';
import { hexToSs58, formatBalance } from '../format';
import type {
  ActivatedAdmin,
  AdminWalletMatch,
  InstitutionDetail,
  ProposalListItem,
} from '../governance/governance-types';
import { offchainApi } from './api';
import type { ClearingBankNodeOnChainInfo } from './types';
import { ClearingBankNodeInfoPanel } from './node';
import { ClearingBankAdminListPage } from './admin';
import { CreateProposalPage } from '../governance/CreateProposalPage';
import { SweepProposalPage } from '../governance/SweepProposalPage';
import { ProposalDetailPage } from '../governance/ProposalDetailPage';

type Props = {
  sfidId: string;
  onBack: () => void;
  onUnregistered: () => void;
};

type Sub =
  | { kind: 'main' }
  | { kind: 'admin-list' }
  | { kind: 'create-transfer' }
  | { kind: 'create-sweep' }
  | { kind: 'proposal'; proposalId: number };

const PROPOSAL_PAGE_SIZE = 10;

export function ClearingBankDetailPage({ sfidId, onBack, onUnregistered }: Props) {
  const [sub, setSub] = useState<Sub>({ kind: 'main' });

  const [detail, setDetail] = useState<InstitutionDetail | null>(null);
  const [nodeInfo, setNodeInfo] = useState<ClearingBankNodeOnChainInfo | null>(null);
  const [activatedAdmins, setActivatedAdmins] = useState<ActivatedAdmin[]>([]);
  const [proposals, setProposals] = useState<ProposalListItem[]>([]);
  const [proposalHasMore, setProposalHasMore] = useState(false);
  const [proposalNextStartId, setProposalNextStartId] = useState<number | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  const refresh = useCallback(() => {
    setLoading(true);
    Promise.all([
      api.getInstitutionDetail(sfidId),
      offchainApi.queryClearingBankNodeInfo(sfidId).catch(() => null),
      api.getActivatedAdmins(sfidId).catch(() => [] as ActivatedAdmin[]),
      api.getNextProposalId().catch(() => 0),
    ])
      .then(async ([d, ni, aa, nextId]) => {
        setDetail(d);
        setNodeInfo(ni);
        setActivatedAdmins(aa);
        if (nextId > 0) {
          try {
            const page = await api.getInstitutionProposalPage(sfidId, nextId - 1, PROPOSAL_PAGE_SIZE);
            setProposals(page.items);
            setProposalHasMore(page.hasMore);
            if (page.items.length > 0) {
              const lastId = page.items[page.items.length - 1].proposalId;
              setProposalNextStartId(lastId > 0 ? lastId - 1 : null);
            }
          } catch (_) { setProposals([]); }
        }
        setError(null);
      })
      .catch((e) => setError(sanitizeError(e)))
      .finally(() => setLoading(false));
  }, [sfidId]);

  useEffect(() => { refresh(); }, [refresh]);

  const adminWallets: AdminWalletMatch[] = activatedAdmins.map(a => ({
    address: hexToSs58(a.pubkeyHex),
    pubkeyHex: a.pubkeyHex,
    name: '',
  }));

  if (loading) return <div><p>加载中…</p></div>;
  if (error || !detail) {
    return (
      <div>
        <button className="back-button" onClick={onBack}>← 返回</button>
        {error && <div className="error">{error}</div>}
      </div>
    );
  }

  if (sub.kind === 'admin-list') {
    return (
      <ClearingBankAdminListPage
        shenfenId={sfidId}
        onBack={() => { setSub({ kind: 'main' }); refresh(); }}
      />
    );
  }
  if (sub.kind === 'create-transfer') {
    return (
      <CreateProposalPage
        shenfenId={sfidId}
        orgType={2 /* PRB 占位:清算行复用 PRB 转账提案路径(链上 propose_transfer 接受任何机构 sfid_id);Step 3 联调时再确认是否细分新枚举 */}
        institutionName={detail.name}
        mainAddress={detail.mainAddress}
        adminWallets={adminWallets}
        onBack={() => setSub({ kind: 'main' })}
        onSuccess={() => { setSub({ kind: 'main' }); refresh(); }}
      />
    );
  }
  if (sub.kind === 'create-sweep') {
    return (
      <SweepProposalPage
        shenfenId={sfidId}
        institutionName={detail.name}
        adminWallets={adminWallets}
        onBack={() => setSub({ kind: 'main' })}
        onSuccess={() => { setSub({ kind: 'main' }); refresh(); }}
      />
    );
  }
  if (sub.kind === 'proposal') {
    return (
      <ProposalDetailPage
        proposalId={sub.proposalId}
        adminWallets={adminWallets}
        onBack={() => setSub({ kind: 'main' })}
        shenfenId={sfidId}
      />
    );
  }

  return (
    <div>
      <button className="back-button" onClick={onBack}>← 返回清算行列表</button>

      <div className="admin-list-header">
        <h2>{detail.name}</h2>
        <span className="admin-list-summary">{sfidId}</span>
      </div>

      <div className="metric-grid">
        <div className="metric-card">
          <h3>主账户</h3>
          <code className="admin-card-address">{hexToSs58(detail.mainAddress)}</code>
          <p className="balance">{detail.balanceFen != null ? formatBalance(detail.balanceFen) : '—'}</p>
        </div>
        {detail.feeAddress && (
          <div className="metric-card">
            <h3>费用账户</h3>
            <code className="admin-card-address">{hexToSs58(detail.feeAddress)}</code>
            <p className="balance">{detail.feeBalanceFen != null ? formatBalance(detail.feeBalanceFen) : '—'}</p>
          </div>
        )}
        {detail.cbFeeAddress && (
          <div className="metric-card">
            <h3>清算费用账户(老 CB)</h3>
            <code className="admin-card-address">{hexToSs58(detail.cbFeeAddress)}</code>
            <p className="balance">{detail.cbFeeBalanceFen != null ? formatBalance(detail.cbFeeBalanceFen) : '—'}</p>
          </div>
        )}
        <div
          className="metric-card admin-card"
          role="button"
          tabIndex={0}
          onClick={() => setSub({ kind: 'admin-list' })}
          onKeyDown={(e) => { if (e.key === 'Enter') setSub({ kind: 'admin-list' }); }}
        >
          <h3>管理员列表 →</h3>
          <p>{detail.admins.length} 人 · 已激活 {activatedAdmins.length}</p>
        </div>
      </div>

      {nodeInfo ? (
        <ClearingBankNodeInfoPanel
          info={nodeInfo}
          sfidId={sfidId}
          admins={adminWallets}
          onChanged={refresh}
          onUnregistered={onUnregistered}
        />
      ) : (
        <div className="metric-card">
          <h3>清算行节点信息</h3>
          <p className="muted">该机构尚未声明清算行节点。</p>
        </div>
      )}

      <div className="admin-list-header">
        <h2>提案</h2>
      </div>
      <div className="form-actions">
        <button
          className="primary-button"
          disabled={adminWallets.length === 0}
          onClick={() => setSub({ kind: 'create-transfer' })}
        >发起转账提案</button>
        <button
          className="secondary-button"
          disabled={adminWallets.length === 0}
          onClick={() => setSub({ kind: 'create-sweep' })}
        >发起手续费划转</button>
        <button className="secondary-button" disabled title="即将上线">换管理员</button>
        <button className="secondary-button" disabled title="即将上线">费率设置</button>
      </div>

      {proposals.length === 0 ? (
        <p className="no-data">暂无活跃提案</p>
      ) : (
        <div className="proposal-list">
          {proposals.map((p) => (
            <div
              key={p.proposalId}
              className="metric-card proposal-row"
              role="button"
              tabIndex={0}
              onClick={() => setSub({ kind: 'proposal', proposalId: p.proposalId })}
              onKeyDown={(e) => { if (e.key === 'Enter') setSub({ kind: 'proposal', proposalId: p.proposalId }); }}
            >
              <div>
                <strong>#{p.displayId}</strong> {p.kindLabel} · {p.statusLabel}
              </div>
              <p className="muted">{p.summary}</p>
            </div>
          ))}
          {proposalHasMore && proposalNextStartId !== null && (
            <button
              className="secondary-button"
              onClick={async () => {
                try {
                  const page = await api.getInstitutionProposalPage(sfidId, proposalNextStartId, PROPOSAL_PAGE_SIZE);
                  setProposals(prev => [...prev, ...page.items]);
                  setProposalHasMore(page.hasMore);
                  if (page.items.length > 0) {
                    const lastId = page.items[page.items.length - 1].proposalId;
                    setProposalNextStartId(lastId > 0 ? lastId - 1 : null);
                  }
                } catch (e) {
                  setError(sanitizeError(e));
                }
              }}
            >加载更多</button>
          )}
        </div>
      )}
    </div>
  );
}
