import { createHmac } from 'node:crypto';
import { spawnSync } from 'node:child_process';
import { createRequire } from 'node:module';

const requireFromWorker = createRequire(
  `${process.env.GMB_ROOT}/citizenapp/cloudflare/package.json`,
);
const {
  blake2AsU8a,
  cryptoWaitReady,
  encodeAddress,
  randomAsU8a,
  sr25519PairFromSeed,
  sr25519Sign,
} = requireFromWorker('@polkadot/util-crypto');
const { hexToU8a, u8aToHex } = requireFromWorker('@polkadot/util');

const BASE_URL = process.env.BASE_URL ?? 'https://www.crcfrcn.com/api-staging';
const ACCESS_TOKEN = process.env.CF_ACCESS_TOKEN;
const STRIPE_KEY = process.env.STRIPE_API_KEY;
const STRIPE_HOOK_SECRET = process.env.STRIPE_HOOK_SECRET;
const NPX = process.env.GMB_NPX_BIN;
const EXPECTED_STRIPE_ACCOUNT = 'acct_1Trr2qQlQZ1x0Cw8';
// 加密钱包首次配置、领测试币和链上确认可能超过 15 分钟；控制台保持 30 分钟等待并在失败后自动清理。
const PAYMENT_TIMEOUT_MS = Number(process.env.GMB_E2E_PAYMENT_TIMEOUT_MS ?? 30 * 60 * 1000);
const POLL_MS = 2_000;

for (const [name, value] of Object.entries({
  CF_ACCESS_TOKEN: ACCESS_TOKEN,
  STRIPE_API_KEY: STRIPE_KEY,
  STRIPE_HOOK_SECRET,
  GMB_NPX_BIN: NPX,
})) {
  if (!value) throw new Error(`缺少 ${name}`);
}
if (!STRIPE_KEY.startsWith('sk_test_')) {
  throw new Error('全链路验收只允许使用 Stripe Sandbox 测试密钥');
}

await cryptoWaitReady();
const pair = sr25519PairFromSeed(randomAsU8a(32));
const ownerAccount = encodeAddress(pair.publicKey, 2027);
const stripeResources = {
  checkoutSessions: new Set(),
  subscriptions: new Set(),
  customers: new Set(),
  eventIds: new Set(),
};

function compact(value) {
  return String(value ?? '').replace(/\s+/g, ' ').slice(0, 360);
}

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

async function apiPost(path, body) {
  const response = await fetch(`${BASE_URL}${path}`, {
    method: 'POST',
    headers: {
      'content-type': 'application/json',
      'cf-access-token': ACCESS_TOKEN,
    },
    body: JSON.stringify(body),
  });
  const text = await response.text();
  let json = null;
  try {
    json = JSON.parse(text);
  } catch {}
  return { status: response.status, json, text };
}

async function stripeRequest(path, options = {}) {
  const response = await fetch(`https://api.stripe.com/v1${path}`, {
    ...options,
    headers: {
      authorization: `Bearer ${STRIPE_KEY}`,
      ...(options.headers ?? {}),
    },
  });
  const text = await response.text();
  let json = {};
  try {
    json = JSON.parse(text);
  } catch {}
  if (!response.ok) {
    throw new Error(`Stripe ${response.status}: ${compact(json?.error?.message ?? text)}`);
  }
  return json;
}

function d1(sql) {
  // 部署令牌不承担 D1 管理权限；测试驱动只复用本机 Wrangler OAuth，且仅操作 staging。
  const wranglerEnv = { ...process.env };
  delete wranglerEnv.CF_ACCOUNT_ID;
  delete wranglerEnv.CF_API_TOKEN;
  delete wranglerEnv.CLOUDFLARE_ACCOUNT_ID;
  delete wranglerEnv.CLOUDFLARE_API_TOKEN;
  const result = spawnSync(
    NPX,
    ['wrangler', 'd1', 'execute', 'DB', '--env', 'staging', '--remote', '--command', sql, '--json'],
    {
      cwd: `${process.env.GMB_ROOT}/citizenapp/cloudflare`,
      env: wranglerEnv,
      encoding: 'utf8',
      timeout: 60_000,
      maxBuffer: 8 * 1024 * 1024,
    },
  );
  if (result.status !== 0) {
    throw new Error(`staging D1 执行失败：${compact(result.stderr || result.stdout)}`);
  }
  const parsed = JSON.parse(result.stdout);
  if (!Array.isArray(parsed) || parsed.some((item) => item.success !== true)) {
    throw new Error(`staging D1 返回失败：${compact(result.stdout)}`);
  }
  return parsed[0]?.results ?? [];
}

