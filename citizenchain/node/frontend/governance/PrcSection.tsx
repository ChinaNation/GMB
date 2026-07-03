// 省储委会顶级 Section：列表（orgType=1）→ 机构详情 两级导航。
import { useState } from 'react';
import { AdminListPage, AdminSetChangePage } from '../admins/admin-management';
import { InstitutionListView } from './InstitutionListView';
import { InstitutionDetailPage } from './InstitutionDetailPage';
import { ProposalDetailPage } from './ProposalDetailPage';
import { CreateMultisigTransferPage } from '../transaction/multisig-transfer/CreateProposalPage';
import { SweepProposalPage } from '../transaction/multisig-transfer/SweepProposalPage';
import { ProtocolUpgradeProposalPage } from './runtime-upgrade';
import type { AdminWalletMatch } from './types';

type PrcView =
  | { page: 'list' }
  | { page: 'detail'; cidNumber: string }
  | { page: 'admin-list'; cidNumber: string; orgType: number }
  | { page: 'admin-set-change'; cidNumber: string; orgType: number; cidFullName: string; adminWallets: AdminWalletMatch[] }
  | { page: 'proposal-detail'; proposalId: number; adminWallets: AdminWalletMatch[]; cidNumber?: string; originCidNumber: string }
  | { page: 'create-proposal'; cidNumber: string; orgType: number; cidFullName: string; mainAccount: string; adminWallets: AdminWalletMatch[] }
  | { page: 'protocol-upgrade'; cidNumber: string; adminWallets: AdminWalletMatch[] }
  | { page: 'propose-sweep'; cidNumber: string; cidFullName: string; adminWallets: AdminWalletMatch[] };

export function PrcSection() {
  const [view, setView] = useState<PrcView>({ page: 'list' });

  const backToList = () => setView({ page: 'list' });
  const backToDetail = (cidNumber: string) => setView({ page: 'detail', cidNumber });

  if (view.page === 'admin-list') {
    return (
      <AdminListPage
        cidNumber={view.cidNumber}
        accountRef={{ cidNumber: view.cidNumber }}
        onBack={() => backToDetail(view.cidNumber)}
      />
    );
  }

  if (view.page === 'proposal-detail') {
    return (
      <ProposalDetailPage
        proposalId={view.proposalId}
        adminWallets={view.adminWallets}
        cidNumber={view.cidNumber}
        onBack={() => backToDetail(view.originCidNumber)}
      />
    );
  }

  if (view.page === 'create-proposal') {
    return (
      <CreateMultisigTransferPage
        cidNumber={view.cidNumber}
        institutionCode="PRC"
        cidFullName={view.cidFullName}
        mainAccount={view.mainAccount}
        adminWallets={view.adminWallets}
        onBack={() => backToDetail(view.cidNumber)}
        onSuccess={() => backToDetail(view.cidNumber)}
      />
    );
  }

  if (view.page === 'admin-set-change') {
    return (
      <AdminSetChangePage
        accountRef={{ cidNumber: view.cidNumber }}
        cidFullName={view.cidFullName}
        adminWallets={view.adminWallets}
        onBack={() => backToDetail(view.cidNumber)}
        onSuccess={() => backToDetail(view.cidNumber)}
      />
    );
  }

  if (view.page === 'protocol-upgrade') {
    return (
      <ProtocolUpgradeProposalPage
        adminWallets={view.adminWallets}
        onBack={() => backToDetail(view.cidNumber)}
        onSuccess={() => backToDetail(view.cidNumber)}
      />
    );
  }

  if (view.page === 'propose-sweep') {
    return (
      <SweepProposalPage
        cidNumber={view.cidNumber}
        cidFullName={view.cidFullName}
        adminWallets={view.adminWallets}
        onBack={() => backToDetail(view.cidNumber)}
        onSuccess={() => backToDetail(view.cidNumber)}
      />
    );
  }

  if (view.page === 'detail') {
    const cidNumber = view.cidNumber;
    return (
      <InstitutionDetailPage
        cidNumber={cidNumber}
        onBack={backToList}
        onOpenAdminList={(sid, orgType) => setView({ page: 'admin-list', cidNumber: sid, orgType })}
        onSelectProposal={(proposalId, adminWallets, sid) =>
          setView({ page: 'proposal-detail', proposalId, adminWallets, cidNumber: sid, originCidNumber: cidNumber })
        }
        onCreateProposal={(sid, orgType, cidFullName, mainAccount, aw) =>
          setView({ page: 'create-proposal', cidNumber: sid, orgType, cidFullName, mainAccount, adminWallets: aw })
        }
        onCreateAdminSetChange={(sid, orgType, cidFullName, aw) =>
          setView({ page: 'admin-set-change', cidNumber: sid, orgType, cidFullName, adminWallets: aw })
        }
        onCreateProtocolUpgrade={(aw) =>
          setView({ page: 'protocol-upgrade', cidNumber, adminWallets: aw })
        }
        onCreateSweep={(sid, cidFullName, aw) =>
          setView({ page: 'propose-sweep', cidNumber: sid, cidFullName, adminWallets: aw })
        }
      />
    );
  }

  // 默认：省储委会机构列表（orgTypeFilter=1）。
  return (
    <InstitutionListView
      orgTypeFilter={1}
      onSelect={(cidNumber) => setView({ page: 'detail', cidNumber })}
    />
  );
}
