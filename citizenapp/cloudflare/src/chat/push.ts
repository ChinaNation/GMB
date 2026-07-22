import type { Env } from '../types';
import { nowMs } from '../shared/time';
import { assertDeliverySize } from '../limits/delivery';

type PushProvider = 'apns' | 'fcm';

interface PushDeviceRow {
  push_provider: PushProvider;
  push_token: string;
}

interface WakePayload {
  kind: 'chat_wake';
  sender_account: string;
}

/**
 * 发送无聊天内容的设备唤醒通知。
 *
 * Cloudflare 不保存待通知任务；未送达密文继续留在发送设备本地队列。推送载荷只
 * 告知“哪个账户有待发送数据”，不得加入消息文字、会话编号、附件或文件名。
 */
export async function sendChatWake(
  env: Env,
  recipientAccount: string,
  senderAccount: string,
): Promise<number> {
  const result = await env.DB.prepare(
    `SELECT push_provider, push_token
      FROM chat_devices
      WHERE owner_account = ? AND expires_at > ?`,
  )
    .bind(recipientAccount, nowMs())
    .all<PushDeviceRow>();
  const payload: WakePayload = {
    kind: 'chat_wake',
    sender_account: senderAccount,
  };
  assertDeliverySize('push_wake', JSON.stringify(payload));
  const outcomes = await Promise.all(
    (result.results ?? []).map((device) => sendDeviceWake(env, device, payload)),
  );
  return outcomes.filter(Boolean).length;
}

async function sendDeviceWake(
  env: Env,
  device: PushDeviceRow,
  payload: WakePayload,
): Promise<boolean> {
  if (device.push_provider === 'apns') {
    return sendApnsWake(env, device.push_token, payload);
  }
  return sendFcmWake(env, device.push_token, payload);
}

/// 单台设备的推送目标（provider + token）；广场扇出按批查出后逐台发送。
export interface PushDevice {
  push_provider: PushProvider;
  push_token: string;
}

interface SquarePostAlert {
  title: string;
  body: string;
  post_id: string;
}

/// 广场发帖可见推送：与 chat_wake 静默唤醒共用 APNS-JWT / FCM-OAuth 传输，仅 payload
/// 为可见通知（横幅+声音）。帖子是公开内容，含作者名；data.kind='square_post' 供客户端
/// 点击导航，绝不触发聊天重连（客户端 chat_wake 判定要求恰好 2 字段，此处天然不匹配）。
export async function sendSquarePostAlert(
  env: Env,
  device: PushDevice,
  alert: SquarePostAlert,
): Promise<boolean> {
  if (device.push_provider === 'apns') {
    return sendApnsAlert(env, device.push_token, alert);
  }
  return sendFcmAlert(env, device.push_token, alert);
}

async function sendApnsAlert(
  env: Env,
  token: string,
  alert: SquarePostAlert,
): Promise<boolean> {
  if (!env.APNS_KEY || !env.APNS_KID || !env.APNS_TEAM || !env.APNS_TOPIC) {
    return false;
  }
  const jwt = await createApnsJwt(env);
  const host = env.APNS_ENV === 'sandbox' ? 'api.sandbox.push.apple.com' : 'api.push.apple.com';
  const response = await fetch(`https://${host}/3/device/${encodeURIComponent(token)}`, {
    method: 'POST',
    headers: {
      authorization: `bearer ${jwt}`,
      'apns-push-type': 'alert',
      'apns-priority': '10',
      'apns-topic': env.APNS_TOPIC,
      'content-type': 'application/json',
    },
    body: JSON.stringify({
      aps: { alert: { title: alert.title, body: alert.body }, sound: 'default' },
      kind: 'square_post',
      post_id: alert.post_id,
    }),
  });
  return response.ok;
}

async function sendFcmAlert(
  env: Env,
  token: string,
  alert: SquarePostAlert,
): Promise<boolean> {
  if (!env.FCM_PROJECT || !env.FCM_EMAIL || !env.FCM_KEY) {
    return false;
  }
  const accessToken = await createFcmAccessToken(env);
  const response = await fetch(
    `https://fcm.googleapis.com/v1/projects/${encodeURIComponent(env.FCM_PROJECT)}/messages:send`,
    {
      method: 'POST',
      headers: {
        authorization: `Bearer ${accessToken}`,
        'content-type': 'application/json',
      },
      body: JSON.stringify({
        message: {
          token,
          notification: { title: alert.title, body: alert.body },
          data: { kind: 'square_post', post_id: alert.post_id },
          // Android 8+ 用 App 侧预建的高优先级渠道 'square_posts' 保证横幅+声音；
          // channel_id 必须与 MainActivity 创建的渠道一致，否则声音由系统默认渠道决定。
          android: {
            priority: 'high',
            notification: { sound: 'default', channel_id: 'square_posts' },
          },
        },
      }),
    },
  );
  return response.ok;
}

