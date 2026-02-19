export type AdminAuth = {
  user: string;
  password: string;
};

function normalizeBaseUrl(raw?: string): string {
  const value = (raw ?? '').trim();
  if (!value) return 'http://127.0.0.1:8899';
  if (value.startsWith('http://') || value.startsWith('https://')) {
    return value.replace(/\/+$/, '');
  }
  return `http://${value.replace(/\/+$/, '')}`;
}

const BASE_URL = normalizeBaseUrl(import.meta.env.VITE_CIIC_API_BASE_URL);

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  let resp: Response;
  try {
    resp = await fetch(`${BASE_URL}${path}`, init);
  } catch (error) {
    const msg = error instanceof Error ? error.message : String(error);
    throw new Error(`无法连接服务器(${BASE_URL})：${msg}`);
  }

  const text = await resp.text();
  let body: any = null;
  try {
    body = text ? JSON.parse(text) : null;
  } catch {
    const snippet = text.slice(0, 120);
    throw new Error(
      `服务响应格式错误(${resp.status})：${snippet || 'empty body'}，请确认后端已重启到最新版本`
    );
  }

  if (!resp.ok || !body || body.code !== 0) {
    throw new Error(body?.message ?? `request failed (${resp.status})`);
  }
  return body.data as T;
}

function adminHeaders(auth: AdminAuth): HeadersInit {
  return {
    'x-admin-user': auth.user,
    'x-admin-password': auth.password
  };
}

export type QueryResult = {
  account_pubkey: string;
  found_pending: boolean;
  found_binding: boolean;
  archive_index?: string;
  ciic_code?: string;
};

export type BindConfirmResult = {
  account_pubkey: string;
  archive_index: string;
  ciic_code: string;
  status: string;
  message: string;
};

export type CitizenRow = {
  seq: number;
  account_pubkey: string;
  archive_index?: string;
  ciic_code?: string;
  is_bound: boolean;
};

export async function listCitizens(auth: AdminAuth, keyword?: string): Promise<CitizenRow[]> {
  const q = keyword ? `?keyword=${encodeURIComponent(keyword)}` : '';
  return request<CitizenRow[]>(`/api/v1/admin/citizens${q}`, {
    headers: adminHeaders(auth)
  });
}

export async function checkAdminAuth(auth: AdminAuth): Promise<{ ok: boolean; user: string }> {
  // Use existing stable admin endpoint as auth probe to avoid 404 when backend
  // doesn't expose /admin/auth/check yet.
  await request<QueryResult>(
    `/api/v1/admin/bind/query?account_pubkey=${encodeURIComponent('__auth_probe__')}`,
    {
      headers: adminHeaders(auth)
    }
  );
  return { ok: true, user: auth.user };
}

export async function confirmBind(
  auth: AdminAuth,
  payload: { account_pubkey: string; archive_index: string }
): Promise<BindConfirmResult> {
  return request<BindConfirmResult>('/api/v1/admin/bind/confirm', {
    method: 'POST',
    headers: {
      'content-type': 'application/json',
      ...adminHeaders(auth)
    },
    body: JSON.stringify(payload)
  });
}

export async function unbind(auth: AdminAuth, accountPubkey: string): Promise<string> {
  return request<string>('/api/v1/admin/bind/unbind', {
    method: 'POST',
    headers: {
      'content-type': 'application/json',
      ...adminHeaders(auth)
    },
    body: JSON.stringify({ account_pubkey: accountPubkey })
  });
}
