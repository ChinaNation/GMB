// CPMS 初始化状态类型。

export interface InstallStatus {
  initialized: boolean;
  cid_number: string | null;
  province_code: string | null;
  city_code: string | null;
  province_name: string | null;
  city_name: string | null;
  admins_bound_count: number;
  archive_signing_ready: boolean;
  cpms_pubkey: string | null;
}
