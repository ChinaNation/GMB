// 省储会顶级 Section：列表（orgType=1）→ 机构详情 两级导航。
// 2026-04-24 重构：从原 GovernanceSection.tsx 的 PRC 分支拆分独立。
import { useState } from 'react';
import { AdminListPage, AdminSetChangePage } from './admins_change';
import { InstitutionListView } from './InstitutionListView';
import { InstitutionDetailPage } from './InstitutionDetailPage';
import { ProposalDetailPage } from './ProposalDetailPage';
import { CreateProposalPage } from './CreateProposalPage';
import { SweepProposalPage } from './SweepProposalPage';
import type { AdminWalletMatch } from './types';

type PrcView =
  | { page: 'list' }
  | { page: 'detail'; sfidNumber: string }
  | { page: 'admin-list'; sfidNumber: string }
  | { page: 'admin-set-change'; sfidNumber: string; institutionName: string; adminWallets: AdminWalletMatch[] }
  | { page: 'proposal-detail'; proposalId: number; adminWallets: AdminWalletMatch[]; sfidNumber?: string; originSfidNumber: string }
  | { page: 'create-proposal'; sfidNumber: string; orgType: number; institutionName: string; mainAddress: string; adminWallets: AdminWalletMatch[] }
  | { page: 'propose-sweep'; sfidNumber: string; institutionName: string; adminWallets: AdminWalletMatch[] };

export function PrcSection() {
  const [view, setView] = useState<PrcView>({ page: 'list' });

  const backToList = () => setView({ page: 'list' });
  const backToDetail = (sfidNumber: string) => setView({ page: 'detail', sfidNumber });

  if (view.page === 'admin-list') {
    return (
      <AdminListPage
        sfidNumber={view.sfidNumber}
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
      <CreateProposalPage
        sfidNumber={view.sfidNumber}
        orgType={view.orgType}
        institutionName={view.institutionName}
        mainAddress={view.mainAddress}
        adminWallets={view.adminWallets}
        onBack={() => backToDetail(view.sfidNumber)}
        onSuccess={() => backToDetail(view.sfidNumber)}
      />
    );
  }

  if (view.page === 'admin-set-change') {
    return (
      <AdminSetChangePage
        sfidNumber={view.sfidNumber}
        institutionName={view.institutionName}
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
        institutionName={view.institutionName}
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
        onOpenAdminList={() => setView({ page: 'admin-list', sfidNumber })}
        onSelectProposal={(proposalId, adminWallets, sid) =>
          setView({ page: 'proposal-detail', proposalId, adminWallets, sfidNumber: sid, originSfidNumber: sfidNumber })
        }
        onCreateProposal={(sid, orgType, name, mainAddress, aw) =>
          setView({ page: 'create-proposal', sfidNumber: sid, orgType, institutionName: name, mainAddress, adminWallets: aw })
        }
        onCreateAdminSetChange={(sid, name, aw) =>
          setView({ page: 'admin-set-change', sfidNumber: sid, institutionName: name, adminWallets: aw })
        }
        onCreateSweep={(sid, name, aw) =>
          setView({ page: 'propose-sweep', sfidNumber: sid, institutionName: name, adminWallets: aw })
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
