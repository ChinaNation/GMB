// 机构工作台类型。字段名与后端 workspace DTO 保持 snake_case。

export type WorkspaceKind = 'registry' | 'judicial' | 'legislation' | 'generic';

export type WorkspaceSectionKind = 'operations' | 'display' | 'records';

export type WorkspaceAction =
  | 'register_citizen'
  | 'register_institution'
  | 'register_private'
  | 'register_education'
  | 'manage_registry_admins'
  | 'manage_own_admins'
  | 'view_own_admins'
  | 'view_legislation'
  | 'propose_legislation'
  | 'cast_house_vote'
  | 'sign_legislation'
  | 'view_institution_profile'
  | 'view_operation_records';

export type WorkspaceActionItem = {
  action: WorkspaceAction;
  title: string;
  enabled: boolean;
};

export type WorkspaceSection = {
  workspace_section: WorkspaceSectionKind;
  workspace_section_title: string;
  workspace_actions: WorkspaceActionItem[];
};

export type InstitutionWorkspace = {
  workspace_kind: WorkspaceKind;
  workspace_title: string;
  workspace_sections: WorkspaceSection[];
};

