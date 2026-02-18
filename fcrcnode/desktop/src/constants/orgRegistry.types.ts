import type { UserRole } from '../types/auth';

export type OrganizationRegistryItem = {
  role: UserRole;
  organizationName: string;
  province?: string;
  adminAddress: string;
};
