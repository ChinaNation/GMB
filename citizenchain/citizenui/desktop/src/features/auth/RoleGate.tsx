import { Alert } from 'antd';
import type { ReactNode } from 'react';
import { useAuthStore } from '../../stores/auth';
import type { UserRole } from '../../types/auth';

type RoleGateProps = {
  role: UserRole;
  children: ReactNode;
};

export function RoleGate({ role, children }: RoleGateProps) {
  const session = useAuthStore((state) => state.session);

  if (!session) {
    return <Alert type="warning" message="未登录，请先在上方使用管理员公钥登录。" showIcon />;
  }

  if (session.role !== role) {
    return <Alert type="error" message={`当前角色无权访问此工作台，请切换为 ${role.toUpperCase()}。`} showIcon />;
  }

  return <>{children}</>;
}
