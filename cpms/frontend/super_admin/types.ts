// CPMS 超级管理员模块类型。

export type AdminRole = 'SUPER_ADMIN' | 'OPERATOR_ADMIN';

export interface AdminUser {
  user_id: string;
  admin_pubkey: string;
  admin_address: string;
  admin_name: string;
  role: AdminRole;
  immutable: boolean;
  can_edit_name: boolean;
  can_delete: boolean;
}

export interface CpmsStatusExportFile {
  proto: 'SFID_CPMS_V1';
  type: 'CPMS_STATUS_EXPORT';
  version: number;
  export_year: number;
  sfid_number: string;
  cpms_pubkey: string;
  export_batch_id: string;
  exported_at: number;
  status_records_count: number;
  archive_release_records_count: number;
  records_hash: string;
  status_records: Array<{
    archive_no: string;
    citizen_status: string;
    voting_eligible: boolean;
    status_updated_at: number;
  }>;
  archive_release_records: Array<{
    archive_no: string;
    released_at: number;
  }>;
  sig: string;
}

export interface CpmsStatusExportState {
  now_utc: number;
  pending_export_year: number | null;
  can_export: boolean;
  reminder_active: boolean;
  operator_lock_active: boolean;
  exported: boolean;
  next_export_available_at: number | null;
  disabled_reason: string | null;
}
