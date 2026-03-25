// 治理主页：子 Tab 排序为 提案 / 国储会 / 省储会 / 省储行 / 钱包管理。
// 国储会直接显示详情页（不经过列表），省储会和省储行各自独立列表。
import { useState } from 'react';
import { InstitutionListView } from './InstitutionListView';
import { InstitutionDetailPage } from './InstitutionDetailPage';
import { ProposalListView } from './ProposalListView';
import { ProposalDetailPage } from './ProposalDetailPage';
import { CreateProposalPage } from './CreateProposalPage';
import { ColdWalletManager } from './ColdWalletManager';
import { DeveloperUpgradePage } from './DeveloperUpgradePage';
import type { AdminWalletMatch } from './governance-types';

/// 国储会 shenfenId（只有 1 个，直接进详情）。
const NRC_SHENFEN_ID = 'GFR-LN001-CB0C-617776487-20260222';

type GovernanceView =
  | { page: 'proposals' }
  | { page: 'nrc' }
  | { page: 'prc' }
  | { page: 'prb' }
  | { page: 'wallets' }
  | { page: 'dev-upgrade' }
  | { page: 'institution-detail'; shenfenId: string; backTab: SubTab }
  | { page: 'proposal-detail'; proposalId: number; adminWallets: AdminWalletMatch[]; shenfenId?: string; backTab: SubTab }
  | { page: 'create-proposal'; shenfenId: string; orgType: number; institutionName: string; duoqianAddress: string; adminWallets: AdminWalletMatch[]; backTab: SubTab };

type SubTab = 'proposals' | 'nrc' | 'prc' | 'prb' | 'wallets' | 'dev-upgrade';

export function GovernanceSection() {
  const [view, setView] = useState<GovernanceView>({ page: 'proposals' });
  const [activeTab, setActiveTab] = useState<SubTab>('proposals');

  const switchTab = (tab: SubTab) => {
    setActiveTab(tab);
    setView({ page: tab });
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

  // 机构详情页（从省储会/省储行列表进入）
  if (view.page === 'institution-detail') {
    return (
      <InstitutionDetailPage
        shenfenId={view.shenfenId}
        onBack={() => setView({ page: view.backTab })}
        onSelectProposal={(proposalId, adminWallets, sid) =>
          setView({ page: 'proposal-detail', proposalId, adminWallets, shenfenId: sid, backTab: view.backTab })
        }
        onCreateProposal={(sid, orgType, name, duoqian, aw) =>
          handleCreateProposal(sid, orgType, name, duoqian, aw, view.backTab)
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
        <button className={activeTab === 'wallets' ? 'active' : ''} onClick={() => switchTab('wallets')}>钱包管理</button>
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
          onSelectProposal={(proposalId, adminWallets, sid) =>
            setView({ page: 'proposal-detail', proposalId, adminWallets, shenfenId: sid, backTab: 'nrc' })
          }
          onCreateProposal={(sid, orgType, name, duoqian, aw) =>
            handleCreateProposal(sid, orgType, name, duoqian, aw, 'nrc')
          }
        />
      )}

      {activeTab === 'prc' && (
        <InstitutionListView orgTypeFilter={1} onSelect={(shenfenId) => setView({ page: 'institution-detail', shenfenId, backTab: 'prc' })} />
      )}

      {activeTab === 'prb' && (
        <InstitutionListView orgTypeFilter={2} onSelect={(shenfenId) => setView({ page: 'institution-detail', shenfenId, backTab: 'prb' })} />
      )}

      {activeTab === 'wallets' && <ColdWalletManager />}

      {activeTab === 'dev-upgrade' && <DeveloperUpgradePage />}
    </div>
  );
}
