import type { Env } from "../types";
import {
  HttpError,
  jsonResponse,
  readJson,
  requireSession,
} from "../shared/http";
import { nowMs } from "../shared/time";
import {
  assertBase64Url,
  assertChatAccount,
  assertCipherSuite,
  assertDeviceId,
  assertDevicePublicKeyHex,
  assertEnvelopeId,
  assertKeyPackageId,
  assertLimit,
  assertMlsMessageKind,
  assertPositiveMillis,
} from "./codec";
import {
  buildDeviceBindingSigningMessageBase64Url,
  verifyDeviceBindingSignature,
} from "./binding";
import { isSha256Hex } from "../shared/hash";
import { createDownloadUrl, createUploadUrl } from "../storage/presigned";
import { sanitizeOwnerAccount } from "../storage/r2_keys";
import {
  notifyChatRealtime,
  requireChatRealtimeNamespace,
  type ChatNoticePayload,
} from "./realtime";

interface RegisterDeviceRequest {
  owner_account?: unknown;
  device_id?: unknown;
  device_public_key_hex?: unknown;
  binding_signature?: unknown;
  expires_at?: unknown;
  nonce?: unknown;
}

interface PublishKeyPackageRequest {
  owner_account?: unknown;
  device_id?: unknown;
  device_public_key_hex?: unknown;
  key_package_id?: unknown;
  key_package?: unknown;
  cipher_suite?: unknown;
  created_at?: unknown;
  expires_at?: unknown;
}

interface ConsumeKeyPackageRequest {
  owner_account?: unknown;
  key_package_id?: unknown;
  requester_account?: unknown;
}

interface SubmitEnvelopeRequest {
  envelope_id?: unknown;
  conversation_id?: unknown;
  sender_account?: unknown;
  sender_device_id?: unknown;
  recipient_account?: unknown;
  recipient_device_id?: unknown;
  mls_message_kind?: unknown;
  envelope?: unknown;
  attachment_manifest_key?: unknown;
  created_at?: unknown;
  expires_at?: unknown;
}

interface AckEnvelopeRequest {
  owner_account?: unknown;
  device_id?: unknown;
  envelope_id?: unknown;
}

interface PrepareAttachmentChunkRequest {
  chunk_id?: unknown;
  byte_size?: unknown;
}

interface PrepareChatAttachmentRequest {
  owner_account?: unknown;
  device_id?: unknown;
  conversation_id?: unknown;
  attachment_id?: unknown;
  manifest_byte_size?: unknown;
  chunks?: unknown;
}

interface CompleteChatAttachmentRequest {
  owner_account?: unknown;
  device_id?: unknown;
  conversation_id?: unknown;
  attachment_id?: unknown;
  manifest_object_key?: unknown;
  manifest_hash?: unknown;
  chunk_refs?: unknown;
}

interface PrepareChatAttachmentDownloadRequest {
  owner_account?: unknown;
  device_id?: unknown;
  conversation_id?: unknown;
  attachment_id?: unknown;
  manifest_object_key?: unknown;
  manifest_hash?: unknown;
  chunk_refs?: unknown;
}

interface ChatDeviceRow {
  owner_account: string;
  device_id: string;
  device_public_key_hex: string;
  expires_at: number;
}

interface ChatKeyPackageRow {
  owner_account: string;
  device_id: string;
  device_public_key_hex: string;
  key_package_id: string;
  key_package: string;
  cipher_suite: string;
  created_at: number;
  expires_at: number;
  consumed_at: number | null;
}

interface ChatEnvelopeRow {
  envelope_id: string;
  conversation_id: string;
  sender_account: string;
  sender_device_id: string;
  recipient_account: string;
  recipient_device_id: string | null;
  mls_message_kind: string;
  encrypted_payload: string;
  attachment_manifest_key: string | null;
  created_at: number;
  expires_at: number;
}

interface ChatEnvelopeCleanupRow {
  envelope_id: string;
  attachment_manifest_key: string | null;
}

