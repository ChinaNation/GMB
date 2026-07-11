import { describe, expect, it } from "vitest";
import { encodeAddress } from "@polkadot/util-crypto/address/encode";
import {
  sr25519PairFromSeed,
  sr25519Sign,
} from "@polkadot/util-crypto/sr25519";
import {
  ackChatEnvelope,
  completeChatAttachmentUpload,
  devGetChatAttachmentObject,
  devPutChatAttachmentObject,
  openChatWebSocket,
  prepareChatAttachmentDownload,
  prepareChatAttachmentUpload,
  registerChatDevice,
} from "../src/chat/service";
import { notifyChatRealtime, type ChatNoticePayload } from "../src/chat/realtime";
import {
  buildChatDeviceBindingMessage,
  buildChatDeviceBindingMessageBase64Url,
} from "../src/chat/binding";
import {
  assertDevicePublicKeyHex,
  base64UrlToBytes,
  bytesToBase64Url,
} from "../src/chat/codec";
import { HttpError } from "../src/shared/http";
import type { Env, SessionState } from "../src/types";

const ownerAccount = "5GrwvaEF5zXb26Fz9rcQpDWS7u4m6DXb6T6TQvF9j5uQ8g6U";

function toHex(buffer: ArrayBuffer | Uint8Array): string {
  const bytes = buffer instanceof Uint8Array ? buffer : new Uint8Array(buffer);
  return [...bytes]
    .map((byte) => byte.toString(16).padStart(2, "0"))
    .join("");
}

