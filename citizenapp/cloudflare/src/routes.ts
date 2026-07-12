import type { Env, FeedKind } from "./types";
import {
  cancelMembershipChallengeRoute,
  cancelMembershipRoute,
  deleteAccountChallengeRoute,
  deleteAccountRoute,
} from "./account/service";
import { createLoginChallenge, createSession, registerDeviceSubkey } from "./auth/service";
import { chainBootstrapRoute } from "./chain/bootstrap";
import { relaySignedExtrinsicRoute } from "./chain/extrinsic_relay";
import {
  consumeChatKeyPackage,
  fetchChatKeyPackages,
  openChatWebSocket,
  publishChatKeyPackage,
  registerChatDevice,
  submitChatEnvelope,
  submitChatSignal,
} from "./chat/service";
import { createTurnCredentials } from "./chat/turn";
import { feedRoute } from "./feeds/service";
import { followRoute, unfollowRoute } from "./feeds/follows";
import { mediaRoute } from "./media/service";
import { subscribeChallengeRoute, subscribeConfirmRoute } from "./membership/subscribe";
import { membershipRoute } from "./membership/service";
import { stripeWebhookRoute } from "./membership/webhook";
import { reportRoute, signalRoute } from "./moderation/service";
import { confirmPostRoute, deletePostRoute } from "./posts/confirm";
import { devPutProfileAsset, prepareProfileAsset } from "./profiles/assets";
import {
  getUserFollowsRoute,
  getUserPostsRoute,
  getUserProfileRoute,
  putProfileRoute,
} from "./profiles/service";
import {
  completeUpload,
  devUploadMediaAsset,
  devPutUploadObject,
  prepareUpload,
  streamWebhookRoute,
} from "./uploads/service";
import { HttpError, jsonResponse, optionsResponse } from "./shared/http";
import { guardRequest, normalizeApiPath } from "./security/request_guard";
import { turnstileConfigRoute, turnstilePageRoute } from "./security/turnstile";

export async function routeRequest(
  request: Request,
  env: Env,
): Promise<Response> {
  const url = new URL(request.url);
  const path = normalizeApiPath(url.pathname);
  if (request.method === "OPTIONS") {
    return optionsResponse();
  }
  await guardRequest(request, env, path);

  if (request.method === "GET" && path === "/health") {
    return jsonResponse({
      ok: true,
      service: "citizenapp-square-api",
      storage_backend: "cloudflare-images-stream",
      // 广场主媒体进 Images / Stream；R2 只保留 manifest，链上只记录发布索引和哈希。
      content_on_chain: false,
    });
  }

  if (request.method === "GET" && path === "/v1/chain/bootstrap") {
    return chainBootstrapRoute(request, env);
  }
  if (request.method === "GET" && path === "/v1/security/turnstile") {
    return turnstilePageRoute(env);
  }
  if (request.method === "GET" && path === "/v1/security/config") {
    return turnstileConfigRoute(env);
  }
  if (request.method === "POST" && path === "/v1/chain/extrinsics/relay") {
    return relaySignedExtrinsicRoute(request, env);
  }

  if (request.method === "POST" && path === "/v1/square/auth/challenge") {
    return createLoginChallenge(request, env);
  }
  if (request.method === "POST" && path === "/v1/square/auth/session") {
    return createSession(request, env);
  }
  if (request.method === "POST" && path === "/v1/square/auth/device/register") {
    return registerDeviceSubkey(request, env);
  }
  if (request.method === "GET" && path === "/v1/square/membership") {
    return membershipRoute(request, env);
  }
  if (request.method === "POST" && path === "/v1/square/membership/subscribe/challenge") {
    return subscribeChallengeRoute(request, env);
  }
  if (request.method === "POST" && path === "/v1/square/membership/subscribe") {
    return subscribeConfirmRoute(request, env);
  }
  if (request.method === "POST" && path === "/v1/square/membership/webhook") {
    return stripeWebhookRoute(request, env);
  }
  if (request.method === "POST" && path === "/v1/square/membership/cancel/challenge") {
    return cancelMembershipChallengeRoute(request, env);
  }
  if (request.method === "POST" && path === "/v1/square/membership/cancel") {
    return cancelMembershipRoute(request, env);
  }
  if (request.method === "POST" && path === "/v1/square/account/delete/challenge") {
    return deleteAccountChallengeRoute(request, env);
  }
  if (request.method === "POST" && path === "/v1/square/account/delete") {
    return deleteAccountRoute(request, env);
  }
  if (request.method === "POST" && path === "/v1/square/uploads/prepare") {
    return prepareUpload(request, env);
  }
  if (request.method === "PUT" && path === "/v1/square/uploads/dev-put") {
    return devPutUploadObject(request, env);
  }
  if (
    (request.method === "POST" || request.method === "PATCH") &&
    path === "/v1/square/uploads/dev-media"
  ) {
    return devUploadMediaAsset(request, env);
  }
  if (request.method === "POST" && path === "/v1/square/uploads/complete") {
    return completeUpload(request, env);
  }
  if (request.method === "POST" && path === "/v1/square/uploads/stream/webhook") {
    return streamWebhookRoute(request, env);
  }
  if (request.method === "POST" && path === "/v1/square/posts/confirm") {
    return confirmPostRoute(request, env);
  }
  if (request.method === "DELETE" && path.startsWith("/v1/square/posts/")) {
    return deletePostRoute(request, env, path.slice("/v1/square/posts/".length));
  }
  if (request.method === "GET" && path.startsWith("/v1/square/media/")) {
    return mediaRoute(request, env, path);
  }
  if (request.method === "GET" && path.startsWith("/v1/square/feed/")) {
    return feedRoute(request, env, parseFeedKind(path));
  }
  if (request.method === "PUT" && path === "/v1/square/profile") {
    return putProfileRoute(request, env);
  }
  if (request.method === "POST" && path === "/v1/square/profile/assets/prepare") {
    return prepareProfileAsset(request, env);
  }
  if (request.method === "PUT" && path === "/v1/square/profile/assets/dev-put") {
    return devPutProfileAsset(request, env);
  }
  if (request.method === "GET" && path.startsWith("/v1/square/users/")) {
    return routeUserPath(request, env, path);
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
  if (request.method === "POST" && path === "/v1/chat/signals") {
    return submitChatSignal(request, env);
  }
  if (request.method === "POST" && path === "/v1/chat/turn") {
    return createTurnCredentials(request, env);
  }
  if (request.method === "GET" && path === "/v1/chat/ws") {
    return openChatWebSocket(request, env);
  }

  throw new HttpError(404, "route_not_found", "广场接口不存在");
}

function routeUserPath(
  request: Request,
  env: Env,
  path: string,
): Promise<Response> {
  const rest = path.slice("/v1/square/users/".length);
  const segments = rest.split("/").filter((segment) => segment.length > 0);
  const account = segments[0] ?? "";
  if (segments.length === 1) {
    return getUserProfileRoute(request, env, account);
  }
  if (segments.length === 2 && segments[1] === "posts") {
    return getUserPostsRoute(request, env, account);
  }
  if (segments.length === 2 && segments[1] === "follows") {
    return getUserFollowsRoute(request, env, account);
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
