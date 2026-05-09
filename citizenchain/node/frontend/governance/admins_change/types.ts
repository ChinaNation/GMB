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

export type AdminSubjectState = {
  subjectIdHex: string;
  sfidNumber: string | null;
  org: number;
  orgLabel: string;
  kind: number;
  kindLabel: string;
  admins: string[];
  threshold: number;
  creatorHex: string;
  createdAt: number;
  updatedAt: number;
  status: number;
  statusLabel: string;
};

