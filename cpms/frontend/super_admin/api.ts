import { del, get, post } from '../common/http';
import type { AdminUser, CpmsStatusExportFile, CpmsStatusExportState } from './types';

export const listOperators = () => get<AdminUser[]>('/api/v1/admin/operators');

export const createOperator = (admin_pubkey: string, admin_name: string) =>
  post<AdminUser>('/api/v1/admin/operators', { admin_pubkey, admin_name });

export const deleteOperator = (id: string) => del<null>(`/api/v1/admin/operators/${id}`);

export const exportStatusFile = () =>
  get<{ file_name: string; export_file: CpmsStatusExportFile }>('/api/v1/archives/status-export');

export const getStatusExportState = () =>
  get<{ state: CpmsStatusExportState }>('/api/v1/archives/status-export/state');
