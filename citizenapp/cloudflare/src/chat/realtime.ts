import { HttpError, jsonResponse } from '../shared/http';
import { nowMs } from '../shared/time';
import type { Env } from '../types';

export interface ChatRelayPayload {
  type: 'gmb_chat_envelope_v2' | 'gmb_chat_signal_v1';
  sender_account: string;
  recipient_account: string;
  recipient_device_id: string | null;
  envelope_id?: string;
  envelope?: string;
  signal?: unknown;
}

interface ChatSocketAttachment {
  owner_account: string;
  device_id: string;
  connected_at: number;
}

const deviceTagPrefix = 'device:';

/**
 * 账户级瞬时 Chat 转发器。
 *
 * Durable Object 只持有休眠 WebSocket 附件，不使用持久化 Storage。消息密文和
 * WebRTC 信令只在当前请求内转发；接收设备不可达时由发送设备本地队列负责重试。
 */
export class ChatRealtimeObject implements DurableObject {
  constructor(
    private readonly state: DurableObjectState,
    private readonly env: Env,
  ) {
    void this.env;
  }

  async fetch(request: Request): Promise<Response> {
    const path = new URL(request.url).pathname;
    if (request.method === 'POST' && path === '/__relay') {
      const payload = (await request.json()) as ChatRelayPayload;
      return jsonResponse({ ok: true, sent: this.deliver(payload) });
    }
    if (request.method === 'POST' && path === '/__close') {
      let closed = 0;
      for (const socket of this.state.getWebSockets()) {
        socket.close(1008, 'account_deleted');
        closed += 1;
      }
      return jsonResponse({ ok: true, closed });
    }
    if (request.headers.get('upgrade')?.toLowerCase() !== 'websocket') {
      return jsonResponse({ ok: false, error_code: 'websocket_required', message: '请使用 WebSocket 连接' }, { status: 426 });
    }

    const ownerAccount = request.headers.get('x-chat-owner');
    const deviceId = request.headers.get('x-chat-device');
    if (!ownerAccount || !deviceId) {
      return jsonResponse({ ok: false, error_code: 'chat_connection_invalid', message: 'Chat 连接缺少设备身份' }, { status: 400 });
    }
    const pair = new WebSocketPair();
    const [client, server] = Object.values(pair) as [WebSocket, WebSocket];
    server.serializeAttachment({ owner_account: ownerAccount, device_id: deviceId, connected_at: nowMs() } satisfies ChatSocketAttachment);
    this.state.acceptWebSocket(server, [deviceTag(deviceId)]);
    server.send(JSON.stringify({ type: 'gmb_chat_ws_ready_v2', server_time: nowMs() }));
    return new Response(null, { status: 101, webSocket: client });
  }

  private deliver(payload: ChatRelayPayload): number {
    const sockets = payload.recipient_device_id
      ? this.state.getWebSockets(deviceTag(payload.recipient_device_id))
      : this.state.getWebSockets();
    const text = JSON.stringify(payload);
    let sent = 0;
    for (const socket of sockets) {
      const attachment = readAttachment(socket);
      if (attachment?.owner_account !== payload.recipient_account) continue;
      try {
        socket.send(text);
        sent += 1;
      } catch {
        socket.close(1011, 'send_failed');
      }
    }
    return sent;
  }

  async webSocketMessage(socket: WebSocket, message: string | ArrayBuffer) {
    if (message === 'ping') socket.send(JSON.stringify({ type: 'gmb_chat_ws_pong_v2' }));
  }

  async webSocketClose(socket: WebSocket) {
    socket.close();
  }

  async webSocketError(socket: WebSocket) {
    socket.close(1011, 'socket_error');
  }
}

export async function relayChatPayload(env: Env, payload: ChatRelayPayload): Promise<number> {
  const namespace = requireChatRealtimeNamespace(env);
  const response = await namespace.getByName(payload.recipient_account).fetch(
    new Request('https://chat.internal/__relay', {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify(payload),
    }),
  );
  if (!response.ok) return 0;
  return ((await response.json()) as { sent?: number }).sent ?? 0;
}

export async function closeChatRealtime(env: Env, ownerAccount: string): Promise<number> {
  const namespace = env.CHAT_REALTIME;
  if (!namespace) return 0;
  const response = await namespace.getByName(ownerAccount).fetch(
    new Request('https://chat.internal/__close', { method: 'POST' }),
  );
  if (!response.ok) return 0;
  return ((await response.json()) as { closed?: number }).closed ?? 0;
}

export function requireChatRealtimeNamespace(env: Env): DurableObjectNamespace {
  if (!env.CHAT_REALTIME) {
    throw new HttpError(503, 'chat_realtime_unavailable', '聊天实时服务未配置');
  }
  return env.CHAT_REALTIME;
}

function deviceTag(deviceId: string): string {
  return `${deviceTagPrefix}${deviceId}`;
}

function readAttachment(socket: WebSocket): ChatSocketAttachment | null {
  const value = socket.deserializeAttachment();
  if (value && typeof value === 'object' && typeof value.owner_account === 'string' && typeof value.device_id === 'string') {
    return value as ChatSocketAttachment;
  }
  return null;
}
