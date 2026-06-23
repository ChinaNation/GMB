// 国储会顶级 Section：直接渲染国储会机构详情页（单机构，无列表层）。
// 特权动作：协议升级、开发升级、安全基金转账提案 — 仅国储会管理员可发起。
import { useState } from 'react';
import { AdminListPage, AdminSetChangePage } from './admins-change';
import { InstitutionDetailPage } from './InstitutionDetailPage';
import { ProposalDetailPage } from './ProposalDetailPage';
import { CreateDuoqianTransferPage } from '../transaction/duoqian-transfer/CreateProposalPage';
import { SafetyFundProposalPage } from '../transaction/duoqian-transfer/SafetyFundProposalPage';
import { SweepProposalPage } from '../transaction/duoqian-transfer/SweepProposalPage';
import { DeveloperUpgradePage, ProtocolUpgradeProposalPage } from './runtime-upgrade';
import type { AdminWalletMatch } from './types';

// 国储会 cidNumber（全链唯一，直接进入详情）。
const NRC_CID_NUMBER = 'LN001-NRC0G-944805165-2026';

type NrcView =
  | { page: 'detail' }
  | { page: 'admin-list' }
  | { page: 'admin-set-change'; cidFullName: string; adminWallets: AdminWalletMatch[] }
  | { page: 'proposal-detail'; proposalId: number; adminWallets: AdminWalletMatch[]; cidNumber?: string }
  | { page: 'create-proposal'; orgType: number; cidFullName: string; mainAccount: string; adminWallets: AdminWalletMatch[] }
  | { page: 'protocol-upgrade'; adminWallets: AdminWalletMatch[] }
  | { page: 'developer-upgrade'; adminWallets: AdminWalletMatch[] }
  | { page: 'propose-safety-fund'; adminWallets: AdminWalletMatch[] }
  | { page: 'propose-sweep'; cidFullName: string; adminWallets: AdminWalletMatch[] };

export function NrcSection() {
  const [view, setView] = useState<NrcView>({ page: 'detail' });

  const backToDetail = () => setView({ page: 'detail' });

  if (view.page === 'admin-list') {
    return (
      <AdminListPage
        cidNumber={NRC_CID_NUMBER}
        accountRef={{ cidNumber: NRC_CID_NUMBER, institutionCode: 'NRC' }}
        onBack={backToDetail}
      />
    );
  }

  if (view.page === 'proposal-detail') {
    return (
      <ProposalDetailPage
        proposalId={view.proposalId}
        adminWallets={view.adminWallets}
        cidNumber={view.cidNumber}
        onBack={backToDetail}
      />
    );
  }

  if (view.page === 'admin-set-change') {
    return (
      <AdminSetChangePage
        accountRef={{ cidNumber: NRC_CID_NUMBER, institutionCode: 'NRC' }}
        cidFullName={view.cidFullName}
        adminWallets={view.adminWallets}
        onBack={backToDetail}
        onSuccess={backToDetail}
      />
    );
  }

  if (view.page === 'create-proposal') {
    return (
      <CreateDuoqianTransferPage
        cidNumber={NRC_CID_NUMBER}
        institutionCode="NRC"
        cidFullName={view.cidFullName}
        mainAccount={view.mainAccount}
        adminWallets={view.adminWallets}
        onBack={backToDetail}
        onSuccess={backToDetail}
      />
    );
  }

  if (view.page === 'protocol-upgrade') {
    return (
      <ProtocolUpgradeProposalPage
        adminWallets={view.adminWallets}
        onBack={backToDetail}
        onSuccess={backToDetail}
      />
    );
  }

  if (view.page === 'developer-upgrade') {
    return (
      <DeveloperUpgradePage
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
        cidNumber={NRC_CID_NUMBER}
        cidFullName={view.cidFullName}
        adminWallets={view.adminWallets}
        onBack={backToDetail}
        onSuccess={backToDetail}
      />
    );
  }

  // 默认直接渲染国储会机构详情（hideBackButton 以保持 tab 语义）。
  return (
    <InstitutionDetailPage
      cidNumber={NRC_CID_NUMBER}
      onBack={backToDetail}
      hideBackButton
      onOpenAdminList={() => setView({ page: 'admin-list' })}
      onSelectProposal={(proposalId, adminWallets, sid) =>
        setView({ page: 'proposal-detail', proposalId, adminWallets, cidNumber: sid })
      }
      onCreateProposal={(_sid, orgType, cidFullName, mainAccount, aw) =>
        setView({ page: 'create-proposal', orgType, cidFullName, mainAccount, adminWallets: aw })
      }
      onCreateAdminSetChange={(_sid, _orgType, cidFullName, aw) =>
        setView({ page: 'admin-set-change', cidFullName, adminWallets: aw })
      }
      onCreateProtocolUpgrade={(aw) =>
        setView({ page: 'protocol-upgrade', adminWallets: aw })
      }
      onCreateDeveloperUpgrade={(aw) =>
        setView({ page: 'developer-upgrade', adminWallets: aw })
      }
      onCreateSafetyFund={(aw) =>
        setView({ page: 'propose-safety-fund', adminWallets: aw })
      }
      onCreateSweep={(_sid, cidFullName, aw) =>
        setView({ page: 'propose-sweep', cidFullName, adminWallets: aw })
      }
    />
  );
}
