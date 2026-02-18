import type { LoginSession } from '../types/auth';

export function getOrganizationName(session: LoginSession): string {
  return session.organizationName;
}
