import { HttpError, jsonResponse } from '../shared/http';
import { nowMs } from '../shared/time';
import type { Env } from '../types';
import { resourceLimit } from '../limits/catalog';
import { fetchChainIdentityStateByCid } from '../chain/identity';

export interface ChatRelayPayload {
  type: 'gmb_chat_envelope_v2' | 'gmb_chat_signal_v1';
  sender_cid_number: string;
  recipient_cid_number: string;
  recipient_device_id: string | null;
  /// 仅由 Worker 在投递前按 finalized 注入；客户端不得提供或决定。
  recipient_binding_revision?: number;
  recipient_binding_account_id?: string;
  envelope_id?: string;
  envelope?: string;
  signal?: unknown;
}

interface ChatSocketAttachment {
  cid_number: string;
  binding_revision: number;
  account_id: string;
  device_id: string;
  connected_at: number;
}

const deviceTagPrefix = 'device:';

/**
 * CID 级瞬时 Chat 转发器；WebSocket 附件额外绑定 finalized 版本与当前授权账户。
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
    if (request.method === 'POST' && path === '/__close_stale') {
      const current = (await request.json()) as {
        binding_revision?: unknown;
        account_id?: unknown;
      };
      let closed = 0;
      for (const socket of this.state.getWebSockets()) {
        const attachment = readAttachment(socket);
        if (
          !attachment
          || attachment.binding_revision !== current.binding_revision
          || attachment.account_id !== current.account_id
        ) {
          socket.close(1008, 'cid_binding_changed');
          closed += 1;
        }
      }
      return jsonResponse({ ok: true, closed });
    }
    if (request.headers.get('upgrade')?.toLowerCase() !== 'websocket') {
      return jsonResponse({ ok: false, error_code: 'websocket_required', message: '请使用 WebSocket 连接' }, { status: 426 });
    }

    const cidNumber = request.headers.get('x-chat-cid-number');
    const bindingRevision = Number.parseInt(
      request.headers.get('x-chat-binding-revision') ?? '',
      10,
    );
    const accountId = request.headers.get('x-chat-account-id');
    const deviceId = request.headers.get('x-chat-device');
    if (
      !cidNumber
      || !Number.isSafeInteger(bindingRevision)
      || bindingRevision <= 0
      || !accountId
      || !deviceId
    ) {
      return jsonResponse({ ok: false, error_code: 'chat_connection_invalid', message: 'Chat 连接缺少设备身份' }, { status: 400 });
    }
    const maxSockets = resourceLimit('chat_device').max_count!;
    if (this.state.getWebSockets().length >= maxSockets) {
      return jsonResponse({ ok: false, error_code: 'chat_socket_limit_exceeded', message: 'Chat 连接数已达到上限' }, { status: 429 });
    }
    const pair = new WebSocketPair();
    const [client, server] = Object.values(pair) as [WebSocket, WebSocket];
    server.serializeAttachment({
      cid_number: cidNumber,
      binding_revision: bindingRevision,
      account_id: accountId,
      device_id: deviceId,
      connected_at: nowMs(),
    } satisfies ChatSocketAttachment);
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
      if (
        attachment?.cid_number !== payload.recipient_cid_number
        || attachment.binding_revision !== payload.recipient_binding_revision
        || attachment.account_id !== payload.recipient_binding_account_id
      ) {
        socket.close(1008, 'cid_binding_changed');
        continue;
      }
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
  // 收件前直读 finalized 当前绑定，旧账户 WebSocket 即使尚未被接管清理也收不到密文。
  const binding = await fetchChainIdentityStateByCid(
    env,
    payload.recipient_cid_number,
  );
  if (
    binding.cid_number !== payload.recipient_cid_number
    || binding.binding_revision <= 0
    || !binding.account_id
  ) {
    return 0;
  }
  const routedPayload: ChatRelayPayload = {
    ...payload,
    recipient_binding_revision: binding.binding_revision,
    recipient_binding_account_id: binding.account_id,
  };
  const namespace = requireChatRealtimeNamespace(env);
  const response = await namespace.getByName(payload.recipient_cid_number).fetch(
    new Request('https://chat.internal/__relay', {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify(routedPayload),
    }),
  );
  if (!response.ok) return 0;
  return ((await response.json()) as { sent?: number }).sent ?? 0;
}

/// 只关闭不属于 finalized 当前绑定三元组的连接；新账户同 CID 连接保持在线。
export async function closeStaleChatRealtime(
  env: Env,
  cidNumber: string,
  bindingRevision: number,
  accountId: string,
): Promise<number> {
  const namespace = env.CHAT_REALTIME;
  if (!namespace) return 0;
  const response = await namespace.getByName(cidNumber).fetch(
    new Request('https://chat.internal/__close_stale', {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({
        binding_revision: bindingRevision,
        account_id: accountId,
      }),
    }),
  );
  if (!response.ok) return 0;
  return ((await response.json()) as { closed?: number }).closed ?? 0;
}

/// 关闭某身份主键 cid_number 的实时信箱，仅供整身份注销使用。
///
/// 换绑时新旧账户共享同一 CID/DO，严禁调用本函数，否则会把新账户连接一并踢下线。
export async function closeChatRealtime(env: Env, cidNumber: string): Promise<number> {
  const namespace = env.CHAT_REALTIME;
  if (!namespace) return 0;
  const response = await namespace.getByName(cidNumber).fetch(
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
  if (
    value
    && typeof value === 'object'
    && typeof value.cid_number === 'string'
    && typeof value.binding_revision === 'number'
    && typeof value.account_id === 'string'
    && typeof value.device_id === 'string'
  ) {
    return value as ChatSocketAttachment;
  }
  return null;
}
