// CPMS 管理员模块类型。

export type AdminUserGroup = 'admins' | 'operators';

export interface AdminUser {
  user_id: string;
  admin_account: string;
  admin_display_name: string;
  user_group: AdminUserGroup;
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
  citizen_binding_records_count: number;
  binding_release_records_count: number;
  records_hash: string;
  citizen_binding_records: Array<{
    archive_no: string;
    wallet_address: string;
    wallet_pubkey: string;
    wallet_sig_alg: 'sr25519';
    wallet_bound_at: number;
    citizen_status: string;
    voting_eligible: boolean;
    status_updated_at: number;
  }>;
  binding_release_records: Array<{
    archive_no: string;
    released_at: number;
    release_reason: 'ARCHIVE_HARD_DELETED_AFTER_100_YEARS';
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