async function sendApnsWake(
  env: Env,
  token: string,
  payload: WakePayload,
): Promise<boolean> {
  if (!env.APNS_KEY || !env.APNS_KID || !env.APNS_TEAM || !env.APNS_TOPIC) {
    return false;
  }
  const jwt = await createApnsJwt(env);
  const host = env.APNS_ENV === 'sandbox' ? 'api.sandbox.push.apple.com' : 'api.push.apple.com';
  const response = await fetch(`https://${host}/3/device/${encodeURIComponent(token)}`, {
    method: 'POST',
    headers: {
      authorization: `bearer ${jwt}`,
      'apns-push-type': 'background',
      'apns-priority': '5',
      'apns-topic': env.APNS_TOPIC,
      'content-type': 'application/json',
    },
    body: JSON.stringify({
      aps: { 'content-available': 1 },
      ...payload,
    }),
  });
  return response.ok;
}

async function createApnsJwt(env: Env): Promise<string> {
  const header = encodeJson({ alg: 'ES256', kid: env.APNS_KID });
  const claims = encodeJson({ iss: env.APNS_TEAM, iat: Math.floor(Date.now() / 1000) });
  const signingInput = `${header}.${claims}`;
  const key = await crypto.subtle.importKey(
    'pkcs8',
    pemBytes(env.APNS_KEY!),
    { name: 'ECDSA', namedCurve: 'P-256' },
    false,
    ['sign'],
  );
  const signature = await crypto.subtle.sign(
    { name: 'ECDSA', hash: 'SHA-256' },
    key,
    new TextEncoder().encode(signingInput),
  );
  return `${signingInput}.${base64Url(new Uint8Array(signature))}`;
}

async function sendFcmWake(
  env: Env,
  token: string,
  payload: WakePayload,
): Promise<boolean> {
  if (!env.FCM_PROJECT || !env.FCM_EMAIL || !env.FCM_KEY) {
    return false;
  }
  const accessToken = await createFcmAccessToken(env);
  const response = await fetch(
    `https://fcm.googleapis.com/v1/projects/${encodeURIComponent(env.FCM_PROJECT)}/messages:send`,
    {
      method: 'POST',
      headers: {
        authorization: `Bearer ${accessToken}`,
        'content-type': 'application/json',
      },
      body: JSON.stringify({
        message: {
          token,
          data: payload,
          android: { priority: 'high', ttl: '300s' },
        },
      }),
    },
  );
  return response.ok;
}

async function createFcmAccessToken(env: Env): Promise<string> {
  const now = Math.floor(Date.now() / 1000);
  const assertionHeader = encodeJson({ alg: 'RS256', typ: 'JWT' });
  const assertionClaims = encodeJson({
    iss: env.FCM_EMAIL,
    scope: 'https://www.googleapis.com/auth/firebase.messaging',
    aud: 'https://oauth2.googleapis.com/token',
    iat: now,
    exp: now + 3600,
  });
  const signingInput = `${assertionHeader}.${assertionClaims}`;
  const key = await crypto.subtle.importKey(
    'pkcs8',
    pemBytes(env.FCM_KEY!),
    { name: 'RSASSA-PKCS1-v1_5', hash: 'SHA-256' },
    false,
    ['sign'],
  );
  const signature = await crypto.subtle.sign(
    'RSASSA-PKCS1-v1_5',
    key,
    new TextEncoder().encode(signingInput),
  );
  const assertion = `${signingInput}.${base64Url(new Uint8Array(signature))}`;
  const body = new URLSearchParams({
    grant_type: 'urn:ietf:params:oauth:grant-type:jwt-bearer',
    assertion,
  });
  const response = await fetch('https://oauth2.googleapis.com/token', {
    method: 'POST',
    headers: { 'content-type': 'application/x-www-form-urlencoded' },
    body,
  });
  if (!response.ok) {
    throw new Error('FCM OAuth token request failed');
  }
  const json = (await response.json()) as { access_token?: string };
  if (!json.access_token) {
    throw new Error('FCM OAuth response missing access token');
  }
  return json.access_token;
}

function pemBytes(value: string): ArrayBuffer {
  const body = value
    .replace(/-----BEGIN [^-]+-----/g, '')
    .replace(/-----END [^-]+-----/g, '')
    .replace(/\\n/g, '')
    .replace(/\s/g, '');
  const binary = atob(body);
  const bytes = new Uint8Array(binary.length);
  for (let index = 0; index < binary.length; index += 1) {
    bytes[index] = binary.charCodeAt(index);
  }
  return bytes.buffer;
}

function encodeJson(value: unknown): string {
  return base64Url(new TextEncoder().encode(JSON.stringify(value)));
}

function base64Url(bytes: Uint8Array): string {
  let binary = '';
  for (const byte of bytes) binary += String.fromCharCode(byte);
  return btoa(binary).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/g, '');
}
