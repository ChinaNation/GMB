// CPMS 超级管理员模块类型。

export interface AdminUser {
  user_id: string;
  admin_pubkey: string;
  admin_name: string;
  role: string;
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
  number_release_records_count: number;
  records_hash: string;
  status_records: Array<{
    archive_no: string;
    citizen_status: string;
    voting_eligible: boolean;
    status_updated_at: number;
  }>;
  number_release_records: Array<{
    archive_no: string;
    passport_no: string;
    hard_deleted_at: number;
  }>;
  sig: string;
}
