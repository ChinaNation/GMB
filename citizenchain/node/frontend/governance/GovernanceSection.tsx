// 治理主页：子 Tab 排序为 提案 / 国储会 / 省储会 / 省储行 / 钱包管理。
// 国储会直接显示详情页（不经过列表），省储会和省储行各自独立列表。
import { useState } from 'react';
import { AdminListPage } from './AdminListPage';
import { InstitutionListView } from './InstitutionListView';
import { InstitutionDetailPage } from './InstitutionDetailPage';
import { ProposalListView } from './ProposalListView';
import { ProposalDetailPage } from './ProposalDetailPage';
import { CreateProposalPage } from './CreateProposalPage';
import { FeeRateProposalPage } from './FeeRateProposalPage';
import { SafetyFundProposalPage } from './SafetyFundProposalPage';
import { SweepProposalPage } from './SweepProposalPage';
// ColdWalletManager 已删除，管理员激活改为机构详情页内操作。
import { DeveloperUpgradePage } from './DeveloperUpgradePage';
import { RuntimeUpgradeProposalPage } from './RuntimeUpgradeProposalPage';
import type { AdminWalletMatch } from './governance-types';

/// 国储会 shenfenId（只有 1 个，直接进详情）。
const NRC_SHENFEN_ID = 'GFR-LN001-CB0C-617776487-20260222';

// 注意:从 NRC 发起的提案页(propose-sweep / propose-fee-rate)返回时,
// backTab='nrc' 必须直接回到 { page: 'nrc' } tab 状态,
// 不能跳 { page: 'institution-detail' } —— 后者是 PRC/PRB 从列表进入的通用机构详情页,
// 与 NRC tab 结构不同(无治理子 Tab 栏、多"返回机构列表"按钮、
// 缺 onCreateSafetyFund/onCreateRuntimeUpgrade handler)。
// 具体分发逻辑见 backToInstitutionParent。
type GovernanceView =
  | { page: 'proposals' }
  | { page: 'nrc' }
  | { page: 'prc' }
  | { page: 'prb' }
  | { page: 'dev-upgrade' }
  | { page: 'institution-detail'; shenfenId: string; backTab: SubTab }
  | { page: 'admin-list'; shenfenId: string; backTab: SubTab }
  | { page: 'proposal-detail'; proposalId: number; adminWallets: AdminWalletMatch[]; shenfenId?: string; backTab: SubTab }
  | { page: 'create-proposal'; shenfenId: string; orgType: number; institutionName: string; duoqianAddress: string; adminWallets: AdminWalletMatch[]; backTab: SubTab }
  | { page: 'propose-upgrade'; adminWallets: AdminWalletMatch[]; backTab: SubTab }
  | { page: 'propose-fee-rate'; shenfenId: string; institutionName: string; adminWallets: AdminWalletMatch[]; backTab: SubTab }
  | { page: 'propose-safety-fund'; adminWallets: AdminWalletMatch[]; backTab: SubTab }
  | { page: 'propose-sweep'; shenfenId: string; institutionName: string; adminWallets: AdminWalletMatch[]; backTab: SubTab };

type SubTab = 'proposals' | 'nrc' | 'prc' | 'prb' | 'dev-upgrade';