describe("chat mailbox helpers", () => {
  it("round-trips base64url bytes without padding", () => {
    const encoded = bytesToBase64Url(new Uint8Array([1, 2, 3, 254, 255]));

    expect(encoded).not.toContain("=");
    expect(Array.from(base64UrlToBytes(encoded))).toEqual([1, 2, 3, 254, 255]);
  });

  it("normalizes Chat device public key hex", () => {
    expect(assertDevicePublicKeyHex("AABBcc")).toBe("aabbcc");
  });

  it("builds deterministic Chat device binding message", () => {
    const input = {
      owner_account: ownerAccount,
      device_id: "alice-phone",
      device_public_key_hex: "aabbcc",
      expires_at: 1_800_000,
      nonce: "nonce-123456",
    };

    expect(buildChatDeviceBindingMessageBase64Url(input)).toBe(
      buildChatDeviceBindingMessageBase64Url(input),
    );
  });

  it("forwards authenticated websocket connections to account Durable Object", async () => {
    const env = fakeEnv({ withDevice: true });
    let routedName = "";
    let routedUrl = "";
    env.CHAT_REALTIME = {
      getByName: (name: string) => {
        routedName = name;
        return {
          fetch: async (request: Request) => {
            routedUrl = request.url;
            return Response.json({ ok: true, routed: true });
          },
        };
      },
    } as unknown as DurableObjectNamespace;

    const response = await openChatWebSocket(
      new Request(
        `https://worker.example/v1/chat/ws?owner_account=${ownerAccount}&device_id=alice-phone`,
        {
          headers: {
            authorization: "Bearer test-session",
            upgrade: "websocket",
          },
        },
      ),
      env,
    );
    const json = (await response.json()) as { routed: boolean };

    expect(json.routed).toBe(true);
    expect(routedName).toBe(ownerAccount);
    expect(routedUrl).toContain("/v1/chat/ws");
  });

  it("routes new envelope notices through recipient account Durable Object", async () => {
    const env = fakeEnv();
    const payload: ChatNoticePayload = {
      type: "gmb_chat_new_envelope_v1",
      envelope_id: "env-123456",
      conversation_id: "dm:alice:bob",
      recipient_account: ownerAccount,
      recipient_device_id: "alice-phone",
      mls_message_kind: "application",
      created_at: Date.now(),
    };
    let routedName = "";
    const routedPayloads: ChatNoticePayload[] = [];
    env.CHAT_REALTIME = {
      getByName: (name: string) => {
        routedName = name;
        return {
          fetch: async (request: Request) => {
            const notice = (await request.json()) as ChatNoticePayload;
            routedPayloads.push(notice);
            return new Response(JSON.stringify({ ok: true, sent: 1 }), {
              headers: { "content-type": "application/json" },
            });
          },
        };
      },
    } as unknown as DurableObjectNamespace;

    const sentCount = await notifyChatRealtime(env, payload);

    expect(sentCount).toBe(1);
    expect(routedName).toBe(ownerAccount);
    expect(routedPayloads[0]?.envelope_id).toBe("env-123456");
  });

  it("deletes Cloudflare envelope and encrypted attachment objects on ack", async () => {
    const manifestObjectKey =
      `chat/${ownerAccount}/conversations/conv-attachment/attachments/att-123456/manifest.enc`;
    const chunkObjectKey =
      `chat/${ownerAccount}/conversations/conv-attachment/attachments/att-123456/chunk_001.bin`;
    const deletedR2Keys: string[] = [];
    const deletedEnvelopeIds: string[] = [];
    const env = fakeEnv({ withDevice: true });
    env.DB = {
      prepare: (sql: string) => ({
        bind: (...values: unknown[]) => ({
          first: async () => {
            if (sql.includes("FROM chat_devices")) {
              return {
                owner_account: ownerAccount,
                device_id: "alice-phone",
                device_public_key_hex: "aabbcc",
                expires_at: Date.now() + 60_000,
              };
            }
            if (sql.includes("FROM chat_envelopes")) {
              return {
                envelope_id: values[0],
                attachment_manifest_key: manifestObjectKey,
              };
            }
            return null;
          },
          run: async () => {
            if (sql.includes("DELETE FROM chat_envelopes")) {
              deletedEnvelopeIds.push(String(values[0]));
              return { meta: { changes: 1 } };
            }
            return { meta: { changes: 0 } };
          },
        }),
      }),
    } as unknown as D1Database;
    env.SQUARE_MEDIA = {
      list: async () => ({
        objects: [{ key: manifestObjectKey }, { key: chunkObjectKey }],
        truncated: false,
      }),
      delete: async (keys: string | string[]) => {
        deletedR2Keys.push(...(Array.isArray(keys) ? keys : [keys]));
      },
    } as unknown as R2Bucket;

    const response = await ackChatEnvelope(
      new Request("https://worker.example/v1/chat/envelopes/ack", {
        method: "POST",
        headers: {
          authorization: "Bearer test-session",
          "content-type": "application/json",
        },
        body: JSON.stringify({
          owner_account: ownerAccount,
          device_id: "alice-phone",
          envelope_id: "env-attachment",
        }),
      }),
      env,
    );
    const json = (await response.json()) as {
      deleted: boolean;
      deleted_attachment_objects: number;
    };

    expect(json.deleted).toBe(true);
    expect(json.deleted_attachment_objects).toBe(2);
    expect(deletedEnvelopeIds).toEqual(["env-attachment"]);
    expect(deletedR2Keys).toEqual([manifestObjectKey, chunkObjectKey]);
  });

  it("accepts a valid P-256 Chat device binding", async () => {
    const fixture = await createChatBindingFixture();

    const response = await registerChatDevice(
      chatBindingRequest(fixture.body),
      fixture.env,
    );
    const json = (await response.json()) as {
      ok: boolean;
      owner_account: string;
      binding_message: string;
    };

    expect(json.ok).toBe(true);
    expect(json.owner_account).toBe(ownerAccount);
    expect(json.binding_message).toBe(
      buildChatDeviceBindingMessageBase64Url(fixture.input),
    );
    expect(fixture.db.devices.has(`${ownerAccount}:alice-phone`)).toBe(true);
  });

  it("rejects a client-supplied owner even when it matches the session", async () => {
    const fixture = await createChatBindingFixture();

    await expect(
      registerChatDevice(
        chatBindingRequest({
          ...fixture.body,
          owner_account: ownerAccount,
        }),
        fixture.env,
      ),
    ).rejects.toMatchObject({
      status: 400,
      code: "legacy_owner_account",
    } satisfies Partial<HttpError>);
  });

  it("rejects an invalid P-256 binding signature before storing device", async () => {
    const fixture = await createChatBindingFixture();

    await expect(
      registerChatDevice(
        chatBindingRequest({
          ...fixture.body,
          binding_signature: "00".repeat(64),
        }),
        fixture.env,
      ),
    ).rejects.toMatchObject({
      status: 401,
      code: "invalid_device_binding_signature",
    } satisfies Partial<HttpError>);
  });

  it("rejects an expired binding before consuming its nonce", async () => {
    const fixture = await createChatBindingFixture({
      expiresAt: Date.now() - 1,
    });

    await expect(
      registerChatDevice(chatBindingRequest(fixture.body), fixture.env),
    ).rejects.toMatchObject({
      status: 400,
      code: "expired_device_binding",
    } satisfies Partial<HttpError>);
    expect(fixture.db.nonces.size).toBe(0);
  });

  it("rejects replay of an already-consumed binding nonce", async () => {
    const fixture = await createChatBindingFixture();

    await registerChatDevice(chatBindingRequest(fixture.body), fixture.env);
    await expect(
      registerChatDevice(chatBindingRequest(fixture.body), fixture.env),
    ).rejects.toMatchObject({
      status: 409,
      code: "replayed_device_binding",
    } satisfies Partial<HttpError>);
  });

  it("rejects a valid legacy sr25519 wallet binding signature", async () => {
    const walletPair = sr25519PairFromSeed(new Uint8Array(32).fill(7));
    const legacyOwner = encodeAddress(walletPair.publicKey);
    const fixture = await createChatBindingFixture({ owner: legacyOwner });
    const legacySignature = toHex(
      sr25519Sign(buildChatDeviceBindingMessage(fixture.input), walletPair),
    );

    await expect(
      registerChatDevice(
        chatBindingRequest({
          ...fixture.body,
          binding_signature: legacySignature,
        }),
        fixture.env,
      ),
    ).rejects.toMatchObject({
      status: 401,
      code: "invalid_device_binding_signature",
    } satisfies Partial<HttpError>);
  });

  it("prepares, stores, and completes encrypted chat attachment objects", async () => {
    const env = fakeEnv({ withDevice: true, withR2: true });
    const prepareResponse = await prepareChatAttachmentUpload(
      new Request("https://worker.example/v1/chat/attachments/prepare", {
        method: "POST",
        headers: {
          authorization: "Bearer test-session",
          "content-type": "application/json",
        },
        body: JSON.stringify({
          owner_account: ownerAccount,
          device_id: "alice-phone",
          conversation_id: "conv-attachment",
          attachment_id: "att-123456",
          manifest_byte_size: 32,
          chunks: [{ chunk_id: "chunk-001", byte_size: 64 }],
        }),
      }),
      env,
    );
    const prepareJson = (await prepareResponse.json()) as {
      manifest_object_key: string;
      manifest_upload_url: string;
      chunks: Array<{ object_key: string; upload_url: string }>;
    };

    expect(prepareJson.manifest_object_key).toContain("/manifest.enc");
    expect(prepareJson.manifest_upload_url).toContain(
      "/v1/chat/attachments/dev-put",
    );
    await devPutChatAttachmentObject(
      new Request(prepareJson.manifest_upload_url, {
        method: "PUT",
        headers: {
          authorization: "Bearer test-session",
          "content-type": "application/octet-stream",
        },
        body: new Uint8Array([1, 2, 3]),
      }),
      env,
    );
    await devPutChatAttachmentObject(
      new Request(prepareJson.chunks[0].upload_url, {
        method: "PUT",
        headers: {
          authorization: "Bearer test-session",
          "content-type": "application/octet-stream",
        },
        body: new Uint8Array([4, 5, 6]),
      }),
      env,
    );

    const completeResponse = await completeChatAttachmentUpload(
      new Request("https://worker.example/v1/chat/attachments/complete", {
        method: "POST",
        headers: {
          authorization: "Bearer test-session",
          "content-type": "application/json",
        },
        body: JSON.stringify({
          owner_account: ownerAccount,
          device_id: "alice-phone",
          conversation_id: "conv-attachment",
          attachment_id: "att-123456",
          manifest_object_key: prepareJson.manifest_object_key,
          manifest_hash: "a".repeat(64),
          chunk_refs: [prepareJson.chunks[0].object_key],
        }),
      }),
      env,
    );
    const completeJson = (await completeResponse.json()) as {
      storage_state: string;
      chunk_refs: string[];
    };

    expect(completeJson.storage_state).toBe("completed");
    expect(completeJson.chunk_refs).toEqual([prepareJson.chunks[0].object_key]);

    const downloadResponse = await prepareChatAttachmentDownload(
      new Request("https://worker.example/v1/chat/attachments/download", {
        method: "POST",
        headers: {
          authorization: "Bearer test-session",
          "content-type": "application/json",
        },
        body: JSON.stringify({
          owner_account: ownerAccount,
          device_id: "alice-phone",
          conversation_id: "conv-attachment",
          attachment_id: "att-123456",
          manifest_object_key: prepareJson.manifest_object_key,
          manifest_hash: "a".repeat(64),
          chunk_refs: [prepareJson.chunks[0].object_key],
        }),
      }),
      env,
    );
    const downloadJson = (await downloadResponse.json()) as {
      manifest_download_url: string;
      chunks: Array<{ download_url: string }>;
    };

    expect(downloadJson.manifest_download_url).toContain(
      "/v1/chat/attachments/dev-get",
    );
    const manifestGetResponse = await devGetChatAttachmentObject(
      new Request(downloadJson.manifest_download_url, {
        headers: { authorization: "Bearer test-session" },
      }),
      env,
    );
    const chunkGetResponse = await devGetChatAttachmentObject(
      new Request(downloadJson.chunks[0].download_url, {
        headers: { authorization: "Bearer test-session" },
      }),
      env,
    );

    expect(Array.from(new Uint8Array(await manifestGetResponse.arrayBuffer())))
      .toEqual([1, 2, 3]);
    expect(Array.from(new Uint8Array(await chunkGetResponse.arrayBuffer())))
      .toEqual([4, 5, 6]);
  });
});

