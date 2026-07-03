// 通用机构工作台。未落专属 UI 的机构先使用三段式通用壳。

import { Empty } from 'antd';
import type { AdminAuth } from '../../auth/types';
import { OwnInstitutionAdminsView } from '../../admins/RegistryAdminsView';
import { LegislationView } from '../../legislation/operator/LegislationView';
import { OwnInstitutionInfoPanel } from '../judicial/JudicialDisplay';
import { WorkspaceShell } from '../WorkspaceShell';

export type GenericWorkspaceProps = {
  auth: AdminAuth;
};

function GenericOperations({ auth }: GenericWorkspaceProps) {
  if (auth.capabilities?.canViewLegislation) {
    return <LegislationView auth={auth} />;
  }
  return <Empty description="暂无可执行操作" />;
}

function GenericDisplay({ auth }: GenericWorkspaceProps) {
  if (auth.capabilities?.canViewOwnAdmins) {
    return (
      <div style={{ display: 'flex', flexDirection: 'column', gap: 16 }}>
        <OwnInstitutionInfoPanel auth={auth} />
        <OwnInstitutionAdminsView layout="cards" />
      </div>
    );
  }
  return <Empty description="暂无可显示内容" />;
}

function GenericRecords() {
  return <Empty description="暂无记录" />;
}

export function GenericWorkspace({ auth }: GenericWorkspaceProps) {
  const workspace = auth.workspace;
  if (!workspace) return null;
  return (
    <WorkspaceShell
      workspace={workspace}
      operations={<GenericOperations auth={auth} />}
      display={<GenericDisplay auth={auth} />}
      records={<GenericRecords />}
    />
  );
}
