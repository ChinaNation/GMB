import { get, post } from '../common/http';
import type { AdminUser } from '../super_admin/types';
import type { InstallStatus } from './types';

export const installStatus = () => get<InstallStatus>('/api/v1/install/status');

export const installInitialize = (sfid_init_qr_content: string) =>
  post<{ sfid_number: string }>('/api/v1/install/initialize', { sfid_init_qr_content });

export const bindSuperAdmin = (admin_pubkey: string) =>
  post<AdminUser>('/api/v1/install/super-admin/bind', { admin_pubkey });
