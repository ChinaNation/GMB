import type { Env } from '../types';
import { HttpError, jsonResponse } from '../shared/http';

interface TurnstileResult {
  success?: boolean;
  hostname?: string;
  action?: string;
}

/** 原生 App WebView 使用的受控验证页，成功后只把单次 token 回传给 App。 */
export function turnstilePageRoute(env: Env): Response {
  const sitekey = env.TURNSTILE_SITEKEY;
  if (!sitekey) {
    throw new HttpError(503, 'turnstile_not_configured', '设备安全验证尚未配置');
  }
  const html = `<!doctype html>
<html lang="zh-CN"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<title>设备安全验证</title><script src="https://challenges.cloudflare.com/turnstile/v0/api.js" async defer></script>
<style>html,body{height:100%;margin:0;background:#fff;font-family:system-ui}main{height:100%;display:grid;place-items:center}</style></head>
<body><main><div class="cf-turnstile" data-sitekey="${escapeHtml(sitekey)}" data-action="device_bind" data-callback="done"></div></main>
<script>function done(token){if(window.Turnstile&&window.Turnstile.postMessage){window.Turnstile.postMessage(token)}else{window.parent.postMessage({type:'turnstile',token:token},'*')}}</script>
</body></html>`;
  return new Response(html, {
    headers: {
      'content-type': 'text/html; charset=utf-8',
      'cache-control': 'no-store',
      'content-security-policy': "default-src 'none'; script-src https://challenges.cloudflare.com 'unsafe-inline'; frame-src https://challenges.cloudflare.com; style-src 'unsafe-inline'; connect-src https://challenges.cloudflare.com"
    }
  });
}

export async function verifyTurnstile(
  request: Request,
  env: Env,
  token: unknown
): Promise<void> {
  if (!env.TURNSTILE_SECRET) {
    if (env.DEV_UPLOAD_PROXY === '1') return;
    throw new HttpError(503, 'turnstile_not_configured', '设备安全验证尚未配置');
  }
  if (typeof token !== 'string' || token.length < 20 || token.length > 2048) {
    throw new HttpError(403, 'turnstile_required', '请先完成设备安全验证');
  }
  const form = new FormData();
  form.set('secret', env.TURNSTILE_SECRET);
  form.set('response', token);
  const ip = request.headers.get('cf-connecting-ip');
  if (ip) form.set('remoteip', ip);
  const response = await fetch('https://challenges.cloudflare.com/turnstile/v0/siteverify', {
    method: 'POST',
    body: form
  });
  const result = await response.json<TurnstileResult>().catch(() => null);
  if (!response.ok || result?.success !== true || result.action !== 'device_bind') {
    throw new HttpError(403, 'turnstile_failed', '设备安全验证失败或已过期');
  }
}

export function turnstileConfigRoute(env: Env): Response {
  return jsonResponse({
    ok: true,
    enabled: Boolean(env.TURNSTILE_SITEKEY && env.TURNSTILE_SECRET),
    verify_path: '/v1/security/turnstile'
  });
}

function escapeHtml(value: string): string {
  return value.replace(/[&<>'"]/g, (char) => ({
    '&': '&amp;', '<': '&lt;', '>': '&gt;', "'": '&#39;', '"': '&quot;'
  }[char] ?? char));
}
