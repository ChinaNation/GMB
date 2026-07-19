import type { Env, FeedKind } from "./types";
import {
  deleteAccountChallengeRoute,
  deleteAccountRoute,
} from "./account/service";
import { createLoginChallenge, createSession, registerDeviceSubkey } from "./auth/service";
import { chainBootstrapRoute } from "./chain/bootstrap";
import { constitutionRoute } from "./chain/constitution";
import { relaySignedExtrinsicRoute } from "./chain/extrinsic_relay";
import { deleteContactRoute, listContactsRoute, putContactRoute } from "./contacts/service";
import {
  consumeChatKeyPackage,
  fetchChatKeyPackages,
  openChatWebSocket,
  publishChatKeyPackage,
  registerChatDevice,
  submitChatEnvelope,
  submitChatSignal,
} from "./chat/service";
import {
  ackChatRelay,
  getChatRelayBlob,
  initChatRelay,
  putChatRelayBlob,
} from "./chat/relay";
import { feedRoute } from "./feeds/service";
import { followRoute, unfollowRoute } from "./feeds/follows";
import { mediaRoute } from "./media/service";
import { platformSubscriptionConfirmRoute } from "./membership/citizen_coin";
import { membershipRoute } from "./membership/service";
import {
  creatorOverviewRoute,
  creatorPlanOfRoute,
  creatorPlanRoute,
  creatorPlanSaveRoute,
  creatorSubscriptionConfirmRoute,
} from "./membership/creator";
import { signalRoute } from "./moderation/service";
import { isTopupPath, routeTopup } from "./topup/routes";
import { confirmPostRoute, deletePostRoute } from "./posts/confirm";
import { prepareProfileAsset, putProfileAsset } from "./profiles/assets";
import {
  getUserFollowsRoute,
  getUserPostsRoute,
  getUserProfileRoute,
  putProfileRoute,
} from "./profiles/service";
import {
  completeUpload,
  putManifest,
  putMediaAsset,
  prepareUpload,
  streamWebhookRoute,
} from "./uploads/service";
import { HttpError, jsonResponse, optionsResponse } from "./shared/http";
import { guardRequest, normalizeApiPath } from "./security/request_guard";
import { turnstileConfigRoute, turnstilePageRoute } from "./security/turnstile";
import { assertKnownRoute } from "./limits/request";

export async function routeRequest(
  request: Request,
  env: Env,
): Promise<Response> {
  const url = new URL(request.url);
  const path = normalizeApiPath(url.pathname);
  assertKnownRoute(request.method, path);
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
  if (request.method === "GET" && path === "/v1/constitution") {
    return constitutionRoute(request, env);
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
  // 平台会员公民币轨：订阅/取消由 App 热钱包 extrinsic 上链，此处只做上链后镜像确认。
  if (request.method === "POST" && path === "/v1/square/membership/confirm") {
    return platformSubscriptionConfirmRoute(request, env);
  }
  if (request.method === "POST" && path === "/v1/square/account/delete/challenge") {
    return deleteAccountChallengeRoute(request, env);
  }
  if (request.method === "POST" && path === "/v1/square/account/delete") {
    return deleteAccountRoute(request, env);
  }
  // 稳定币充值购买公民币:App(config/submit/status)+ 本地部署控制台结算(settlement/*)。
  if (isTopupPath(path)) {
    return routeTopup(request, env, path);
  }
  if (request.method === "GET" && path === "/v1/square/creator/plan") {
    return creatorPlanRoute(request, env);
  }
  if (request.method === "GET" && path === "/v1/square/creator/overview") {
    return creatorOverviewRoute(request, env);
  }
  if (request.method === "POST" && path === "/v1/square/creator/plan") {
    return creatorPlanSaveRoute(request, env);
  }
  if (request.method === "POST" && path === "/v1/square/creator/subscription/confirm") {
    return creatorSubscriptionConfirmRoute(request, env);
  }
  if (request.method === "GET" && path.startsWith("/v1/square/creator/plan/")) {
    return creatorPlanOfRoute(request, env, path.slice("/v1/square/creator/plan/".length));
  }
  if (request.method === "GET" && path === "/v1/square/contacts") {
    return listContactsRoute(request, env);
  }
  if (request.method === "PUT" && path.startsWith("/v1/square/contacts/")) {
    return putContactRoute(request, env, path.slice("/v1/square/contacts/".length));
  }
  if (request.method === "DELETE" && path.startsWith("/v1/square/contacts/")) {
    return deleteContactRoute(request, env, path.slice("/v1/square/contacts/".length));
  }
  if (request.method === "POST" && path === "/v1/square/uploads/prepare") {
    return prepareUpload(request, env);
  }
  if (request.method === "PUT" && path === "/v1/square/uploads/manifest") {
    return putManifest(request, env);
  }
  if (request.method === "PUT" && path === "/v1/square/uploads/media") {
    return putMediaAsset(request, env);
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
  if (request.method === "PUT" && path === "/v1/square/profile/assets") {
    return putProfileAsset(request, env);
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
  if (request.method === "GET" && path === "/v1/chat/ws") {
    return openChatWebSocket(request, env);
  }
  // 大媒体(>100MB)瞬时中转:init 申请 → blob 流式 PUT/GET → ack 删。仅薪火 + 仅 >100MB。
  if (request.method === "POST" && path === "/v1/chat/relay/init") {
    return initChatRelay(request, env);
  }
  if (path.startsWith("/v1/chat/relay/") && path.endsWith("/blob")) {
    const relayKey = path.slice("/v1/chat/relay/".length, -"/blob".length);
    if (request.method === "PUT") {
      return putChatRelayBlob(request, env, relayKey);
    }
    if (request.method === "GET") {
      return getChatRelayBlob(request, env, relayKey);
    }
  }
  if (
    request.method === "POST" &&
    path.startsWith("/v1/chat/relay/") &&
    path.endsWith("/ack")
  ) {
    const relayKey = path.slice("/v1/chat/relay/".length, -"/ack".length);
    return ackChatRelay(request, env, relayKey);
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
