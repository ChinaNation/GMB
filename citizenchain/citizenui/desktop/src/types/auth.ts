export type UserRole = 'nrc' | 'prc' | 'prb' | 'full';

export type LoginIdentity = {
  role: UserRole;
  publicKey: string;
  province?: string;
  organizationName: string;
};

export type LoginSession = LoginIdentity & {
  issuedAt: number;
  expiresAt: number;
};
