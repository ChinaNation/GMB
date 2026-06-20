// 省储会顶级 Section：列表（orgType=1）→ 机构详情 两级导航。
// 2026-04-24 重构：从原 GovernanceSection.tsx 的 PRC 分支拆分独立。
import { useState } from 'react';
import { AdminListPage, AdminSetChangePage } from './admins_change';
import { InstitutionListView } from './InstitutionListView';
import { InstitutionDetailPage } from './InstitutionDetailPage';
import { ProposalDetailPage } from './ProposalDetailPage';
import { CreateDuoqianTransferPage } from '../transaction/duoqian-transfer/CreateProposalPage';
import { SweepProposalPage } from '../transaction/duoqian-transfer/SweepProposalPage';
import { ProtocolUpgradeProposalPage } from './runtime-upgrade';
import type { AdminWalletMatch } from './types';

type PrcView =
  | { page: 'list' }
  | { page: 'detail'; sfidNumber: string }
  | { page: 'admin-list'; sfidNumber: string; orgType: number }
  | { page: 'admin-set-change'; sfidNumber: string; orgType: number; sfidFullName: string; adminWallets: AdminWalletMatch[] }
  | { page: 'proposal-detail'; proposalId: number; adminWallets: AdminWalletMatch[]; sfidNumber?: string; originSfidNumber: string }
  | { page: 'create-proposal'; sfidNumber: string; orgType: number; sfidFullName: string; mainAccount: string; adminWallets: AdminWalletMatch[] }
  | { page: 'protocol-upgrade'; sfidNumber: string; adminWallets: AdminWalletMatch[] }
  | { page: 'propose-sweep'; sfidNumber: string; sfidFullName: string; adminWallets: AdminWalletMatch[] };

export function PrcSection() {
  const [view, setView] = useState<PrcView>({ page: 'list' });

  const backToList = () => setView({ page: 'list' });
  const backToDetail = (sfidNumber: string) => setView({ page: 'detail', sfidNumber });

  if (view.page === 'admin-list') {
    return (
      <AdminListPage
        sfidNumber={view.sfidNumber}
        accountRef={{ sfidNumber: view.sfidNumber, org: view.orgType }}
        onBack={() => backToDetail(view.sfidNumber)}
      />
    );
  }

  if (view.page === 'proposal-detail') {
    return (
      <ProposalDetailPage
        proposalId={view.proposalId}
        adminWallets={view.adminWallets}
        sfidNumber={view.sfidNumber}
        onBack={() => backToDetail(view.originSfidNumber)}
      />
    );
  }

  if (view.page === 'create-proposal') {
    return (
      <CreateDuoqianTransferPage
        sfidNumber={view.sfidNumber}
        orgType={view.orgType}
        sfidFullName={view.sfidFullName}
        mainAccount={view.mainAccount}
        adminWallets={view.adminWallets}
        onBack={() => backToDetail(view.sfidNumber)}
        onSuccess={() => backToDetail(view.sfidNumber)}
      />
    );
  }

  if (view.page === 'admin-set-change') {
    return (
      <AdminSetChangePage
        accountRef={{ sfidNumber: view.sfidNumber, org: view.orgType }}
        sfidFullName={view.sfidFullName}
        adminWallets={view.adminWallets}
        onBack={() => backToDetail(view.sfidNumber)}
        onSuccess={() => backToDetail(view.sfidNumber)}
      />
    );
  }

  if (view.page === 'protocol-upgrade') {
    return (
      <ProtocolUpgradeProposalPage
        adminWallets={view.adminWallets}
        onBack={() => backToDetail(view.sfidNumber)}
        onSuccess={() => backToDetail(view.sfidNumber)}
      />
    );
  }

  if (view.page === 'propose-sweep') {
    return (
      <SweepProposalPage
        sfidNumber={view.sfidNumber}
        sfidFullName={view.sfidFullName}
        adminWallets={view.adminWallets}
        onBack={() => backToDetail(view.sfidNumber)}
        onSuccess={() => backToDetail(view.sfidNumber)}
      />
    );
  }

  if (view.page === 'detail') {
    const sfidNumber = view.sfidNumber;
    return (
      <InstitutionDetailPage
        sfidNumber={sfidNumber}
        onBack={backToList}
        onOpenAdminList={(sid, orgType) => setView({ page: 'admin-list', sfidNumber: sid, orgType })}
        onSelectProposal={(proposalId, adminWallets, sid) =>
          setView({ page: 'proposal-detail', proposalId, adminWallets, sfidNumber: sid, originSfidNumber: sfidNumber })
        }
        onCreateProposal={(sid, orgType, name, mainAccount, aw) =>
          setView({ page: 'create-proposal', sfidNumber: sid, orgType, sfidFullName: name, mainAccount, adminWallets: aw })
        }
        onCreateAdminSetChange={(sid, orgType, name, aw) =>
          setView({ page: 'admin-set-change', sfidNumber: sid, orgType, sfidFullName: name, adminWallets: aw })
        }
        onCreateProtocolUpgrade={(aw) =>
          setView({ page: 'protocol-upgrade', sfidNumber, adminWallets: aw })
        }
        onCreateSweep={(sid, name, aw) =>
          setView({ page: 'propose-sweep', sfidNumber: sid, sfidFullName: name, adminWallets: aw })
        }
      />
    );
  }

  // 默认：省储会机构列表（orgTypeFilter=1）。
  return (
    <InstitutionListView
      orgTypeFilter={1}
      onSelect={(sfidNumber) => setView({ page: 'detail', sfidNumber })}
    />
  );
}
