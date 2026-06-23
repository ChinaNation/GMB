import type {
  ActivateRequestResult,
  ActivatedAdmin,
  InstitutionDetail,
  VoteSignRequestResult,
  VoteSubmitResult,
} from '../types';

export type {
  ActivateRequestResult,
  ActivatedAdmin,
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
  admins: string[];
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