export async function registerChatDevice(
  request: Request,
  env: Env,
): Promise<Response> {
  // 设备登记只建立钱包账户与 IM 设备公钥的授权关系，不保存聊天明文。
  const session = await requireSession(request, env);
  const body = await readJson<RegisterDeviceRequest>(request);
  const ownerAccount = assertChatAccount(body.owner_account);
  if (ownerAccount !== session.owner_account) {
    throw new HttpError(
      403,
      "chat_owner_mismatch",
      "只能登记当前钱包账户的 IM 设备",
    );
  }

  const deviceId = assertDeviceId(body.device_id);
  const devicePublicKeyHex = assertDevicePublicKeyHex(
    body.device_public_key_hex,
  );
  const expiresAt = assertPositiveMillis(
    body.expires_at,
    "invalid_binding_expires_at",
    "IM 设备绑定过期时间不合法",
  );
  if (expiresAt <= nowMs()) {
    throw new HttpError(400, "expired_device_binding", "IM 设备绑定凭证已过期");
  }
  if (
    typeof body.nonce !== "string" ||
    body.nonce.length < 8 ||
    body.nonce.length > 128
  ) {
    throw new HttpError(
      400,
      "invalid_binding_nonce",
      "IM 设备绑定 nonce 不合法",
    );
  }
  if (
    typeof body.binding_signature !== "string" ||
    body.binding_signature.length === 0
  ) {
    throw new HttpError(
      400,
      "invalid_binding_signature",
      "IM 设备绑定签名不合法",
    );
  }

  const bindingInput = {
    wallet_account: ownerAccount,
    im_device_id: deviceId,
    im_device_pubkey: devicePublicKeyHex,
    expires_at_millis: expiresAt,
    nonce: body.nonce,
  };
  const validSignature = await verifyDeviceBindingSignature(
    bindingInput,
    body.binding_signature,
  );
  if (!validSignature) {
    throw new HttpError(
      401,
      "invalid_device_binding_signature",
      "IM 设备绑定签名校验失败",
    );
  }

  await env.DB.prepare(
    `INSERT INTO chat_devices
      (owner_account, device_id, device_public_key_hex, binding_signature, expires_at, created_at, revoked_at)
      VALUES (?, ?, ?, ?, ?, ?, NULL)
      ON CONFLICT(owner_account, device_id) DO UPDATE SET
        device_public_key_hex = excluded.device_public_key_hex,
        binding_signature = excluded.binding_signature,
        expires_at = excluded.expires_at,
        created_at = excluded.created_at,
        revoked_at = NULL`,
  )
    .bind(
      ownerAccount,
      deviceId,
      devicePublicKeyHex,
      body.binding_signature,
      expiresAt,
      nowMs(),
    )
    .run();

  return jsonResponse({
    ok: true,
    owner_account: ownerAccount,
    device_id: deviceId,
    device_public_key_hex: devicePublicKeyHex,
    binding_message: buildDeviceBindingSigningMessageBase64Url(bindingInput),
    expires_at: expiresAt,
  });
}

export async function publishChatKeyPackage(
  request: Request,
  env: Env,
): Promise<Response> {
  // KeyPackage 是发起 MLS 会话所需的预密钥材料，Worker 只做存取和一次性消费控制。
  const session = await requireSession(request, env);
  const body = await readJson<PublishKeyPackageRequest>(request);
  const ownerAccount = assertChatAccount(body.owner_account);
  if (ownerAccount !== session.owner_account) {
    throw new HttpError(
      403,
      "chat_owner_mismatch",
      "只能发布当前钱包账户的 KeyPackage",
    );
  }
  const deviceId = assertDeviceId(body.device_id);
  const devicePublicKeyHex = assertDevicePublicKeyHex(
    body.device_public_key_hex,
  );
  await requireActiveDevice(env, ownerAccount, deviceId, devicePublicKeyHex);

  const keyPackageId = assertKeyPackageId(body.key_package_id);
  const keyPackage = assertBase64Url(
    body.key_package,
    "invalid_key_package",
    "KeyPackage 必须是 base64url 编码",
  );
  const cipherSuite = assertCipherSuite(body.cipher_suite);
  const createdAt = assertPositiveMillis(
    body.created_at,
    "invalid_key_package_created_at",
    "KeyPackage 创建时间不合法",
  );
  const expiresAt = assertPositiveMillis(
    body.expires_at,
    "invalid_key_package_expires_at",
    "KeyPackage 过期时间不合法",
  );
  if (expiresAt <= nowMs() || expiresAt <= createdAt) {
    throw new HttpError(400, "expired_key_package", "KeyPackage 已过期");
  }

  const existing = await env.DB.prepare(
    `SELECT key_package_id FROM chat_keypackages WHERE key_package_id = ?`,
  )
    .bind(keyPackageId)
    .first<{ key_package_id: string }>();
  if (existing) {
    throw new HttpError(409, "key_package_exists", "KeyPackage 已存在");
  }

  await env.DB.prepare(
    `INSERT INTO chat_keypackages
      (owner_account, device_id, key_package_id, key_package, cipher_suite, created_at,
        expires_at, consumed_at, consumed_by_account)
      VALUES (?, ?, ?, ?, ?, ?, ?, NULL, NULL)`,
  )
    .bind(
      ownerAccount,
      deviceId,
      keyPackageId,
      keyPackage,
      cipherSuite,
      createdAt,
      expiresAt,
    )
    .run();

  return jsonResponse({
    ok: true,
    owner_account: ownerAccount,
    device_id: deviceId,
    key_package_id: keyPackageId,
    expires_at: expiresAt,
  });
}

