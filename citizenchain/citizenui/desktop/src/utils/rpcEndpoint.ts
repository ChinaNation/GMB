const LOCAL_RPC_HOSTS = new Set(['127.0.0.1', 'localhost']);

function isValidPort(port: string): boolean {
  if (!/^\d+$/.test(port)) {
    return false;
  }
  const value = Number(port);
  return Number.isInteger(value) && value >= 1 && value <= 65535;
}

export function normalizeRpcEndpoint(input: string): string {
  return input.trim().replace(/\/+$/, '');
}

export function isSafeLocalRpcEndpoint(input: string): boolean {
  const normalized = normalizeRpcEndpoint(input);
  if (!normalized) {
    return false;
  }
  try {
    const url = new URL(normalized);
    if (url.protocol !== 'ws:') {
      return false;
    }
    if (!LOCAL_RPC_HOSTS.has(url.hostname)) {
      return false;
    }
    if (!isValidPort(url.port)) {
      return false;
    }
    if (url.username || url.password) {
      return false;
    }
    if (url.search || url.hash) {
      return false;
    }
    if (url.pathname !== '/' && url.pathname !== '') {
      return false;
    }
    return true;
  } catch {
    return false;
  }
}

export function assertSafeLocalRpcEndpoint(input: string): string {
  const normalized = normalizeRpcEndpoint(input);
  if (!isSafeLocalRpcEndpoint(normalized)) {
    throw new Error('RPC 地址仅允许 ws://127.0.0.1:<端口> 或 ws://localhost:<端口>');
  }
  return normalized;
}
