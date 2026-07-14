import type {
  ActivateRequestResult,
  ActivatedAdmin,
  InstitutionAdminInfo,
  InstitutionRoleAssignmentInfo,
  InstitutionDetail,
} from '../governance/types';

export type {
  ActivateRequestResult,
  ActivatedAdmin,
  InstitutionAdminInfo,
  InstitutionRoleAssignmentInfo,
  InstitutionDetail,
};

export type AdminAccountState = {
  accountHex: string;
  cidNumber: string | null;
  institutionCode: number[];
  institutionCodeLabel: string;
  kind: number;
  kindLabel: string;
  admins: InstitutionAdminInfo[];
  status: number;
  statusLabel: string;
};

export type AdminAccountRef = {
  cidNumber?: string | null;
  accountHex?: string | null;
  institutionCode?: string | null;
};
