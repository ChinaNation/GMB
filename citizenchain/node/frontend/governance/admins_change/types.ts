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
  sfidNumber: string | null;
  org: number;
  orgLabel: string;
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
  sfidNumber?: string | null;
  accountHex?: string | null;
  org?: number | null;
};