interface ChatBindingRequestBody {
  device_id: string;
  device_public_key_hex: string;
  binding_signature: string;
  expires_at: number;
  nonce: string;
  owner_account?: string;
}

class ChatBindingStatement {
  private values: unknown[] = [];

  constructor(
    private readonly db: ChatBindingDatabase,
    private readonly sql: string,
  ) {}

  bind(...values: unknown[]): ChatBindingStatement {
    this.values = values;
    return this;
  }

  async first<T>(): Promise<T | null> {
    if (this.sql.includes("FROM square_device_subkeys")) {
      return { p256_pubkey: this.db.p256PublicKeyHex } as T;
    }
    if (this.sql.includes("FROM chat_device_binding_nonces")) {
      const key = `${String(this.values[0])}:${String(this.values[1])}`;
      return this.db.nonces.has(key) ? ({ used: 1 } as T) : null;
    }
    return null;
  }

  async run(): Promise<{ meta: { changes: number } }> {
    if (this.sql.includes("DELETE FROM chat_device_binding_nonces")) {
      const now = Number(this.values[0]);
      for (const [key, expiresAt] of this.db.nonces.entries()) {
        if (expiresAt <= now) {
          this.db.nonces.delete(key);
        }
      }
      return { meta: { changes: 0 } };
    }
    if (this.sql.includes("INSERT INTO chat_device_binding_nonces")) {
      const key = `${String(this.values[0])}:${String(this.values[1])}`;
      if (this.db.nonces.has(key)) {
        throw new Error("UNIQUE constraint failed: chat_device_binding_nonces");
      }
      this.db.nonces.set(key, Number(this.values[2]));
      return { meta: { changes: 1 } };
    }
    if (this.sql.includes("INSERT INTO chat_devices")) {
      const key = `${String(this.values[0])}:${String(this.values[1])}`;
      this.db.devices.set(key, {
        device_public_key_hex: String(this.values[2]),
        binding_signature: String(this.values[3]),
      });
      return { meta: { changes: 1 } };
    }
    return { meta: { changes: 0 } };
  }
}

