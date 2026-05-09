// 国储会顶级 Section：直接渲染国储会机构详情页（单机构，无列表层）。
// 特权动作：runtime 升级提案、安全基金转账提案 — 仅国储会管理员可发起。
// 子视图类型来自原 GovernanceSection.tsx 的 NRC 分支拆分（2026-04-24 重构）。
import { useState } from 'react';
import { AdminListPage, AdminSetChangePage } from './admins_change';
import { InstitutionDetailPage } from './InstitutionDetailPage';
import { ProposalDetailPage } from './ProposalDetailPage';
import { CreateDuoqianTransferPage } from '../duoqian-transfer/CreateProposalPage';
import { SafetyFundProposalPage } from '../duoqian-transfer/SafetyFundProposalPage';
import { SweepProposalPage } from '../duoqian-transfer/SweepProposalPage';
import { RuntimeUpgradeProposalPage } from './RuntimeUpgradeProposalPage';
import type { AdminWalletMatch } from './types';

// 国储会 sfidNumber（全链唯一，直接进入详情）。
const NRC_SFID_NUMBER = 'GFR-LN001-CB0X-944805165-2026';

type NrcView =
  | { page: 'detail' }
  | { page: 'admin-list' }
  | { page: 'admin-set-change'; institutionName: string; adminWallets: AdminWalletMatch[] }
  | { page: 'proposal-detail'; proposalId: number; adminWallets: AdminWalletMatch[]; sfidNumber?: string }
  | { page: 'create-proposal'; orgType: number; institutionName: string; mainAddress: string; adminWallets: AdminWalletMatch[] }
  | { page: 'propose-upgrade'; adminWallets: AdminWalletMatch[] }
  | { page: 'propose-safety-fund'; adminWallets: AdminWalletMatch[] }
  | { page: 'propose-sweep'; institutionName: string; adminWallets: AdminWalletMatch[] };

export function NrcSection() {
  const [view, setView] = useState<NrcView>({ page: 'detail' });

  const backToDetail = () => setView({ page: 'detail' });

  if (view.page === 'admin-list') {
    return (
      <AdminListPage
        sfidNumber={NRC_SFID_NUMBER}
        onBack={backToDetail}
      />
    );
  }

  if (view.page === 'proposal-detail') {
    return (
      <ProposalDetailPage
        proposalId={view.proposalId}
        adminWallets={view.adminWallets}
        sfidNumber={view.sfidNumber}
        onBack={backToDetail}
      />
    );
  }

  if (view.page === 'admin-set-change') {
    return (
      <AdminSetChangePage
        sfidNumber={NRC_SFID_NUMBER}
        institutionName={view.institutionName}
        adminWallets={view.adminWallets}
        onBack={backToDetail}
        onSuccess={backToDetail}
      />
    );
  }

  if (view.page === 'create-proposal') {
    return (
      <CreateDuoqianTransferPage
        sfidNumber={NRC_SFID_NUMBER}
        orgType={view.orgType}
        institutionName={view.institutionName}
        mainAddress={view.mainAddress}
        adminWallets={view.adminWallets}
        onBack={backToDetail}
        onSuccess={backToDetail}
      />
    );
  }

  if (view.page === 'propose-upgrade') {
    return (
      <RuntimeUpgradeProposalPage
        adminWallets={view.adminWallets}
        onBack={backToDetail}
        onSuccess={backToDetail}
      />
    );
  }

  if (view.page === 'propose-safety-fund') {
    return (
      <SafetyFundProposalPage
        adminWallets={view.adminWallets}
        onBack={backToDetail}
        onSuccess={backToDetail}
      />
    );
  }

  if (view.page === 'propose-sweep') {
    return (
      <SweepProposalPage
        sfidNumber={NRC_SFID_NUMBER}
        institutionName={view.institutionName}
        adminWallets={view.adminWallets}
        onBack={backToDetail}
        onSuccess={backToDetail}
      />
    );
  }

  // 默认直接渲染国储会机构详情（hideBackButton 以保持 tab 语义）。
  return (
    <InstitutionDetailPage
      sfidNumber={NRC_SFID_NUMBER}
      onBack={backToDetail}
      hideBackButton
      onOpenAdminList={() => setView({ page: 'admin-list' })}
      onSelectProposal={(proposalId, adminWallets, sid) =>
        setView({ page: 'proposal-detail', proposalId, adminWallets, sfidNumber: sid })
      }
      onCreateProposal={(_sid, orgType, name, mainAddress, aw) =>
        setView({ page: 'create-proposal', orgType, institutionName: name, mainAddress, adminWallets: aw })
      }
      onCreateAdminSetChange={(_sid, name, aw) =>
        setView({ page: 'admin-set-change', institutionName: name, adminWallets: aw })
      }
      onCreateRuntimeUpgrade={(aw) =>
        setView({ page: 'propose-upgrade', adminWallets: aw })
      }
      onCreateSafetyFund={(aw) =>
        setView({ page: 'propose-safety-fund', adminWallets: aw })
      }
      onCreateSweep={(_sid, name, aw) =>
        setView({ page: 'propose-sweep', institutionName: name, adminWallets: aw })
      }
    />
  );
}
