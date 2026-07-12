import { describe, expect, it } from 'vitest';
import { buildChatDeviceBindingMessageBase64Url } from '../src/chat/binding';
import { assertDevicePublicKeyHex, base64UrlToBytes, bytesToBase64Url } from '../src/chat/codec';
import { openChatWebSocket, submitChatEnvelope } from '../src/chat/service';
import { relayChatPayload } from '../src/chat/realtime';
import type { Env, SessionState } from '../src/types';

const OWNER = '5GrwvaEF5zXb26Fz9rcQpDWS7u4m6DXb6T6TQvF9j5uQ8g6U';
const RECIPIENT = '5FHneW46xGXgs5mUiveU4sbTyGBzmstLr6nCMvQNoHGKutQY';

class ChatStmt {
  private values: unknown[] = [];
  constructor(private readonly sql: string) {}
  bind(...values: unknown[]): ChatStmt {
    this.values = values;
    return this;
  }
  async first<T>(): Promise<T | null> {
    if (this.sql.includes('FROM chat_devices')) {
      return {
        owner_account: this.values[0],
        device_id: this.values[1],
        device_public_key_hex: 'aabbcc',
        expires_at: Date.now() + 60_000,
      } as T;
    }
    return null;
  }
  async all<T>(): Promise<{ results: T[] }> {
    return { results: [] };
  }
  async run(): Promise<{ meta: { changes: number } }> {
    return { meta: { changes: 1 } };
  }
}

class SessionKv {
  async get<T>(key: string): Promise<T | null> {
    if (key === 'square_session:test-session') {
      return {
        owner_account: OWNER,
        created_at: Date.now(),
        expires_at: Date.now() + 60_000,
      } as T;
    }
    return null;
  }
}

function fakeEnv(sent = 1): Env {
  return {
    DB: { prepare: (sql: string) => new ChatStmt(sql) } as unknown as D1Database,
    SQUARE_CACHE: new SessionKv() as unknown as KVNamespace,
    CHAT_REALTIME: {
      getByName: () => ({
        fetch: async (request: Request) => {
          if (new URL(request.url).pathname === '/__relay') {
            return Response.json({ ok: true, sent });
          }
          return Response.json({ ok: true, routed: true });
        },
      }),
    } as unknown as DurableObjectNamespace,
  } as Env;
}

describe('device-only Chat transport', () => {
  it('round-trips base64url bytes and normalizes device keys', () => {
    const encoded = bytesToBase64Url(new Uint8Array([1, 2, 3, 254, 255]));
    expect(encoded).not.toContain('=');
    expect(Array.from(base64UrlToBytes(encoded))).toEqual([1, 2, 3, 254, 255]);
    expect(assertDevicePublicKeyHex('AABBcc')).toBe('aabbcc');
  });

  it('builds a deterministic device binding payload', () => {
    const input = {
      owner_account: OWNER,
      device_id: 'alice-phone',
      device_public_key_hex: 'aabbcc',
      expires_at: 1_800_000,
      nonce: 'nonce-123456',
    };
    expect(buildChatDeviceBindingMessageBase64Url(input)).toBe(
      buildChatDeviceBindingMessageBase64Url(input),
    );
  });

  it('relays encrypted envelopes without a storage write', async () => {
    const env = fakeEnv(1);
    const response = await submitChatEnvelope(
      new Request('https://worker.test/v1/chat/envelopes', {
        method: 'POST',
        headers: { authorization: 'Bearer test-session', 'content-type': 'application/json' },
        body: JSON.stringify({
          envelope_id: 'env-123456',
          sender_device_id: 'alice-phone',
          recipient_account: RECIPIENT,
          envelope: 'AQID',
        }),
      }),
      env,
    );
    const json = (await response.json()) as { delivery_state: string; recipient_connections: number };
    expect(json.delivery_state).toBe('sent');
    expect(json.recipient_connections).toBe(1);
  });

  it('keeps delivery queued when the recipient device is unavailable', async () => {
    const response = await submitChatEnvelope(
      new Request('https://worker.test/v1/chat/envelopes', {
        method: 'POST',
        headers: { authorization: 'Bearer test-session', 'content-type': 'application/json' },
        body: JSON.stringify({
          envelope_id: 'env-queued',
          sender_device_id: 'alice-phone',
          recipient_account: RECIPIENT,
          envelope: 'AQID',
        }),
      }),
      fakeEnv(0),
    );
    const json = (await response.json()) as { delivery_state: string };
    expect(json.delivery_state).toBe('queued');
  });

  it('routes websocket connections from the verified session and device header', async () => {
    const response = await openChatWebSocket(
      new Request('https://worker.test/v1/chat/ws', {
        headers: {
          authorization: 'Bearer test-session',
          upgrade: 'websocket',
          'x-chat-device': 'alice-phone',
        },
      }),
      fakeEnv(),
    );
    expect((await response.json()) as { routed: boolean }).toMatchObject({ routed: true });
  });

  it('routes only the transient payload to the recipient account object', async () => {
    let routedName = '';
    const env = fakeEnv();
    env.CHAT_REALTIME = {
      getByName: (name: string) => {
        routedName = name;
        return { fetch: async () => Response.json({ ok: true, sent: 1 }) };
      },
    } as unknown as DurableObjectNamespace;
    const sent = await relayChatPayload(env, {
      type: 'gmb_chat_envelope_v2',
      sender_account: OWNER,
      recipient_account: RECIPIENT,
      recipient_device_id: null,
      envelope_id: 'env-route',
      envelope: 'AQID',
    });
    expect(sent).toBe(1);
    expect(routedName).toBe(RECIPIENT);
  });
});
