// CPMS 共享类型定义

export interface ApiResponse<T> {
  code: number;
  message: string;
  data: T | null;
}

export interface ApiError {
  code: number;
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
  full_name: string;
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
  qr4_payload: string;
  created_at: number;
  updated_at: number;
}

export interface QrPayload {
  ver: string;
  issuer_id: string;
  site_sfid: string;
  sign_key_id: string;
  archive_no: string;
  citizen_status: string;
  voting_eligible: boolean;
  issued_at: number;
  expire_at: number;
  qr_id: string;
  sig_alg: string;
  signature: string;
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
  site_sfid: string | null;
  province_name: string | null;
  city_name: string | null;
  institution_name: string | null;
  super_admin_bound_count: number;
  qr2_ready: boolean;
  qr2_payload: string | null;
  anon_cert_done: boolean;
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
