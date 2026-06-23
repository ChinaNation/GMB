// 省储行顶级 Section：列表（orgType=2）→ 机构详情 两级导航。
// 2026-04-24 重构：从原 GovernanceSection.tsx 的 PRB 分支拆分独立。
// 与 PrcSection 同构；唯一差异是 orgTypeFilter=2。省储行同样支持手续费划转提案。
import { useState } from 'react';
import { AdminListPage, AdminSetChangePage } from './admins_change';
import { InstitutionListView } from './InstitutionListView';
import { InstitutionDetailPage } from './InstitutionDetailPage';
import { ProposalDetailPage } from './ProposalDetailPage';
import { CreateDuoqianTransferPage } from '../transaction/duoqian-transfer/CreateProposalPage';
import { SweepProposalPage } from '../transaction/duoqian-transfer/SweepProposalPage';
import type { AdminWalletMatch } from './types';

type PrbView =
  | { page: 'list' }
  | { page: 'detail'; cidNumber: string }
  | { page: 'admin-list'; cidNumber: string; orgType: number }
  | { page: 'admin-set-change'; cidNumber: string; orgType: number; cidFullName: string; adminWallets: AdminWalletMatch[] }
  | { page: 'proposal-detail'; proposalId: number; adminWallets: AdminWalletMatch[]; cidNumber?: string; originCidNumber: string }
  | { page: 'create-proposal'; cidNumber: string; orgType: number; cidFullName: string; mainAccount: string; adminWallets: AdminWalletMatch[] }
  | { page: 'propose-sweep'; cidNumber: string; cidFullName: string; adminWallets: AdminWalletMatch[] };

export function PrbSection() {
  const [view, setView] = useState<PrbView>({ page: 'list' });

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
      <CreateDuoqianTransferPage
        cidNumber={view.cidNumber}
        institutionCode="PRB"
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
        onCreateProposal={(sid, orgType, name, mainAccount, aw) =>
          setView({ page: 'create-proposal', cidNumber: sid, orgType, cidFullName: name, mainAccount, adminWallets: aw })
        }
        onCreateAdminSetChange={(sid, orgType, name, aw) =>
          setView({ page: 'admin-set-change', cidNumber: sid, orgType, cidFullName: name, adminWallets: aw })
        }
        onCreateSweep={(sid, name, aw) =>
          setView({ page: 'propose-sweep', cidNumber: sid, cidFullName: name, adminWallets: aw })
        }
      />
    );
  }

  // 默认：省储行机构列表（orgTypeFilter=2）。
  return (
    <InstitutionListView
      orgTypeFilter={2}
      onSelect={(cidNumber) => setView({ page: 'detail', cidNumber })}
    />
  );
}
