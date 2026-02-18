export type UserRole = 'nrc' | 'prc' | 'prb';

export type LoginSession = {
  role: UserRole;
  publicKey: string;
  province?: string;
  organizationName: string;
};