export function GovernanceSection() {
  const [view, setView] = useState<GovernanceView>({ page: 'proposals' });
  const [activeTab, setActiveTab] = useState<SubTab>('proposals');

  const switchTab = (tab: SubTab) => {
    setActiveTab(tab);
    setView({ page: tab });
  };

  /// 机构详情类提案页(sweep / fee-rate)的返回父级分发:
  ///   - NRC(backTab='nrc')→ 直接回 tab 状态 { page: 'nrc' },因为 NRC 的"真正主页"
  ///     是 tab 下嵌入的内联渲染(带治理子 Tab 栏),与通用 institution-detail 视图结构不同。
  ///   - PRC/PRB → 回到通用 institution-detail 视图(来时即从此处进入)。
  const backToInstitutionParent = (backTab: SubTab, shenfenId: string) => {
    if (backTab === 'nrc') {
      setView({ page: 'nrc' });
    } else {
      setView({ page: 'institution-detail', shenfenId, backTab });
    }
  };

  const handleCreateProposal = (
    sid: string, orgType: number, name: string, duoqian: string,
    aw: AdminWalletMatch[], backTab: SubTab,
  ) => {
    setView({
      page: 'create-proposal', shenfenId: sid, orgType,
      institutionName: name, duoqianAddress: duoqian, adminWallets: aw, backTab,
    });
  };

  // 创建提案页
  if (view.page === 'create-proposal') {
    return (
      <CreateProposalPage
        shenfenId={view.shenfenId}
        orgType={view.orgType}
        institutionName={view.institutionName}
        duoqianAddress={view.duoqianAddress}
        adminWallets={view.adminWallets}
        onBack={() => setView({ page: view.backTab })}
        onSuccess={() => setView({ page: view.backTab })}
      />
    );
  }

  // 费率设置提案页
  if (view.page === 'propose-fee-rate') {
    const backToParent = () => backToInstitutionParent(view.backTab, view.shenfenId);
    return (
      <FeeRateProposalPage
        shenfenId={view.shenfenId}
        institutionName={view.institutionName}
        adminWallets={view.adminWallets}
        onBack={backToParent}
        onSuccess={backToParent}
      />
    );
  }

  // 手续费划转提案页
  if (view.page === 'propose-sweep') {
    const backToParent = () => backToInstitutionParent(view.backTab, view.shenfenId);
    return (
      <SweepProposalPage
        shenfenId={view.shenfenId}
        institutionName={view.institutionName}
        adminWallets={view.adminWallets}
        onBack={backToParent}
        onSuccess={backToParent}
      />
    );
  }

  // 安全基金转账提案页
  if (view.page === 'propose-safety-fund') {
    return (
      <SafetyFundProposalPage
        adminWallets={view.adminWallets}
        onBack={() => setView({ page: view.backTab })}
        onSuccess={() => setView({ page: view.backTab })}
      />
    );
  }

  // Runtime 升级提案页
  if (view.page === 'propose-upgrade') {
    return (
      <RuntimeUpgradeProposalPage
        adminWallets={view.adminWallets}
        onBack={() => setView({ page: view.backTab })}
        onSuccess={() => setView({ page: view.backTab })}
      />
    );
  }

  // 提案详情页
  if (view.page === 'proposal-detail') {
    return (
      <ProposalDetailPage
        proposalId={view.proposalId}
        adminWallets={view.adminWallets}
        shenfenId={view.shenfenId}
        onBack={() => setView({ page: view.backTab })}
      />
    );
  }

  // 管理员列表页
  // NRC(backTab='nrc')直接回到 Tab；PRC/PRB 回到机构详情页
  if (view.page === 'admin-list') {
    const backToDetail = view.backTab !== 'nrc';
    return (
      <AdminListPage
        shenfenId={view.shenfenId}
        onBack={() => backToDetail
          ? setView({ page: 'institution-detail', shenfenId: view.shenfenId, backTab: view.backTab })
          : setView({ page: view.backTab })
        }
      />
    );
  }

  // 机构详情页（从省储会/省储行列表进入）
  if (view.page === 'institution-detail') {
    return (
      <InstitutionDetailPage
        shenfenId={view.shenfenId}
        onBack={() => setView({ page: view.backTab })}
        onOpenAdminList={() =>
          setView({ page: 'admin-list', shenfenId: view.shenfenId, backTab: view.backTab })
        }
        onSelectProposal={(proposalId, adminWallets, sid) =>
          setView({ page: 'proposal-detail', proposalId, adminWallets, shenfenId: sid, backTab: view.backTab })
        }
        onCreateProposal={(sid, orgType, name, duoqian, aw) =>
          handleCreateProposal(sid, orgType, name, duoqian, aw, view.backTab)
        }
        onCreateFeeRate={(sid, name, aw) =>
          setView({ page: 'propose-fee-rate', shenfenId: sid, institutionName: name, adminWallets: aw, backTab: view.backTab })
        }
        onCreateSweep={(sid, name, aw) =>
          setView({ page: 'propose-sweep', shenfenId: sid, institutionName: name, adminWallets: aw, backTab: view.backTab })
        }
      />
    );
  }

  return (
    <div>
      <div className="governance-sub-tabs">
        <button className={activeTab === 'proposals' ? 'active' : ''} onClick={() => switchTab('proposals')}>提案</button>
        <button className={activeTab === 'nrc' ? 'active' : ''} onClick={() => switchTab('nrc')}>国储会</button>
        <button className={activeTab === 'prc' ? 'active' : ''} onClick={() => switchTab('prc')}>省储会</button>
        <button className={activeTab === 'prb' ? 'active' : ''} onClick={() => switchTab('prb')}>省储行</button>
        <button className={activeTab === 'dev-upgrade' ? 'active' : ''} onClick={() => switchTab('dev-upgrade')}>开发升级</button>
      </div>

      {activeTab === 'proposals' && (
        <ProposalListView
          onSelect={(proposalId) =>
            setView({ page: 'proposal-detail', proposalId, adminWallets: [], backTab: 'proposals' })
          }
        />
      )}

      {activeTab === 'nrc' && (
        <InstitutionDetailPage
          shenfenId={NRC_SHENFEN_ID}
          onBack={() => switchTab('proposals')}
          hideBackButton
          onOpenAdminList={() =>
            setView({ page: 'admin-list', shenfenId: NRC_SHENFEN_ID, backTab: 'nrc' })
          }
          onSelectProposal={(proposalId, adminWallets, sid) =>
            setView({ page: 'proposal-detail', proposalId, adminWallets, shenfenId: sid, backTab: 'nrc' })
          }
          onCreateProposal={(sid, orgType, name, duoqian, aw) =>
            handleCreateProposal(sid, orgType, name, duoqian, aw, 'nrc')
          }
          onCreateRuntimeUpgrade={(aw) =>
            setView({ page: 'propose-upgrade', adminWallets: aw, backTab: 'nrc' })
          }
          onCreateSafetyFund={(aw) =>
            setView({ page: 'propose-safety-fund', adminWallets: aw, backTab: 'nrc' })
          }
          onCreateSweep={(sid, name, aw) =>
            setView({ page: 'propose-sweep', shenfenId: sid, institutionName: name, adminWallets: aw, backTab: 'nrc' })
          }
        />
      )}

      {activeTab === 'prc' && (
        <InstitutionListView orgTypeFilter={1} onSelect={(shenfenId) => setView({ page: 'institution-detail', shenfenId, backTab: 'prc' })} />
      )}

      {activeTab === 'prb' && (
        <InstitutionListView orgTypeFilter={2} onSelect={(shenfenId) => setView({ page: 'institution-detail', shenfenId, backTab: 'prb' })} />
      )}

      {activeTab === 'dev-upgrade' && <DeveloperUpgradePage />}
    </div>
  );
}
