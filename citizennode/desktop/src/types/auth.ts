export type UserRole = 'nrc' | 'prc' | 'prb' | 'full';

export type LoginSession = {
  role: UserRole;
  publicKey: string;
  province?: string;
  organizationName: string;
};
