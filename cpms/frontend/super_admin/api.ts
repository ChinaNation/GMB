import { del, get, post, put } from '../common/http';
import type { AdminRole, AdminUser, CpmsStatusExportFile, CpmsStatusExportState } from './types';

export const listAdmins = () => get<AdminUser[]>('/api/v1/admin/admins');

export const createAdmin = (body: { role: AdminRole; admin_pubkey: string; admin_name: string }) =>
  post<AdminUser>('/api/v1/admin/admins', body);

export const updateAdminName = (id: string, admin_name: string) =>
  put<AdminUser>(`/api/v1/admin/admins/${id}`, { admin_name });

export const deleteAdmin = (id: string) => del<null>(`/api/v1/admin/admins/${id}`);

export const exportStatusFile = () =>
  get<{ file_name: string; export_file: CpmsStatusExportFile }>('/api/v1/archives/status-export');

export const getStatusExportState = () =>
  get<{ state: CpmsStatusExportState }>('/api/v1/archives/status-export/state');
