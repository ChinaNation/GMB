// 机构工作台路由。登录机构决定进入注册局、司法院或通用机构工作台。

import type { AdminAuth } from '../auth/types';
import type { CapabilitySet } from '../auth/AuthContext';
import type { CidMetaResult } from '../china/api';
import { isSubordinateRegistry, isTier1Registry } from '../platform/registryTier';
import type { InstitutionWorkspace, WorkspaceKind } from './types';
import { GenericWorkspace } from './generic/GenericWorkspace';
import { JudicialWorkspace } from './judicial/JudicialWorkspace';
import { RegistryWorkspace } from './registry/RegistryWorkspace';

export type WorkspaceRouterProps = {
  auth: AdminAuth;
  capabilities: CapabilitySet;
  passkeyRegistered: boolean | null;
  cidMeta: CidMetaResult | null;
  setCidMeta: (next: CidMetaResult | null) => void;
};

function fallbackWorkspaceKind(auth: AdminAuth, capabilities: CapabilitySet): WorkspaceKind {
  if (isTier1Registry(auth.institution_code) || isSubordinateRegistry(auth.institution_code)) {
    return 'registry';
  }
  if (auth.institution_code === 'NJD') return 'judicial';
  if (capabilities.canViewLegislation) return 'legislation';
  return 'generic';
}

function fallbackWorkspace(auth: AdminAuth, capabilities: CapabilitySet): InstitutionWorkspace {
  const workspaceKind = fallbackWorkspaceKind(auth, capabilities);
  const workspaceTitle = `${auth.cid_short_name || auth.institution_code}工作台`;
  return {
    workspace_kind: workspaceKind,
    workspace_title: workspaceTitle,
    workspace_sections: [
      { workspace_section: 'operations', workspace_section_title: '操作', workspace_actions: [] },
      { workspace_section: 'display', workspace_section_title: '显示', workspace_actions: [] },
      { workspace_section: 'records', workspace_section_title: '记录', workspace_actions: [] },
    ],
  };
}

export function WorkspaceRouter({
  auth,
  capabilities,
  passkeyRegistered,
  cidMeta,
  setCidMeta,
}: WorkspaceRouterProps) {
  const authWithWorkspace: AdminAuth = {
    ...auth,
    workspace: auth.workspace ?? fallbackWorkspace(auth, capabilities),
  };
  const workspaceKind = authWithWorkspace.workspace?.workspace_kind ?? 'generic';

  if (workspaceKind === 'registry') {
    return (
      <RegistryWorkspace
        auth={authWithWorkspace}
        capabilities={capabilities}
        passkeyRegistered={passkeyRegistered}
        cidMeta={cidMeta}
        setCidMeta={setCidMeta}
      />
    );
  }
  if (workspaceKind === 'judicial') {
    return <JudicialWorkspace auth={authWithWorkspace} />;
  }
  return <GenericWorkspace auth={authWithWorkspace} />;
}