export async function fetchChatKeyPackages(
  request: Request,
  env: Env,
): Promise<Response> {
  await requireSession(request, env);
  const url = new URL(request.url);
  const ownerAccount = assertChatAccount(url.pathname.split("/").pop());
  const limit = assertLimit(url.searchParams.get("limit"), 1, 20);
  const rows = await env.DB.prepare(
    `SELECT kp.owner_account, kp.device_id, d.device_public_key_hex, kp.key_package_id,
        kp.key_package, kp.cipher_suite, kp.created_at, kp.expires_at, kp.consumed_at
      FROM chat_keypackages kp
      JOIN chat_devices d
        ON d.owner_account = kp.owner_account AND d.device_id = kp.device_id
      WHERE kp.owner_account = ?
        AND kp.consumed_at IS NULL
        AND kp.expires_at > ?
        AND d.revoked_at IS NULL
        AND d.expires_at > ?
      ORDER BY kp.created_at ASC
      LIMIT ?`,
  )
    .bind(ownerAccount, nowMs(), nowMs(), limit)
    .all<ChatKeyPackageRow>();

  return jsonResponse({
    ok: true,
    owner_account: ownerAccount,
    key_packages: rows.results ?? [],
  });
}

export async function consumeChatKeyPackage(
  request: Request,
  env: Env,
): Promise<Response> {
  const session = await requireSession(request, env);
  const body = await readJson<ConsumeKeyPackageRequest>(request);
  const ownerAccount = assertChatAccount(body.owner_account);
  const keyPackageId = assertKeyPackageId(body.key_package_id);
  const requesterAccount = assertChatAccount(
    body.requester_account,
    "invalid_requester_account",
  );
  if (requesterAccount !== session.owner_account) {
    throw new HttpError(
      403,
      "requester_mismatch",
      "只能以当前钱包账户消费 KeyPackage",
    );
  }

  const row = await env.DB.prepare(
    `SELECT kp.owner_account, kp.device_id, d.device_public_key_hex, kp.key_package_id,
        kp.key_package, kp.cipher_suite, kp.created_at, kp.expires_at, kp.consumed_at
      FROM chat_keypackages kp
      JOIN chat_devices d
        ON d.owner_account = kp.owner_account AND d.device_id = kp.device_id
      WHERE kp.owner_account = ?
        AND kp.key_package_id = ?
        AND kp.consumed_at IS NULL
        AND kp.expires_at > ?
        AND d.revoked_at IS NULL
        AND d.expires_at > ?`,
  )
    .bind(ownerAccount, keyPackageId, nowMs(), nowMs())
    .first<ChatKeyPackageRow>();
  if (!row) {
    throw new HttpError(
      404,
      "key_package_not_available",
      "KeyPackage 不存在或已被消费",
    );
  }

  const consumedAt = nowMs();
  await env.DB.prepare(
    `UPDATE chat_keypackages
      SET consumed_at = ?, consumed_by_account = ?
      WHERE key_package_id = ? AND consumed_at IS NULL`,
  )
    .bind(consumedAt, requesterAccount, keyPackageId)
    .run();

  return jsonResponse({
    ok: true,
    key_package: {
      ...row,
      consumed_at: consumedAt,
    },
  });
}

export async function submitChatEnvelope(
  request: Request,
  env: Env,
): Promise<Response> {
  await cleanupExpiredChatEnvelopes(env);
  const session = await requireSession(request, env);
  const body = await readJson<SubmitEnvelopeRequest>(request);
  const envelopeId = assertEnvelopeId(body.envelope_id);
  const conversationId = assertNonEmptyString(
    body.conversation_id,
    "invalid_conversation_id",
  );
  const senderAccount = assertChatAccount(
    body.sender_account,
    "invalid_sender_account",
  );
  if (senderAccount !== session.owner_account) {
    throw new HttpError(
      403,
      "sender_mismatch",
      "只能发送当前钱包账户的 IM 密文",
    );
  }
  const senderDeviceId = assertDeviceId(body.sender_device_id);
  await requireActiveDevice(env, senderAccount, senderDeviceId);

  const recipientAccount = assertChatAccount(
    body.recipient_account,
    "invalid_recipient_account",
  );
  const recipientDeviceId =
    typeof body.recipient_device_id === "string" &&
    body.recipient_device_id.length > 0
      ? assertDeviceId(body.recipient_device_id)
      : null;
  const mlsMessageKind = assertMlsMessageKind(body.mls_message_kind);
  const encryptedPayload = assertBase64Url(
    body.envelope,
    "invalid_envelope",
    "密文 envelope 必须是 base64url 编码",
  );
  const attachmentManifestKey =
    typeof body.attachment_manifest_key === "string" &&
    body.attachment_manifest_key.length > 0
      ? assertNonEmptyString(
          body.attachment_manifest_key,
          "invalid_attachment_manifest_key",
        )
      : null;
  const createdAt = assertPositiveMillis(
    body.created_at,
    "invalid_envelope_created_at",
    "密文创建时间不合法",
  );
  const expiresAt = assertPositiveMillis(
    body.expires_at,
    "invalid_envelope_expires_at",
    "密文过期时间不合法",
  );
  if (expiresAt <= nowMs() || expiresAt <= createdAt) {
    throw new HttpError(400, "expired_envelope", "密文 envelope 已过期");
  }

  const existing = await env.DB.prepare(
    `SELECT envelope_id FROM chat_envelopes WHERE envelope_id = ?`,
  )
    .bind(envelopeId)
    .first<{ envelope_id: string }>();
  if (existing) {
    throw new HttpError(409, "envelope_exists", "密文 envelope 已存在");
  }

  await env.DB.prepare(
    `INSERT INTO chat_envelopes
      (envelope_id, conversation_id, sender_account, sender_device_id, recipient_account,
        recipient_device_id, mls_message_kind, encrypted_payload, attachment_manifest_key,
        created_at, expires_at)
      VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)`,
  )
    .bind(
      envelopeId,
      conversationId,
      senderAccount,
      senderDeviceId,
      recipientAccount,
      recipientDeviceId,
      mlsMessageKind,
      encryptedPayload,
      attachmentManifestKey,
      createdAt,
      expiresAt,
    )
    .run();

  await notifyChatRealtime(env, {
    type: "gmb_im_new_envelope_v1",
    envelope_id: envelopeId,
    conversation_id: conversationId,
    recipient_account: recipientAccount,
    recipient_device_id: recipientDeviceId,
    mls_message_kind: mlsMessageKind,
    created_at: createdAt,
  }).catch(() => 0);

  return jsonResponse({
    ok: true,
    envelope_id: envelopeId,
    delivery_state: "stored",
  });
}

