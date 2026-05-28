// CPMS 共享类型定义

export interface ApiResponse<T> {
  code: number;
  message: string;
  data: T | null;
}

export interface ApiError {
  code: number;
  error_code: string;
  message: string;
  trace_id: string;
}

export interface SessionUser {
  user_id: string;
  role: string;
}

export interface AdminUser {
  user_id: string;
  admin_pubkey: string;
  admin_name: string;
  role: string;
  status: string;
}

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
  status: string;
  citizen_status: string;
  voting_eligible: boolean;
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

export interface QrPrintRecord {
  print_id: string;
  archive_id: string;
  archive_no: string;
  citizen_status: string;
  voting_eligible: boolean;
  printed_at: number;
}

export interface InstallStatus {
  initialized: boolean;
  sfid_number: string | null;
  province_code: string | null;
  city_code: string | null;
  province_name: string | null;
  city_name: string | null;
  super_admin_bound_count: number;
  archive_signing_ready: boolean;
  cpms_pubkey: string | null;
}

export interface ChallengeData {
  challenge_id: string;
  challenge_payload: string;
  nonce: string;
  expire_at: number;
}

export interface VerifyData {
  access_token: string;
  expires_in: number;
  user: SessionUser;
}
