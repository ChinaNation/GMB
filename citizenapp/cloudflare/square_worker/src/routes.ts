import type { Env, FeedKind } from "./types";
import { createLoginChallenge, createSession } from "./auth/service";
import {
  ackChatEnvelope,
  completeChatAttachmentUpload,
  consumeChatKeyPackage,
  devGetChatAttachmentObject,
  devPutChatAttachmentObject,
  fetchChatKeyPackages,
  fetchPendingChatEnvelopes,
  openChatWebSocket,
  prepareChatAttachmentDownload,
  prepareChatAttachmentUpload,
  publishChatKeyPackage,
  registerChatDevice,
  submitChatEnvelope,
} from "./chat/service";
import { feedRoute } from "./feeds/service";
import { followRoute, unfollowRoute } from "./feeds/follows";
import { membershipRoute } from "./membership/service";
import { reportRoute, signalRoute } from "./moderation/service";
import { confirmPostRoute } from "./posts/confirm";
import {
  completeUpload,
  devPutUploadObject,
  prepareUpload,
} from "./uploads/service";
import { HttpError, jsonResponse, optionsResponse } from "./shared/http";

export async function routeRequest(
  request: Request,
  env: Env,
): Promise<Response> {
  if (request.method === "OPTIONS") {
    return optionsResponse();
  }

  const url = new URL(request.url);
  const path = url.pathname;

  if (request.method === "GET" && path === "/health") {
    return jsonResponse({
      ok: true,
      service: "citizenapp-square-api",
      storage_backend: "cloudflare-r2",
      // 广场内容只进 R2，链上只记录发布索引和哈希。
      content_on_chain: false,
    });
  }

  if (request.method === "POST" && path === "/v1/square/auth/challenge") {
    return createLoginChallenge(request, env);
  }
  if (request.method === "POST" && path === "/v1/square/auth/session") {
    return createSession(request, env);
  }
  if (request.method === "GET" && path === "/v1/square/membership") {
    return membershipRoute(request, env);
  }
  if (request.method === "POST" && path === "/v1/square/uploads/prepare") {
    return prepareUpload(request, env);
  }
  if (request.method === "PUT" && path === "/v1/square/uploads/dev-put") {
    return devPutUploadObject(request, env);
  }
  if (request.method === "POST" && path === "/v1/square/uploads/complete") {
    return completeUpload(request, env);
  }
  if (request.method === "POST" && path === "/v1/square/posts/confirm") {
    return confirmPostRoute(request, env);
  }
  if (request.method === "GET" && path.startsWith("/v1/square/feed/")) {
    return feedRoute(request, env, parseFeedKind(path));
  }
  if (request.method === "POST" && path === "/v1/square/follows") {
    return followRoute(request, env);
  }
  if (request.method === "DELETE" && path.startsWith("/v1/square/follows/")) {
    return unfollowRoute(request, env);
  }
  if (request.method === "POST" && path === "/v1/square/signals") {
    return signalRoute(request, env);
  }
  if (request.method === "POST" && path === "/v1/square/reports") {
    return reportRoute(request, env);
  }
  if (request.method === "POST" && path === "/v1/chat/devices/register") {
    return registerChatDevice(request, env);
  }
  if (request.method === "POST" && path === "/v1/chat/keypackages") {
    return publishChatKeyPackage(request, env);
  }
  if (request.method === "POST" && path === "/v1/chat/keypackages/consume") {
    return consumeChatKeyPackage(request, env);
  }
  if (request.method === "GET" && path.startsWith("/v1/chat/keypackages/")) {
    return fetchChatKeyPackages(request, env);
  }
  if (request.method === "POST" && path === "/v1/chat/envelopes") {
    return submitChatEnvelope(request, env);
  }
  if (request.method === "GET" && path === "/v1/chat/ws") {
    return openChatWebSocket(request, env);
  }
  if (request.method === "GET" && path === "/v1/chat/envelopes/pending") {
    return fetchPendingChatEnvelopes(request, env);
  }
  if (request.method === "POST" && path === "/v1/chat/envelopes/ack") {
    return ackChatEnvelope(request, env);
  }
  if (request.method === "POST" && path === "/v1/chat/attachments/prepare") {
    return prepareChatAttachmentUpload(request, env);
  }
  if (request.method === "PUT" && path === "/v1/chat/attachments/dev-put") {
    return devPutChatAttachmentObject(request, env);
  }
  if (request.method === "POST" && path === "/v1/chat/attachments/complete") {
    return completeChatAttachmentUpload(request, env);
  }
  if (request.method === "POST" && path === "/v1/chat/attachments/download") {
    return prepareChatAttachmentDownload(request, env);
  }
  if (request.method === "GET" && path === "/v1/chat/attachments/dev-get") {
    return devGetChatAttachmentObject(request, env);
  }

  throw new HttpError(404, "route_not_found", "广场接口不存在");
}

function parseFeedKind(path: string): FeedKind {
  const feedKind = path.split("/").pop();
  if (
    feedKind === "recommended" ||
    feedKind === "following" ||
    feedKind === "campaign"
  ) {
    return feedKind;
  }
  throw new HttpError(404, "feed_not_found", "广场信息流不存在");
}
