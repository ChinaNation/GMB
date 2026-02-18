import { invoke } from '@tauri-apps/api/core';

export async function generateOfflineQrDataUrl(payload: string): Promise<string | null> {
  try {
    const dataUrl = await invoke<string>('generate_qr_data_url', { payload });
    return dataUrl;
  } catch {
    return null;
  }
}
