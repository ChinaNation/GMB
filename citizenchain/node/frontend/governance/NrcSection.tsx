// 国家储委会顶级 Section：直接渲染国家储委会机构详情页（单机构，无列表层）。
// 特权动作：协议升级、开发升级、安全基金转账提案 — 仅国家储委会管理员可发起。
import { useState } from 'react';
import { AdminListPage } from '../admins';
import { InstitutionDetailPage } from './InstitutionDetailPage';
import { ProposalDetailPage } from './ProposalDetailPage';
import { CreateMultisigTransferPage } from '../transaction/multisig/CreateProposalPage';
import { SafetyFundProposalPage } from '../transaction/multisig/SafetyFundProposalPage';
import { SweepProposalPage } from '../transaction/multisig/SweepProposalPage';
import { DeveloperUpgradePage, ProtocolUpgradeProposalPage } from './runtime-upgrade';
import type { AdminWalletMatch } from './types';

// 国家储委会 cidNumber（全链唯一，直接进入详情）。
const NRC_CID_NUMBER = 'LN001-NRC0G-944805165-2026';

type NrcView =
  | { page: 'detail' }
  | { page: 'admin-list' }
  | { page: 'proposal-detail'; proposalId: number; adminWallets: AdminWalletMatch[]; cidNumber?: string }
  | { page: 'create-proposal'; orgType: number; cidFullName: string; institutionAccount: string; adminWallets: AdminWalletMatch[] }
  | { page: 'protocol-upgrade'; adminWallets: AdminWalletMatch[] }
  | { page: 'developer-upgrade'; adminWallets: AdminWalletMatch[] }
  | { page: 'propose-safety-fund'; actorCidNumber: string; institutionAccount: string; adminWallets: AdminWalletMatch[] }
  | { page: 'propose-sweep'; actorCidNumber: string; institutionAccount: string; cidFullName: string; adminWallets: AdminWalletMatch[] };

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

  if (view.page === 'create-proposal') {
    return (
      <CreateMultisigTransferPage
        cidNumber={NRC_CID_NUMBER}
        cidFullName={view.cidFullName}
        institutionAccount={view.institutionAccount}
        adminWallets={view.adminWallets}
        onBack={backToDetail}
        onSuccess={backToDetail}
      />
    );
  }

  if (view.page === 'protocol-upgrade') {
    return (
      <ProtocolUpgradeProposalPage
        actorCidNumber={NRC_CID_NUMBER}
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
        actorCidNumber={view.actorCidNumber}
        institutionAccount={view.institutionAccount}
        adminWallets={view.adminWallets}
        onBack={backToDetail}
        onSuccess={backToDetail}
      />
    );
  }

  if (view.page === 'propose-sweep') {
    return (
      <SweepProposalPage
        actorCidNumber={view.actorCidNumber}
        institutionAccount={view.institutionAccount}
        cidFullName={view.cidFullName}
        adminWallets={view.adminWallets}
        onBack={backToDetail}
        onSuccess={backToDetail}
      />
    );
  }

  // 默认直接渲染国家储委会机构详情（hideBackButton 以保持 tab 语义）。
  return (
    <InstitutionDetailPage
      cidNumber={NRC_CID_NUMBER}
      onBack={backToDetail}
      hideBackButton
      onOpenAdminList={() => setView({ page: 'admin-list' })}
      onSelectProposal={(proposalId, adminWallets, sid) =>
        setView({ page: 'proposal-detail', proposalId, adminWallets, cidNumber: sid })
      }
      onCreateProposal={(_sid, orgType, cidFullName, institutionAccount, aw) =>
        setView({ page: 'create-proposal', orgType, cidFullName, institutionAccount, adminWallets: aw })
      }
      onCreateProtocolUpgrade={(aw) =>
        setView({ page: 'protocol-upgrade', adminWallets: aw })
      }
      onCreateDeveloperUpgrade={(aw) =>
        setView({ page: 'developer-upgrade', adminWallets: aw })
      }
      onCreateSafetyFund={(actorCidNumber, institutionAccount, aw) =>
        setView({ page: 'propose-safety-fund', actorCidNumber, institutionAccount, adminWallets: aw })
      }
      onCreateSweep={(actorCidNumber, institutionAccount, cidFullName, aw) =>
        setView({ page: 'propose-sweep', actorCidNumber, institutionAccount, cidFullName, adminWallets: aw })
      }
    />
  );
}