class ChatBindingDatabase {
  readonly nonces = new Map<string, number>();
  readonly devices = new Map<
    string,
    { device_public_key_hex: string; binding_signature: string }
  >();

  constructor(readonly p256PublicKeyHex: string) {}

  prepare(sql: string): ChatBindingStatement {
    return new ChatBindingStatement(this, sql);
  }
}

function chatBindingRequest(body: ChatBindingRequestBody): Request {
  return new Request("https://worker.example/v1/chat/devices/register", {
    method: "POST",
    headers: {
      authorization: "Bearer test-session",
      "content-type": "application/json",
    },
    body: JSON.stringify(body),
  });
}

async function createChatBindingFixture(options?: {
  owner?: string;
  expiresAt?: number;
}): Promise<{
  env: Env;
  db: ChatBindingDatabase;
  input: {
    owner_account: string;
    device_id: string;
    device_public_key_hex: string;
    expires_at: number;
    nonce: string;
  };
  body: ChatBindingRequestBody;
}> {
  const owner = options?.owner ?? ownerAccount;
  const expiresAt = options?.expiresAt ?? Date.now() + 60_000;
  const keyPair = await crypto.subtle.generateKey(
    { name: "ECDSA", namedCurve: "P-256" },
    true,
    ["sign", "verify"],
  );
  const p256PublicKeyHex = toHex(
    await crypto.subtle.exportKey("raw", keyPair.publicKey),
  );
  const db = new ChatBindingDatabase(p256PublicKeyHex);
  const session: SessionState = {
    owner_account: owner,
    created_at: Date.now(),
    expires_at: Date.now() + 60_000,
  };
  const env = {
    DB: db,
    SQUARE_CACHE: {
      get: async () => session,
    },
  } as unknown as Env;
  const input = {
    owner_account: owner,
    device_id: "alice-phone",
    device_public_key_hex: "aabbcc",
    expires_at: expiresAt,
    nonce: "nonce-123456",
  };
  const bindingSignature = toHex(
    await crypto.subtle.sign(
      { name: "ECDSA", hash: "SHA-256" },
      keyPair.privateKey,
      buildChatDeviceBindingMessage(input),
    ),
  );
  return {
    env,
    db,
    input,
    body: {
      device_id: input.device_id,
      device_public_key_hex: input.device_public_key_hex,
      binding_signature: bindingSignature,
      expires_at: input.expires_at,
      nonce: input.nonce,
    },
  };
}

