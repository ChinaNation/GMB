// CPMS 档案模块类型：档案创建、查询、更新、打印和删除签名共用。

export type ElectionScopeLevel = 'PROVINCE' | 'CITY' | 'TOWN';

export interface Archive {
  archive_id: string;
  archive_no: string;
  province_code: string;
  city_code: string;
  last_name: string;
  first_name: string;
  birth_date: string;
  gender_code: string;
  height_cm: number | null;
  passport_no: string;
  town_code: string;
  village_id: string;
  address: string;
  birth_province_code: string;
  birth_city_code: string;
  birth_town_code: string;
  election_scope_level: ElectionScopeLevel;
  status: string;
  citizen_status: string;
  voting_eligible: boolean;
  valid_from: string;
  valid_until: string;
  wallet_address: string | null;
  wallet_pubkey: string | null;
  wallet_sig_alg: string;
  wallet_bound_at: number | null;
  wallet_bound_by: string | null;
  archive_qr_payload: string;
  deleted_at: number | null;
  deleted_by: string | null;
  delete_reason: string | null;
  created_at: number;
  updated_at: number;
}

export interface CreateArchiveRequest {
  last_name: string;
  first_name: string;
  birth_date: string;
  gender_code: string;
  height_cm: number;
  town_code?: string;
  village_id?: string;
  address?: string;
  birth_province_code: string;
  birth_city_code: string;
  birth_town_code: string;
  election_scope_level: ElectionScopeLevel;
  citizen_status?: string;
  voting_eligible?: boolean;
}

export interface QrPrintRecord {
  print_id: string;
  archive_id: string;
  archive_no: string;
  citizen_status: string;
  voting_eligible: boolean;
  printed_at: number;
}

export type ArchiveMaterialType = 'PHOTO' | 'BIRTH_CERTIFICATE' | 'COPY' | 'VIDEO' | 'OTHER';

export interface ArchiveMaterial {
  material_id: string;
  archive_id: string;
  material_type: ArchiveMaterialType;
  original_file_name: string;
  mime_type: string;
  file_size: number;
  sha256: string;
  note: string;
  uploaded_by: string;
  uploaded_at: number;
}

export interface ArchiveAuditLog {
  log_id: string;
  operator_user_id: string | null;
  operator_account: string | null;
  action: string;
  target_type: string;
  target_id: string | null;
  result: 'SUCCESS' | 'FAILED';
  detail: Record<string, unknown>;
  created_at: number;
}
