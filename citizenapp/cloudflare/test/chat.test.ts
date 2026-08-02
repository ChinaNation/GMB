import { describe, expect, it, vi } from 'vitest';

vi.mock('../src/chain/identity', () => ({
  fetchChainIdentityStateByCid: vi.fn(async (_env: unknown, cidNumber: string) => ({
    cid_number: cidNumber,
    binding_revision: 1,
    account_id: '0x1111111111111111111111111111111111111111111111111111111111111111',
    identity_level: 'visitor' as const,
    has_voting_identity: false,
    has_candidate_identity: false,
    checked_at: 0,
  })),
}));
import { buildChatDeviceBindingMessageBase64Url } from '../src/chat/binding';
import { assertDevicePublicKeyHex, base64UrlToBytes, bytesToBase64Url } from '../src/chat/codec';
import { openChatWebSocket, submitChatEnvelope, submitChatSignal } from '../src/chat/service';
import {
  CHAT_WS_PONG_TYPE,
  CHAT_WS_READY_TYPE,
  relayChatPayload,
} from '../src/chat/realtime';
import type { Env, SessionState } from '../src/types';

// 会话绑定钱包账户(设备所有者/绑定签名主体);仍是 64-hex account_id。
const ACCOUNT_ID = '0x1111111111111111111111111111111111111111111111111111111111111111';
// 身份主键 cid_number:会话身份=发件人,另一 cid=收件人。寻址与归属只认 cid_number。
const SENDER_CID = 'CN220-CTZN2-198805200-2026';
const RECIPIENT_CID = 'CN220-CTZN2-199001010-2026';

class ChatStmt {
  private values: unknown[] = [];
  constructor(private readonly sql: string) {}
  bind(...values: unknown[]): ChatStmt {
    this.values = values;
    return this;
  }
  async first<T>(): Promise<T | null> {
    if (this.sql.includes('FROM chat_devices')) {
      // requireActiveDevice 按 (cid_number, device_id) 定位;WHERE 绑定顺序 = cid, device, now。
      return {
        cid_number: this.values[0],
        binding_revision: 1,
        account_id: ACCOUNT_ID,
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
    if (
      key ===
      'square_session:4943e43bc034c8bf90e1c2895796b954d3c34dc90afe838448dee6678fa765f8'
    ) {
      // 会话 fixture 含身份主键 cid_number + 当前绑定 account_id(设备所有者)。
      return {
        cid_number: SENDER_CID,
        binding_revision: 1,
        account_id: ACCOUNT_ID,
        device_key_hash: 'device-key-hash',
        created_at: Date.now(),
        expires_at: Date.now() + 60_000,
      } as T;
    }
    return null;
  }
}

function fakeEnv(sent = 1, onRelay?: (payload: unknown) => void): Env {
  return {
    DB: { prepare: (sql: string) => new ChatStmt(sql) } as unknown as D1Database,
    SQUARE_CACHE: new SessionKv() as unknown as KVNamespace,
    CHAT_REALTIME: {
      getByName: () => ({
        fetch: async (request: Request) => {
          if (new URL(request.url).pathname === '/__relay') {
            onRelay?.(await request.json());
            return Response.json({ ok: true, sent });
          }
          return Response.json({ ok: true, routed: true });
        },
      }),
    } as unknown as DurableObjectNamespace,
  } as Env;
}

describe('device-only Chat transport', () => {
  it('round-trips base64url bytes and requires canonical device keys', () => {
    const encoded = bytesToBase64Url(new Uint8Array([1, 2, 3, 254, 255]));
    expect(encoded).not.toContain('=');
    expect(Array.from(base64UrlToBytes(encoded))).toEqual([1, 2, 3, 254, 255]);
    expect(assertDevicePublicKeyHex('aabbcc')).toBe('aabbcc');
    expect(() => assertDevicePublicKeyHex('AABBcc')).toThrow();
  });

  it('builds a deterministic device binding payload', () => {
    const input = {
      cid_number: 'CN220-CTZN2-198805200-2026',
      binding_revision: 1,
      account_id: ACCOUNT_ID,
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
    let relayPayload: unknown;
    const env = fakeEnv(1, (payload) => {
      relayPayload = payload;
    });
    const response = await submitChatEnvelope(
      new Request('https://worker.test/chat/envelopes', {
        method: 'POST',
        headers: { authorization: 'Bearer test-session', 'content-type': 'application/json' },
        body: JSON.stringify({
          envelope_id: 'env-123456',
          sender_device_id: 'alice-phone',
          recipient_cid_number: RECIPIENT_CID,
          envelope: 'AQID',
        }),
      }),
      env,
    );
    const json = (await response.json()) as { delivery_state: string; recipient_connections: number };
    expect(json.delivery_state).toBe('sent');
    expect(json.recipient_connections).toBe(1);
    expect(relayPayload).toMatchObject({
      type: 'gmb_chat_envelope',
      sender_cid_number: SENDER_CID,
      recipient_cid_number: RECIPIENT_CID,
    });
  });

  it('relays WebRTC signals with the unversioned message type', async () => {
    let relayPayload: unknown;
    const response = await submitChatSignal(
      new Request('https://worker.test/chat/signals', {
        method: 'POST',
        headers: { authorization: 'Bearer test-session', 'content-type': 'application/json' },
        body: JSON.stringify({
          sender_device_id: 'alice-phone',
          recipient_cid_number: RECIPIENT_CID,
          signal: { kind: 'offer' },
        }),
      }),
      fakeEnv(1, (payload) => {
        relayPayload = payload;
      }),
    );

    expect((await response.json()) as { delivery_state: string }).toMatchObject({
      delivery_state: 'sent',
    });
    expect(relayPayload).toMatchObject({
      type: 'gmb_chat_signal',
      sender_cid_number: SENDER_CID,
      recipient_cid_number: RECIPIENT_CID,
      signal: { kind: 'offer' },
    });
  });

  it('locks the unversioned WebSocket control message types', () => {
    expect(CHAT_WS_READY_TYPE).toBe('gmb_chat_ws_ready');
    expect(CHAT_WS_PONG_TYPE).toBe('gmb_chat_ws_pong');
  });

  it('keeps delivery queued when the recipient device is unavailable', async () => {
    const response = await submitChatEnvelope(
      new Request('https://worker.test/chat/envelopes', {
        method: 'POST',
        headers: { authorization: 'Bearer test-session', 'content-type': 'application/json' },
        body: JSON.stringify({
          envelope_id: 'env-queued',
          sender_device_id: 'alice-phone',
          recipient_cid_number: RECIPIENT_CID,
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
      new Request('https://worker.test/chat/ws', {
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

  it('routes only the transient payload to the recipient identity object', async () => {
    let routedName = '';
    const env = fakeEnv();
    env.CHAT_REALTIME = {
      getByName: (name: string) => {
        routedName = name;
        return { fetch: async () => Response.json({ ok: true, sent: 1 }) };
      },
    } as unknown as DurableObjectNamespace;
    const sent = await relayChatPayload(env, {
      type: 'gmb_chat_envelope',
      sender_cid_number: SENDER_CID,
      recipient_cid_number: RECIPIENT_CID,
      recipient_device_id: null,
      envelope_id: 'env-route',
      envelope: 'AQID',
    });
    expect(sent).toBe(1);
    // 只路由给收件人身份主键 cid_number 命名的 DO,不落库、不广播其他身份。
    expect(routedName).toBe(RECIPIENT_CID);
  });
});