export async function openChatWebSocket(
  request: Request,
  env: Env,
): Promise<Response> {
  if (request.headers.get("upgrade")?.toLowerCase() !== "websocket") {
    throw new HttpError(426, "websocket_required", "请使用 WebSocket 连接");
  }

  const session = await requireSession(request, env);
  const url = new URL(request.url);
  const ownerAccount = assertChatAccount(url.searchParams.get("owner_account"));
  if (ownerAccount !== session.owner_account) {
    throw new HttpError(
      403,
      "chat_owner_mismatch",
      "只能连接当前钱包账户的实时通知",
    );
  }
  const deviceId = assertDeviceId(url.searchParams.get("device_id"));
  await requireActiveDevice(env, ownerAccount, deviceId);

  const realtime = requireChatRealtimeNamespace(env);
  return realtime.getByName(ownerAccount).fetch(request);
}

export async function fetchPendingChatEnvelopes(
  request: Request,
  env: Env,
): Promise<Response> {
  await cleanupExpiredChatEnvelopes(env);
  const session = await requireSession(request, env);
  const url = new URL(request.url);
  const ownerAccount = assertChatAccount(url.searchParams.get("owner_account"));
  if (ownerAccount !== session.owner_account) {
    throw new HttpError(
      403,
      "chat_owner_mismatch",
      "只能拉取当前钱包账户的密文",
    );
  }
  const deviceId = assertDeviceId(url.searchParams.get("device_id"));
  await requireActiveDevice(env, ownerAccount, deviceId);
  const limit = assertLimit(url.searchParams.get("limit"), 50, 200);
  const rows = await env.DB.prepare(
    `SELECT envelope_id, conversation_id, sender_account, sender_device_id, recipient_account,
        recipient_device_id, mls_message_kind, encrypted_payload, attachment_manifest_key,
        created_at, expires_at
      FROM chat_envelopes
      WHERE recipient_account = ?
        AND (recipient_device_id IS NULL OR recipient_device_id = ?)
        AND expires_at > ?
      ORDER BY created_at ASC
      LIMIT ?`,
  )
    .bind(ownerAccount, deviceId, nowMs(), limit)
    .all<ChatEnvelopeRow>();

  return jsonResponse({
    ok: true,
    owner_account: ownerAccount,
    device_id: deviceId,
    envelopes: (rows.results ?? []).map((row) => ({
      envelope_id: row.envelope_id,
      conversation_id: row.conversation_id,
      sender_account: row.sender_account,
      sender_device_id: row.sender_device_id,
      recipient_account: row.recipient_account,
      recipient_device_id: row.recipient_device_id,
      mls_message_kind: row.mls_message_kind,
      envelope: row.encrypted_payload,
      attachment_manifest_key: row.attachment_manifest_key,
      created_at: row.created_at,
      expires_at: row.expires_at,
    })),
  });
}

