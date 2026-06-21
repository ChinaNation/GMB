import { get, post } from '../common/http';
import type { AdminUser } from '../admins/types';
import type { InstallStatus } from './types';

export const installStatus = () => get<InstallStatus>('/api/v1/install/status');

export const installInitialize = (sfid_init_qr_content: string) =>
  post<{ sfid_number: string }>('/api/v1/install/initialize', { sfid_init_qr_content });

export const bindAdmin = (admin_account: string) =>
  post<AdminUser>('/api/v1/install/admins/bind', { admin_account });
