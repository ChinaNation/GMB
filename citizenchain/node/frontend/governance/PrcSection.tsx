// 省储会顶级 Section：列表（orgType=1）→ 机构详情 两级导航。
// 2026-04-24 重构：从原 GovernanceSection.tsx 的 PRC 分支拆分独立。
import { useState } from 'react';
import { AdminListPage } from './AdminListPage';
import { InstitutionListView } from './InstitutionListView';
import { InstitutionDetailPage } from './InstitutionDetailPage';
import { ProposalDetailPage } from './ProposalDetailPage';
import { CreateProposalPage } from './CreateProposalPage';
import { SweepProposalPage } from './SweepProposalPage';
import type { AdminWalletMatch } from './types';

type PrcView =
  | { page: 'list' }
  | { page: 'detail'; shenfenId: string }
  | { page: 'admin-list'; shenfenId: string }
  | { page: 'proposal-detail'; proposalId: number; adminWallets: AdminWalletMatch[]; shenfenId?: string; originShenfenId: string }
  | { page: 'create-proposal'; shenfenId: string; orgType: number; institutionName: string; mainAddress: string; adminWallets: AdminWalletMatch[] }
  | { page: 'propose-sweep'; shenfenId: string; institutionName: string; adminWallets: AdminWalletMatch[] };

export function PrcSection() {
  const [view, setView] = useState<PrcView>({ page: 'list' });

  const backToList = () => setView({ page: 'list' });
  const backToDetail = (shenfenId: string) => setView({ page: 'detail', shenfenId });

  if (view.page === 'admin-list') {
    return (
      <AdminListPage
        shenfenId={view.shenfenId}
        onBack={() => backToDetail(view.shenfenId)}
      />
    );
  }

  if (view.page === 'proposal-detail') {
    return (
      <ProposalDetailPage
        proposalId={view.proposalId}
        adminWallets={view.adminWallets}
        shenfenId={view.shenfenId}
        onBack={() => backToDetail(view.originShenfenId)}
      />
    );
  }

  if (view.page === 'create-proposal') {
    return (
      <CreateProposalPage
        shenfenId={view.shenfenId}
        orgType={view.orgType}
        institutionName={view.institutionName}
        mainAddress={view.mainAddress}
        adminWallets={view.adminWallets}
        onBack={() => backToDetail(view.shenfenId)}
        onSuccess={() => backToDetail(view.shenfenId)}
      />
    );
  }

  if (view.page === 'propose-sweep') {
    return (
      <SweepProposalPage
        shenfenId={view.shenfenId}
        institutionName={view.institutionName}
        adminWallets={view.adminWallets}
        onBack={() => backToDetail(view.shenfenId)}
        onSuccess={() => backToDetail(view.shenfenId)}
      />
    );
  }

  if (view.page === 'detail') {
    const shenfenId = view.shenfenId;
    return (
      <InstitutionDetailPage
        shenfenId={shenfenId}
        onBack={backToList}
        onOpenAdminList={() => setView({ page: 'admin-list', shenfenId })}
        onSelectProposal={(proposalId, adminWallets, sid) =>
          setView({ page: 'proposal-detail', proposalId, adminWallets, shenfenId: sid, originShenfenId: shenfenId })
        }
        onCreateProposal={(sid, orgType, name, mainAddress, aw) =>
          setView({ page: 'create-proposal', shenfenId: sid, orgType, institutionName: name, mainAddress, adminWallets: aw })
        }
        onCreateSweep={(sid, name, aw) =>
          setView({ page: 'propose-sweep', shenfenId: sid, institutionName: name, adminWallets: aw })
        }
      />
    );
  }

  // 默认：省储会机构列表（orgTypeFilter=1）。
  return (
    <InstitutionListView
      orgTypeFilter={1}
      onSelect={(shenfenId) => setView({ page: 'detail', shenfenId })}
    />
  );
}