export async function ackChatEnvelope(
  request: Request,
  env: Env,
): Promise<Response> {
  const session = await requireSession(request, env);
  const body = await readJson<AckEnvelopeRequest>(request);
  const ownerAccount = assertChatAccount(body.owner_account);
  if (ownerAccount !== session.owner_account) {
    throw new HttpError(
      403,
      "chat_owner_mismatch",
      "只能确认当前钱包账户的密文",
    );
  }
  const deviceId = assertDeviceId(body.device_id);
  await requireActiveDevice(env, ownerAccount, deviceId);
  const envelopeId = assertEnvelopeId(body.envelope_id);
  const row = await env.DB.prepare(
    `SELECT envelope_id, attachment_manifest_key
      FROM chat_envelopes
      WHERE envelope_id = ?
        AND recipient_account = ?
        AND (recipient_device_id IS NULL OR recipient_device_id = ?)
      LIMIT 1`,
  )
    .bind(envelopeId, ownerAccount, deviceId)
    .first<ChatEnvelopeCleanupRow>();

  if (!row) {
    return jsonResponse({
      ok: true,
      envelope_id: envelopeId,
      acked: false,
      deleted: false,
      deleted_attachment_objects: 0,
    });
  }

  const deletedAttachmentObjects = row.attachment_manifest_key
    ? await deleteChatAttachmentObjectsByManifest(
        env,
        row.attachment_manifest_key,
      )
    : 0;
  const result = await env.DB.prepare(
    `DELETE FROM chat_envelopes
      WHERE envelope_id = ?
        AND recipient_account = ?
        AND (recipient_device_id IS NULL OR recipient_device_id = ?)`,
  )
    .bind(envelopeId, ownerAccount, deviceId)
    .run();

  return jsonResponse({
    ok: true,
    envelope_id: envelopeId,
    acked: (result.meta?.changes ?? 0) > 0,
    deleted: (result.meta?.changes ?? 0) > 0,
    deleted_attachment_objects: deletedAttachmentObjects,
  });
}

export async function prepareChatAttachmentUpload(
  request: Request,
  env: Env,
): Promise<Response> {
  const session = await requireSession(request, env);
  const body = await readJson<PrepareChatAttachmentRequest>(request);
  const ownerAccount = assertChatAccount(body.owner_account);
  if (ownerAccount !== session.owner_account) {
    throw new HttpError(
      403,
      "chat_owner_mismatch",
      "只能为当前钱包账户上传聊天附件",
    );
  }
  const deviceId = assertDeviceId(body.device_id);
  await requireActiveDevice(env, ownerAccount, deviceId);
  const conversationId = assertNonEmptyString(
    body.conversation_id,
    "invalid_conversation_id",
  );
  const attachmentId = assertEnvelopeId(body.attachment_id);
  const manifestByteSize = assertAttachmentByteSize(
    body.manifest_byte_size,
    "invalid_manifest_byte_size",
  );
  const chunks = assertAttachmentChunks(body.chunks);
  const objectPlan = buildChatAttachmentObjectPlan(
    ownerAccount,
    conversationId,
    attachmentId,
    chunks,
  );
  const requestUrl = new URL(request.url);
  const expiresSeconds = Number.parseInt(
    env.SQUARE_UPLOAD_URL_TTL_SECONDS ?? "900",
    10,
  );

  const manifestUploadUrl = await createUploadUrl(env, {
    object_key: objectPlan.manifest_object_key,
    content_type: "application/octet-stream",
    expires_seconds: Number.isSafeInteger(expiresSeconds)
      ? expiresSeconds
      : 900,
    request_url: requestUrl,
    upload_id: attachmentId,
    dev_upload_path: "/v1/chat/attachments/dev-put",
  });
  const chunkUploadUrls = await Promise.all(
    objectPlan.chunks.map((chunk) =>
      createUploadUrl(env, {
        object_key: chunk.object_key,
        content_type: "application/octet-stream",
        expires_seconds: Number.isSafeInteger(expiresSeconds)
          ? expiresSeconds
          : 900,
        request_url: requestUrl,
        upload_id: attachmentId,
        dev_upload_path: "/v1/chat/attachments/dev-put",
      }),
    ),
  );

  return jsonResponse({
    ok: true,
    owner_account: ownerAccount,
    device_id: deviceId,
    conversation_id: conversationId,
    attachment_id: attachmentId,
    manifest_byte_size: manifestByteSize,
    manifest_object_key: objectPlan.manifest_object_key,
    manifest_upload_url: manifestUploadUrl,
    chunks: objectPlan.chunks.map((chunk, index) => ({
      chunk_id: chunk.chunk_id,
      byte_size: chunk.byte_size,
      object_key: chunk.object_key,
      upload_url: chunkUploadUrls[index],
    })),
  });
}

export async function devPutChatAttachmentObject(
  request: Request,
  env: Env,
): Promise<Response> {
  if (env.SQUARE_DEV_UPLOAD_PROXY !== "1") {
    throw new HttpError(
      404,
      "dev_upload_proxy_disabled",
      "开发上传代理未启用",
    );
  }
  const session = await requireSession(request, env);
  const requestUrl = new URL(request.url);
  const objectKey = requestUrl.searchParams.get("object_key");
  if (!objectKey || !isChatAttachmentObjectKey(session.owner_account, objectKey)) {
    throw new HttpError(
      403,
      "chat_attachment_object_forbidden",
      "无权写入该聊天附件对象",
    );
  }
  const contentType =
    request.headers.get("content-type") ?? "application/octet-stream";
  const body = await request.arrayBuffer();
  await env.SQUARE_MEDIA.put(objectKey, body, {
    httpMetadata: {
      contentType,
    },
  });

  return jsonResponse({
    ok: true,
    object_key: objectKey,
    byte_size: body.byteLength,
  });
}