function fakeEnv(options?: { withDevice?: boolean; withR2?: boolean }): Env {
  const session: SessionState = {
    owner_account: ownerAccount,
    created_at: Date.now(),
    expires_at: Date.now() + 60_000,
  };
  const r2Objects = new Map<string, ArrayBuffer>();
  return {
    DB: options?.withDevice
      ? ({
          prepare: (sql: string) => ({
            bind: () => ({
              first: async () =>
                sql.includes("FROM chat_envelopes")
                  ? ({ envelope_id: "env-attachment" })
                  : {
                      owner_account: ownerAccount,
                      device_id: "alice-phone",
                      device_public_key_hex: "aabbcc",
                      expires_at: Date.now() + 60_000,
                    },
            }),
          }),
        } as unknown as D1Database)
      : ({} as D1Database),
    SQUARE_MEDIA: options?.withR2
      ? ({
          put: async (key: string, value: ArrayBuffer) => {
            r2Objects.set(key, value);
            return null;
          },
          head: async (key: string) =>
            r2Objects.has(key) ? ({ key } as unknown as R2Object) : null,
          get: async (key: string) => {
            const value = r2Objects.get(key);
            if (!value) {
              return null;
            }
            return {
              body: new Response(value).body,
              writeHttpMetadata: (headers: Headers) => {
                headers.set("content-type", "application/octet-stream");
              },
            } as unknown as R2ObjectBody;
          },
        } as unknown as R2Bucket)
      : ({} as R2Bucket),
    SQUARE_CACHE: {
      get: async () => session,
    } as unknown as KVNamespace,
    DEV_UPLOAD_PROXY: "1",
  };
}
