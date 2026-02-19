import { invoke } from '@tauri-apps/api/core';

function toWebQrUrl(payload: string): string {
  return `https://api.qrserver.com/v1/create-qr-code/?size=220x220&data=${encodeURIComponent(payload)}`;
}

function blobToDataUrl(blob: Blob): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(String(reader.result));
    reader.onerror = () => reject(new Error('failed to read qr blob'));
    reader.readAsDataURL(blob);
  });
}

async function generateWebQrDataUrl(payload: string): Promise<string | null> {
  try {
    const response = await fetch(toWebQrUrl(payload), { method: 'GET' });
    if (!response.ok) {
      return null;
    }
    const blob = await response.blob();
    return await blobToDataUrl(blob);
  } catch {
    return null;
  }
}

export async function generateOfflineQrDataUrl(payload: string): Promise<string | null> {
  const browserQr = await generateWebQrDataUrl(payload);
  if (browserQr) {
    return browserQr;
  }

  try {
    const dataUrl = await invoke<string>('generate_qr_data_url', { payload });
    return dataUrl;
  } catch {
    return null;
  }
}