export async function completeChatAttachmentUpload(
  request: Request,
  env: Env,
): Promise<Response> {
  const session = await requireSession(request, env);
  const body = await readJson<CompleteChatAttachmentRequest>(request);
  const ownerAccount = assertChatAccount(body.owner_account);
  if (ownerAccount !== session.owner_account) {
    throw new HttpError(
      403,
      "chat_owner_mismatch",
      "只能完成当前钱包账户的聊天附件上传",
    );
  }
  const deviceId = assertDeviceId(body.device_id);
  await requireActiveDevice(env, ownerAccount, deviceId);
  assertNonEmptyString(body.conversation_id, "invalid_conversation_id");
  assertEnvelopeId(body.attachment_id);
  const manifestObjectKey = assertChatAttachmentObjectKey(
    ownerAccount,
    body.manifest_object_key,
  );
  if (!isSha256Hex(body.manifest_hash)) {
    throw new HttpError(
      400,
      "invalid_manifest_hash",
      "聊天附件 manifest hash 必须是 sha256 hex",
    );
  }
  const chunkRefs = assertChatAttachmentObjectKeys(ownerAccount, body.chunk_refs);
  const objectKeys = [manifestObjectKey, ...chunkRefs];
  for (const objectKey of objectKeys) {
    const objectMeta = await env.SQUARE_MEDIA.head(objectKey);
    if (!objectMeta) {
      throw new HttpError(
        409,
        "chat_attachment_object_missing",
        `聊天附件密文对象未上传：${objectKey}`,
      );
    }
  }

  return jsonResponse({
    ok: true,
    owner_account: ownerAccount,
    device_id: deviceId,
    manifest_object_key: manifestObjectKey,
    manifest_hash: body.manifest_hash.toLowerCase(),
    chunk_refs: chunkRefs,
    storage_state: "completed",
  });
}

export async function prepareChatAttachmentDownload(
  request: Request,
  env: Env,
): Promise<Response> {
  const session = await requireSession(request, env);
  const body = await readJson<PrepareChatAttachmentDownloadRequest>(request);
  const ownerAccount = assertChatAccount(body.owner_account);
  if (ownerAccount !== session.owner_account) {
    throw new HttpError(
      403,
      "chat_owner_mismatch",
      "只能下载当前钱包账户可访问的聊天附件",
    );
  }
  const deviceId = assertDeviceId(body.device_id);
  await requireActiveDevice(env, ownerAccount, deviceId);
  const conversationId = assertNonEmptyString(
    body.conversation_id,
    "invalid_conversation_id",
  );
  const attachmentId = assertEnvelopeId(body.attachment_id);
  const manifestObjectKey = assertAnyChatAttachmentObjectKey(
    body.manifest_object_key,
  );
  if (!isSha256Hex(body.manifest_hash)) {
    throw new HttpError(
      400,
      "invalid_manifest_hash",
      "聊天附件 manifest hash 必须是 sha256 hex",
    );
  }
  const chunkRefs = assertAnyChatAttachmentObjectKeys(body.chunk_refs);
  assertSameAttachmentPrefix(manifestObjectKey, chunkRefs);
  await requireChatAttachmentAccess(
    env,
    ownerAccount,
    conversationId,
    manifestObjectKey,
  );

  const objectKeys = [manifestObjectKey, ...chunkRefs];
  for (const objectKey of objectKeys) {
    const objectMeta = await env.SQUARE_MEDIA.head(objectKey);
    if (!objectMeta) {
      throw new HttpError(
        404,
        "chat_attachment_object_not_found",
        `聊天附件密文对象不存在：${objectKey}`,
      );
    }
  }

  const requestUrl = new URL(request.url);
  const expiresSeconds = Number.parseInt(
    env.SQUARE_UPLOAD_URL_TTL_SECONDS ?? "900",
    10,
  );
  const safeExpires = Number.isSafeInteger(expiresSeconds)
    ? expiresSeconds
    : 900;
  const devQuery = {
    conversation_id: conversationId,
    manifest_object_key: manifestObjectKey,
  };
  const manifestDownloadUrl = await createDownloadUrl(env, {
    object_key: manifestObjectKey,
    expires_seconds: safeExpires,
    request_url: requestUrl,
    access_id: attachmentId,
    dev_download_path: "/v1/chat/attachments/dev-get",
    dev_query: devQuery,
  });
  const chunkDownloadUrls = await Promise.all(
    chunkRefs.map((objectKey) =>
      createDownloadUrl(env, {
        object_key: objectKey,
        expires_seconds: safeExpires,
        request_url: requestUrl,
        access_id: attachmentId,
        dev_download_path: "/v1/chat/attachments/dev-get",
        dev_query: devQuery,
      }),
    ),
  );

  return jsonResponse({
    ok: true,
    owner_account: ownerAccount,
    device_id: deviceId,
    conversation_id: conversationId,
    attachment_id: attachmentId,
    manifest_object_key: manifestObjectKey,
    manifest_hash: String(body.manifest_hash).toLowerCase(),
    manifest_download_url: manifestDownloadUrl,
    chunks: chunkRefs.map((objectKey, index) => ({
      object_key: objectKey,
      download_url: chunkDownloadUrls[index],
    })),
  });
}