function sqlText(value) {
  return `'${String(value).replaceAll("'", "''")}'`;
}

function signingMessage(payloadHex) {
  return blake2AsU8a(new Uint8Array([0x47, 0x4d, 0x42, 0x1d, ...hexToU8a(payloadHex)]), 256);
}

function signChallenge(challenge) {
  return u8aToHex(sr25519Sign(signingMessage(challenge.signing_payload_hex), pair));
}

const operations = {
  subscribe: {
    challenge: '/v1/square/membership/subscribe/challenge',
    confirm: '/v1/square/membership/subscribe',
  },
  prepaid: {
    challenge: '/v1/square/membership/prepaid/challenge',
    confirm: '/v1/square/membership/prepaid',
  },
  change: {
    challenge: '/v1/square/membership/prepaid/change/challenge',
    confirm: '/v1/square/membership/prepaid/change',
  },
  cancel: {
    challenge: '/v1/square/membership/cancel/challenge',
    confirm: '/v1/square/membership/cancel',
  },
};

async function signedOperation(kind, fields = {}) {
  const challenge = await apiPost(operations[kind].challenge, {
    owner_account: ownerAccount,
    ...fields,
  });
  if (challenge.status !== 200 || !challenge.json?.signing_payload_hex) {
    throw new Error(`${kind} challenge 失败：HTTP ${challenge.status} ${compact(challenge.text)}`);
  }
  const confirm = await apiPost(operations[kind].confirm, {
    owner_account: ownerAccount,
    ...fields,
    challenge_id: challenge.json.challenge_id,
    signature: signChallenge(challenge.json),
  });
  if (confirm.status !== 200) {
    throw new Error(`${kind} confirm 失败：HTTP ${confirm.status} ${compact(confirm.text)}`);
  }
  return confirm.json;
}

async function waitForCheckout(sessionId, paymentStates) {
  const deadline = Date.now() + PAYMENT_TIMEOUT_MS;
  while (Date.now() < deadline) {
    const session = await stripeRequest(`/checkout/sessions/${encodeURIComponent(sessionId)}?expand[]=subscription`);
    const subscription = typeof session.subscription === 'object' ? session.subscription : null;
    const customerId = typeof session.customer === 'string' ? session.customer : session.customer?.id;
    if (subscription?.id) stripeResources.subscriptions.add(subscription.id);
    if (customerId) stripeResources.customers.add(customerId);
    if (session.status === 'complete' && paymentStates.includes(session.payment_status)) {
      return session;
    }
    await sleep(POLL_MS);
  }
  throw new Error(`等待 Checkout ${sessionId} 完成超时`);
}

async function waitForMembership(predicate, description) {
  const deadline = Date.now() + 60_000;
  while (Date.now() < deadline) {
    const row = d1(
      `SELECT * FROM square_memberships WHERE owner_account = ${sqlText(ownerAccount)} LIMIT 1;`,
    )[0];
    if (row && predicate(row)) return row;
    await sleep(POLL_MS);
  }
  throw new Error(`等待 staging D1 ${description} 超时`);
}

async function findCheckoutEvent(sessionId) {
  const deadline = Date.now() + 60_000;
  while (Date.now() < deadline) {
    const events = await stripeRequest('/events?type=checkout.session.completed&limit=100');
    const event = events.data?.find((item) => item?.data?.object?.id === sessionId);
    if (event) {
      stripeResources.eventIds.add(event.id);
      return event;
    }
    await sleep(POLL_MS);
  }
  throw new Error(`未找到 Checkout ${sessionId} 的真实 Stripe webhook 事件`);
}

