import type {
  ActivateRequestResult,
  ActivatedAdmin,
  AdminProfileInfo,
  InstitutionDetail,
  VoteSignRequestResult,
  VoteSubmitResult,
} from '../../governance/types';

export type {
  ActivateRequestResult,
  ActivatedAdmin,
  AdminProfileInfo,
  InstitutionDetail,
  VoteSignRequestResult,
  VoteSubmitResult,
};

export type AdminAccountState = {
  accountHex: string;
  cidNumber: string | null;
  institutionCode: number[];
  institutionCodeLabel: string;
  kind: number;
  kindLabel: string;
  admins: AdminProfileInfo[];
  creatorHex: string;
  createdAt: number;
  updatedAt: number;
  status: number;
  statusLabel: string;
};

export type AdminAccountRef = {
  cidNumber?: string | null;
  accountHex?: string | null;
  institutionCode?: string | null;
};
