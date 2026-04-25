// 国储会顶级 Section：直接渲染国储会机构详情页（单机构，无列表层）。
// 特权动作：runtime 升级提案、安全基金转账提案 — 仅国储会管理员可发起。
// 子视图类型来自原 GovernanceSection.tsx 的 NRC 分支拆分（2026-04-24 重构）。
import { useState } from 'react';
import { AdminListPage } from './AdminListPage';
import { InstitutionDetailPage } from './InstitutionDetailPage';
import { ProposalDetailPage } from './ProposalDetailPage';
import { CreateProposalPage } from './CreateProposalPage';
import { SafetyFundProposalPage } from './SafetyFundProposalPage';
import { SweepProposalPage } from './SweepProposalPage';
import { RuntimeUpgradeProposalPage } from './RuntimeUpgradeProposalPage';
import type { AdminWalletMatch } from './governance-types';

// 国储会 shenfenId（全链唯一，直接进入详情）。
const NRC_SHENFEN_ID = 'GFR-LN001-CB0C-617776487-20260222';

type NrcView =
  | { page: 'detail' }
  | { page: 'admin-list' }
  | { page: 'proposal-detail'; proposalId: number; adminWallets: AdminWalletMatch[]; shenfenId?: string }
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
        shenfenId={NRC_SHENFEN_ID}
        onBack={backToDetail}
      />
    );
  }

  if (view.page === 'proposal-detail') {
    return (
      <ProposalDetailPage
        proposalId={view.proposalId}
        adminWallets={view.adminWallets}
        shenfenId={view.shenfenId}
        onBack={backToDetail}
      />
    );
  }

  if (view.page === 'create-proposal') {
    return (
      <CreateProposalPage
        shenfenId={NRC_SHENFEN_ID}
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
        shenfenId={NRC_SHENFEN_ID}
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
      shenfenId={NRC_SHENFEN_ID}
      onBack={backToDetail}
      hideBackButton
      onOpenAdminList={() => setView({ page: 'admin-list' })}
      onSelectProposal={(proposalId, adminWallets, sid) =>
        setView({ page: 'proposal-detail', proposalId, adminWallets, shenfenId: sid })
      }
      onCreateProposal={(_sid, orgType, name, mainAddress, aw) =>
        setView({ page: 'create-proposal', orgType, institutionName: name, mainAddress, adminWallets: aw })
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