async function replayWebhook(event) {
  const timestamp = Math.floor(Date.now() / 1000);
  const raw = JSON.stringify(event);
  const signature = createHmac('sha256', STRIPE_HOOK_SECRET)
    .update(`${timestamp}.${raw}`)
    .digest('hex');
  const response = await fetch(`${BASE_URL}/v1/square/membership/webhook`, {
    method: 'POST',
    headers: {
      'content-type': 'application/json',
      'stripe-signature': `t=${timestamp},v1=${signature}`,
    },
    body: raw,
  });
  const json = await response.json().catch(() => ({}));
  if (!response.ok) throw new Error(`webhook 重放失败：HTTP ${response.status} ${compact(JSON.stringify(json))}`);
  return json;
}

async function cleanup() {
  for (const subscriptionId of stripeResources.subscriptions) {
    try {
      await stripeRequest(`/subscriptions/${encodeURIComponent(subscriptionId)}`, { method: 'DELETE' });
    } catch {}
  }
  for (const checkoutId of stripeResources.checkoutSessions) {
    try {
      const session = await stripeRequest(`/checkout/sessions/${encodeURIComponent(checkoutId)}`);
      if (session.status === 'open') {
        await stripeRequest(`/checkout/sessions/${encodeURIComponent(checkoutId)}/expire`, { method: 'POST' });
      }
    } catch {}
  }
  for (const customerId of stripeResources.customers) {
    try {
      await stripeRequest(`/customers/${encodeURIComponent(customerId)}`, { method: 'DELETE' });
    } catch {}
  }
  const eventIds = [...stripeResources.eventIds].map(sqlText).join(',');
  d1(`
    DELETE FROM square_login_challenges WHERE owner_account = ${sqlText(ownerAccount)};
    DELETE FROM square_memberships WHERE owner_account = ${sqlText(ownerAccount)};
    DELETE FROM square_stripe_payments WHERE owner_account = ${sqlText(ownerAccount)};
    ${eventIds ? `DELETE FROM square_stripe_webhook_events WHERE event_id IN (${eventIds});` : ''}
  `);
}

