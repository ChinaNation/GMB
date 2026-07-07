import { HttpError, jsonResponse } from "../shared/http";
import { nowMs } from "../shared/time";
import type { Env } from "../types";
import { assertChatAccount, assertDeviceId } from "./codec";

export interface ChatNoticePayload {
  type: "gmb_im_new_envelope_v1";
  envelope_id: string;
  conversation_id: string;
  recipient_account: string;
  recipient_device_id: string | null;
  mls_message_kind: string;
  created_at: number;
}

interface ChatSocketAttachment {
  owner_account: string;
  device_id: string;
  connected_at: number;
}

const deviceTagPrefix = "device:";

/// 账户级实时通知 Durable Object。
///
/// 一个钱包聊天账户对应一个对象，所有在线设备 WebSocket 都挂在同一个
/// 对象里。Worker 写入 D1 mailbox 后只调用本对象广播“有新密文”通知，
/// 不推送明文，也不推送密文正文。
export class ChatRealtimeObject implements DurableObject {
  constructor(
    private readonly state: DurableObjectState,
    private readonly env: Env,
  ) {
    void this.env;
  }

  async fetch(request: Request): Promise<Response> {
    // 内部推送入口：Worker 写入 D1 后经 stub.fetch(/__notify) 触发广播。
    // 走 fetch 而非自定义方法 RPC，避免依赖 DurableObject RPC 基类。
    if (
      request.method === "POST" &&
      new URL(request.url).pathname === "/__notify"
    ) {
      const payload = (await request.json()) as ChatNoticePayload;
      const sent = await this.deliver(payload);
      return jsonResponse({ ok: true, sent });
    }

    if (request.headers.get("upgrade")?.toLowerCase() !== "websocket") {
      return jsonResponse(
        {
          ok: false,
          error_code: "websocket_required",
          message: "请使用 WebSocket 连接",
        },
        { status: 426 },
      );
    }

    const url = new URL(request.url);
    const ownerAccount = assertChatAccount(url.searchParams.get("owner_account"));
    const deviceId = assertDeviceId(url.searchParams.get("device_id"));
    const pair = new WebSocketPair();
    const [client, server] = Object.values(pair) as [WebSocket, WebSocket];
    server.serializeAttachment({
      owner_account: ownerAccount,
      device_id: deviceId,
      connected_at: nowMs(),
    } satisfies ChatSocketAttachment);
    this.state.acceptWebSocket(server, [deviceTag(deviceId)]);
    server.send(
      JSON.stringify({
        type: "gmb_im_ws_ready_v1",
        owner_account: ownerAccount,
        device_id: deviceId,
        server_time: nowMs(),
      }),
    );

    return new Response(null, {
      status: 101,
      webSocket: client,
    });
  }

  private async deliver(payload: ChatNoticePayload): Promise<number> {
    const sockets = payload.recipient_device_id
      ? this.state.getWebSockets(deviceTag(payload.recipient_device_id))
      : this.state.getWebSockets();
    const payloadText = JSON.stringify(payload);
    let sentCount = 0;

    for (const socket of sockets) {
      const attachment = readAttachment(socket);
      if (attachment?.owner_account !== payload.recipient_account) {
        continue;
      }
      try {
        socket.send(payloadText);
        sentCount += 1;
      } catch (_) {
        socket.close(1011, "send_failed");
      }
    }

    return sentCount;
  }

  async webSocketMessage(socket: WebSocket, message: string | ArrayBuffer) {
    if (message === "ping") {
      socket.send(JSON.stringify({ type: "gmb_im_ws_pong_v1" }));
    }
  }

  async webSocketClose(socket: WebSocket) {
    socket.close();
  }

  async webSocketError(socket: WebSocket) {
    socket.close(1011, "socket_error");
  }
}

export async function notifyChatRealtime(
  env: Env,
  payload: ChatNoticePayload,
): Promise<number> {
  const namespace = env.CHAT_REALTIME;
  if (!namespace) {
    return 0;
  }
  const stub = namespace.getByName(payload.recipient_account);
  const request = new Request("https://chat-realtime.internal/__notify", {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify(payload),
  });
  const response = await stub.fetch(request);
  if (!response.ok) {
    return 0;
  }
  const data = (await response.json()) as { sent?: number };
  return data.sent ?? 0;
}

export function requireChatRealtimeNamespace(
  env: Env,
): DurableObjectNamespace {
  const namespace = env.CHAT_REALTIME;
  if (!namespace) {
    throw new HttpError(
      503,
      "chat_realtime_unavailable",
      "聊天实时通知服务未配置",
    );
  }
  return namespace;
}

function deviceTag(deviceId: string): string {
  return `${deviceTagPrefix}${deviceId}`;
}

function readAttachment(socket: WebSocket): ChatSocketAttachment | null {
  const attachment = socket.deserializeAttachment();
  if (
    attachment &&
    typeof attachment === "object" &&
    typeof attachment.owner_account === "string" &&
    typeof attachment.device_id === "string"
  ) {
    return attachment as ChatSocketAttachment;
  }
  return null;
}