export async function devGetChatAttachmentObject(
  request: Request,
  env: Env,
): Promise<Response> {
  if (env.SQUARE_DEV_UPLOAD_PROXY !== "1") {
    throw new HttpError(
      404,
      "dev_download_proxy_disabled",
      "开发下载代理未启用",
    );
  }
  const session = await requireSession(request, env);
  const requestUrl = new URL(request.url);
  const conversationId = assertNonEmptyString(
    requestUrl.searchParams.get("conversation_id"),
    "invalid_conversation_id",
  );
  const manifestObjectKey = assertAnyChatAttachmentObjectKey(
    requestUrl.searchParams.get("manifest_object_key"),
  );
  const objectKey = assertAnyChatAttachmentObjectKey(
    requestUrl.searchParams.get("object_key"),
  );
  assertSameAttachmentPrefix(manifestObjectKey, [objectKey]);
  await requireChatAttachmentAccess(
    env,
    session.owner_account,
    conversationId,
    manifestObjectKey,
  );

  const object = await env.SQUARE_MEDIA.get(objectKey);
  if (!object) {
    throw new HttpError(
      404,
      "chat_attachment_object_not_found",
      `聊天附件密文对象不存在：${objectKey}`,
    );
  }
  const headers = new Headers();
  object.writeHttpMetadata(headers);
  if (!headers.has("content-type")) {
    headers.set("content-type", "application/octet-stream");
  }
  return new Response(object.body, {
    headers,
  });
}

async function requireActiveDevice(
  env: Env,
  ownerAccount: string,
  deviceId: string,
  expectedPublicKeyHex?: string,
): Promise<ChatDeviceRow> {
  const row = await env.DB.prepare(
    `SELECT owner_account, device_id, device_public_key_hex, expires_at
      FROM chat_devices
      WHERE owner_account = ?
        AND device_id = ?
        AND revoked_at IS NULL
        AND expires_at > ?`,
  )
    .bind(ownerAccount, deviceId, nowMs())
    .first<ChatDeviceRow>();
  if (!row) {
    throw new HttpError(
      403,
      "chat_device_not_registered",
      "IM 设备未绑定或已过期",
    );
  }
  if (
    expectedPublicKeyHex &&
    row.device_public_key_hex !== expectedPublicKeyHex
  ) {
    throw new HttpError(
      403,
      "chat_device_key_mismatch",
      "IM 设备公钥与绑定记录不一致",
    );
  }
  return row;
}

async function requireChatAttachmentAccess(
  env: Env,
  ownerAccount: string,
  conversationId: string,
  manifestObjectKey: string,
): Promise<void> {
  const row = await env.DB.prepare(
    `SELECT envelope_id
      FROM chat_envelopes
      WHERE conversation_id = ?
        AND attachment_manifest_key = ?
        AND (sender_account = ? OR recipient_account = ?)
        AND expires_at > ?
      LIMIT 1`,
  )
    .bind(conversationId, manifestObjectKey, ownerAccount, ownerAccount, nowMs())
    .first<{ envelope_id: string }>();
  if (!row) {
    throw new HttpError(
      403,
      "chat_attachment_access_denied",
      "无权下载该聊天附件",
    );
  }
}

async function cleanupExpiredChatEnvelopes(env: Env): Promise<void> {
  const rows = await env.DB.prepare(
    `SELECT envelope_id, attachment_manifest_key
      FROM chat_envelopes
      WHERE expires_at <= ?
      LIMIT 100`,
  )
    .bind(nowMs())
    .all<ChatEnvelopeCleanupRow>();
  const results = rows.results ?? [];
  for (const row of results) {
    if (row.attachment_manifest_key) {
      await deleteChatAttachmentObjectsByManifest(
        env,
        row.attachment_manifest_key,
      );
    }
  }
  if (results.length > 0) {
    await env.DB.prepare(
      `DELETE FROM chat_envelopes
        WHERE expires_at <= ?`,
    )
      .bind(nowMs())
      .run();
  }
}

async function deleteChatAttachmentObjectsByManifest(
  env: Env,
  manifestObjectKey: string,
): Promise<number> {
  if (!isAnyChatAttachmentObjectKey(manifestObjectKey)) {
    return 0;
  }
  const prefix = `${attachmentPrefix(manifestObjectKey)}/`;
  let cursor: string | undefined;
  let deletedCount = 0;
  do {
    const listed = await env.SQUARE_MEDIA.list({
      prefix,
      cursor,
    });
    const keys = listed.objects.map((object) => object.key);
    if (keys.length > 0) {
      await env.SQUARE_MEDIA.delete(keys);
      deletedCount += keys.length;
    }
    cursor = listed.truncated ? listed.cursor : undefined;
  } while (cursor);
  return deletedCount;
}

