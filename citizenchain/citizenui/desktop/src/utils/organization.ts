import type { UserRole } from '../types/auth';
import type { LoginSession } from '../types/auth';

const FULL_NODE_DISPLAY_NAME = 'SFID 本地管理员';

export function formatNodeDisplayName(input: { role: UserRole; organizationName: string }): string {
  if (input.role === 'full') {
    return FULL_NODE_DISPLAY_NAME;
  }

  return input.organizationName.endsWith('节点')
    ? input.organizationName
    : `${input.organizationName}节点`;
}

export function getOrganizationName(session: LoginSession): string {
  return formatNodeDisplayName({
    role: session.role,
    organizationName: session.organizationName
  });
}