let completed = false;
try {
  const account = await stripeRequest('/account');
  if (account.id !== EXPECTED_STRIPE_ACCOUNT) {
    throw new Error(`staging 未连接专用 Sandbox：实际 ${account.id}`);
  }
  console.log(`[PASS] Sandbox 隔离：${account.id}（livemode=false）`);
  console.log(`[INFO] 本轮真实签名钱包：${ownerAccount}`);
  await cleanup();

  const cardCheckout = await signedOperation('subscribe', { membership_level: 'freedom' });
  stripeResources.checkoutSessions.add(cardCheckout.checkout_session_id);
  console.log(`[ACTION] 请在 Stripe 测试页完成银行卡 Checkout：${cardCheckout.checkout_url}`);
  const cardSession = await waitForCheckout(cardCheckout.checkout_session_id, ['paid']);
  const cardSubscriptionId = cardSession.subscription?.id ?? cardSession.subscription;
  if (!cardSubscriptionId) throw new Error('卡 Checkout 完成但缺少 subscription');
  stripeResources.subscriptions.add(cardSubscriptionId);
  const cardRow = await waitForMembership(
    (row) => row.subscription_source === 'stripe' && row.stripe_subscription_id === cardSubscriptionId,
    '卡订阅 webhook 授权',
  );
  console.log(`[PASS] 卡付款→subscription webhook→D1 授权：${cardRow.membership_level}`);

  const cancel = await signedOperation('cancel');
  if (cancel.cancel_kind !== 'stripe') throw new Error(`卡取消识别错误：${compact(JSON.stringify(cancel))}`);
  const canceledSubscription = await stripeRequest(`/subscriptions/${encodeURIComponent(cardSubscriptionId)}`);
  if (canceledSubscription.cancel_at_period_end !== true) throw new Error('Stripe 未设置到期取消');
  console.log('[PASS] 真签名取消→Stripe 到期取消');

  const cryptoCheckout = await signedOperation('prepaid', {
    membership_level: 'freedom',
    duration: 'quarter',
  });
  stripeResources.checkoutSessions.add(cryptoCheckout.checkout_session_id);
  console.log(`[ACTION] 请用 Stripe 测试网 USDC 完成 Crypto Checkout：${cryptoCheckout.checkout_url}`);
  const cryptoSession = await waitForCheckout(cryptoCheckout.checkout_session_id, ['paid']);
  if (!cryptoSession.payment_method_types?.includes('crypto')) throw new Error('USDC Checkout 未使用 crypto');
  const prepaidRow = await waitForMembership(
    (row) => row.subscription_source === 'usdc_prepaid' && row.membership_level === 'freedom',
    'USDC webhook 授时长',
  );
  const paymentCount = Number(
    d1(`SELECT COUNT(*) AS count FROM square_stripe_payments WHERE owner_account = ${sqlText(ownerAccount)};`)[0]?.count,
  );
  if (paymentCount !== 1) throw new Error(`USDC 付款凭证数量异常：${paymentCount}`);
  console.log('[PASS] Stripe Crypto USDC→checkout webhook→D1 预付授权');

  const event = await findCheckoutEvent(cryptoCheckout.checkout_session_id);
  const expiresBeforeReplay = prepaidRow.expires_at;
  const replayOne = await replayWebhook(event);
  const replayTwo = await replayWebhook(event);
  const afterReplay = d1(
    `SELECT expires_at FROM square_memberships WHERE owner_account = ${sqlText(ownerAccount)} LIMIT 1;`,
  )[0];
  if (
    replayOne.action !== 'stripe_event_duplicate' ||
    replayTwo.action !== 'stripe_event_duplicate' ||
    afterReplay?.expires_at !== expiresBeforeReplay
  ) {
    throw new Error('真实 webhook 重放改变了会员时长');
  }
  console.log('[PASS] 真实 Stripe event 连续重放两次，D1 时长不变');

  const cardSwitchCheckout = await signedOperation('subscribe', { membership_level: 'freedom' });
  stripeResources.checkoutSessions.add(cardSwitchCheckout.checkout_session_id);
  console.log(`[ACTION] 请完成 USDC→银行卡 Checkout：${cardSwitchCheckout.checkout_url}`);
  const cardSwitchSession = await waitForCheckout(cardSwitchCheckout.checkout_session_id, [
    'paid',
    'no_payment_required',
  ]);
  const cardSwitchSubscriptionId = cardSwitchSession.subscription?.id ?? cardSwitchSession.subscription;
  if (!cardSwitchSubscriptionId) throw new Error('USDC→卡 Checkout 缺少 subscription');
  stripeResources.subscriptions.add(cardSwitchSubscriptionId);
  await waitForMembership(
    (row) => row.subscription_source === 'stripe' && row.stripe_subscription_id === cardSwitchSubscriptionId,
    'USDC→卡 webhook 切换',
  );
  console.log('[PASS] USDC→卡完整切换，Stripe trial 与 D1 订阅均落地');

  await signedOperation('cancel');
  console.log('[PASS] 切换后的卡订阅真签名取消');
  completed = true;
} finally {
  await cleanup();
  const residual = d1(`SELECT
    (SELECT COUNT(*) FROM square_login_challenges WHERE owner_account = ${sqlText(ownerAccount)}) AS challenge_count,
    (SELECT COUNT(*) FROM square_memberships WHERE owner_account = ${sqlText(ownerAccount)}) AS membership_count,
    (SELECT COUNT(*) FROM square_stripe_payments WHERE owner_account = ${sqlText(ownerAccount)}) AS payment_count;`)[0];
  console.log(
    `[CLEANUP] challenge=${residual?.challenge_count ?? '?'} membership=${residual?.membership_count ?? '?'} payment=${residual?.payment_count ?? '?'}`,
  );
}

if (!completed) process.exitCode = 1;
else console.log('PASS=8 BLOCKED=0 FAIL=0');