function assertNonEmptyString(value: unknown, code: string): string {
  if (
    typeof value !== "string" ||
    value.trim().length === 0 ||
    value.length > 220
  ) {
    throw new HttpError(400, code, "IM 请求字段格式不合法");
  }
  return value;
}

function assertAttachmentByteSize(value: unknown, code: string): number {
  if (
    typeof value !== "number" ||
    !Number.isSafeInteger(value) ||
    value <= 0 ||
    value > 50 * 1024 * 1024
  ) {
    throw new HttpError(400, code, "聊天附件密文字节数不合法");
  }
  return value;
}

function assertAttachmentChunks(value: unknown): PrepareAttachmentChunkRequest[] {
  if (!Array.isArray(value) || value.length === 0 || value.length > 64) {
    throw new HttpError(
      400,
      "invalid_attachment_chunks",
      "聊天附件分片列表不合法",
    );
  }
  return value.map((item) => {
    if (typeof item !== "object" || item === null) {
      throw new HttpError(
        400,
        "invalid_attachment_chunk",
        "聊天附件分片格式不合法",
      );
    }
    const chunk = item as PrepareAttachmentChunkRequest;
    const chunkId = assertNonEmptyString(chunk.chunk_id, "invalid_chunk_id");
    const byteSize = assertAttachmentByteSize(
      chunk.byte_size,
      "invalid_chunk_byte_size",
    );
    return {
      chunk_id: chunkId,
      byte_size: byteSize,
    };
  });
}

function buildChatAttachmentObjectPlan(
  ownerAccount: string,
  conversationId: string,
  attachmentId: string,
  chunks: PrepareAttachmentChunkRequest[],
): {
  manifest_object_key: string;
  chunks: Array<{ chunk_id: string; byte_size: number; object_key: string }>;
} {
  const safeOwner = sanitizeOwnerAccount(ownerAccount);
  const safeConversation = sanitizeOwnerAccount(conversationId);
  const safeAttachment = sanitizeOwnerAccount(attachmentId);
  const basePath = `chat/${safeOwner}/conversations/${safeConversation}/attachments/${safeAttachment}`;
  return {
    manifest_object_key: `${basePath}/manifest.enc`,
    chunks: chunks.map((chunk, index) => ({
      chunk_id: String(chunk.chunk_id),
      byte_size: Number(chunk.byte_size),
      object_key: `${basePath}/chunk_${String(index + 1).padStart(3, "0")}.bin`,
    })),
  };
}

function isChatAttachmentObjectKey(
  ownerAccount: string,
  objectKey: string,
): boolean {
  const safeOwner = sanitizeOwnerAccount(ownerAccount);
  return objectKey.startsWith(`chat/${safeOwner}/conversations/`);
}

function assertChatAttachmentObjectKey(
  ownerAccount: string,
  value: unknown,
): string {
  if (typeof value !== "string" || !isChatAttachmentObjectKey(ownerAccount, value)) {
    throw new HttpError(
      400,
      "invalid_chat_attachment_object_key",
      "聊天附件对象 key 不合法",
    );
  }
  return value;
}

function assertChatAttachmentObjectKeys(
  ownerAccount: string,
  value: unknown,
): string[] {
  if (!Array.isArray(value) || value.length === 0 || value.length > 64) {
    throw new HttpError(
      400,
      "invalid_chat_attachment_object_keys",
      "聊天附件对象 key 列表不合法",
    );
  }
  return value.map((item) => assertChatAttachmentObjectKey(ownerAccount, item));
}

function isAnyChatAttachmentObjectKey(objectKey: string): boolean {
  return /^chat\/[^/]+\/conversations\/[^/]+\/attachments\/[^/]+\/(manifest\.enc|chunk_[0-9]{3}\.bin)$/.test(
    objectKey,
  );
}

function assertAnyChatAttachmentObjectKey(value: unknown): string {
  if (typeof value !== "string" || !isAnyChatAttachmentObjectKey(value)) {
    throw new HttpError(
      400,
      "invalid_chat_attachment_object_key",
      "聊天附件对象 key 不合法",
    );
  }
  return value;
}

function assertAnyChatAttachmentObjectKeys(value: unknown): string[] {
  if (!Array.isArray(value) || value.length === 0 || value.length > 64) {
    throw new HttpError(
      400,
      "invalid_chat_attachment_object_keys",
      "聊天附件对象 key 列表不合法",
    );
  }
  return value.map((item) => assertAnyChatAttachmentObjectKey(item));
}

function attachmentPrefix(objectKey: string): string {
  return objectKey.substring(0, objectKey.lastIndexOf("/"));
}

function assertSameAttachmentPrefix(
  manifestObjectKey: string,
  objectKeys: string[],
): void {
  const prefix = attachmentPrefix(manifestObjectKey);
  for (const objectKey of objectKeys) {
    if (attachmentPrefix(objectKey) !== prefix) {
      throw new HttpError(
        400,
        "chat_attachment_object_mismatch",
        "聊天附件对象不属于同一个附件",
      );
    }
  }
}
