#!/usr/bin/env bash
set -euo pipefail

# 中文注释：Worker 部署和真实测试都从控制台进程环境读取 Keychain 注入值，不读写明文 Secret 文件。
environment="${1:?缺少环境}"
secret_names=(
  CF_ACCOUNT_ID CF_API_TOKEN CHAIN_ID CHAIN_SECRET CHAIN_URL
  FCM_EMAIL FCM_KEY FCM_PROJECT HASH_KEY IMAGES_SIGNING_KEY
  R2_ACCESS_ID R2_SECRET_KEY STREAM_HOOK_SECRET STRIPE_API_KEY
  STRIPE_HOOK_SECRET TURNSTILE_SECRET
)

run_membership_test() {
  local cloudflared="$GMB_ROOT/deploy/.runtime/cloudflared"
  local base_url='https://www.crcfrcn.com/api-staging'
  local access_url="$base_url/health"
  local access_token health_response health_status

  echo '[步骤 1] 检查会员真实测试环境'
  [[ -x "$cloudflared" ]] || { echo '缺少 deploy/.runtime/cloudflared，请先由部署控制台安装官方客户端' >&2; exit 1; }
  [[ -x "${GMB_NODE_BIN:-}" ]] || { echo '部署控制台未传入可执行 Node 路径' >&2; exit 1; }
  [[ -x "${GMB_NPX_BIN:-}" ]] || { echo '部署控制台未传入可执行 npx 路径' >&2; exit 1; }
  [[ -n "${CF_ACCOUNT_ID:-}" ]] || { echo '缺少 CF_ACCOUNT_ID' >&2; exit 1; }
  [[ -n "${CF_API_TOKEN:-}" ]] || { echo '缺少 CF_API_TOKEN' >&2; exit 1; }
  [[ "${STRIPE_API_KEY:-}" == sk_test_* ]] || { echo '会员真实测试只允许使用 Stripe 测试密钥' >&2; exit 1; }

  echo '[步骤 2] 取得 Cloudflare Access 登录态'
  # 中文注释：Access 应用按 /api-staging/* 建立，登录发现必须使用真正受保护的具体路径。
  access_token="$($cloudflared access token "$access_url" 2>/dev/null || true)"
  health_response=""
  if [[ -n "$access_token" ]]; then
    health_response="$(curl --silent --show-error --max-time 20 -w $'\n%{http_code}' \
      -H "cf-access-token: $access_token" "$base_url/health" || true)"
  fi
  health_status="${health_response##*$'\n'}"
  if [[ -z "$access_token" || "$health_status" != '200' ]]; then
    echo '当前没有有效 Access 会话，正在打开系统浏览器登录窗口；登录完成后测试会自动继续。'
    # 中文注释：cloudflared 会自行打开浏览器；隐藏带一次性转移令牌的回退 URL，禁止进入控制台日志。
    if ! access_token="$($cloudflared access login --no-verbose --auto-close "$access_url" 2>/dev/null)"; then
      echo 'Cloudflare Access 登录未完成或已超时' >&2
      exit 1
    fi
  fi
  [[ "$access_token" == *.*.* ]] || { echo 'Cloudflare Access 未返回有效 JWT' >&2; exit 1; }
  export CF_ACCESS_TOKEN="$access_token"

  echo '[步骤 3] 验证 Access 后的真实 staging 健康接口'
  health_response="$(curl --silent --show-error --max-time 20 -w $'\n%{http_code}' \
    -H "cf-access-token: $CF_ACCESS_TOKEN" "$base_url/health")"
  health_status="${health_response##*$'\n'}"
  [[ "$health_status" == '200' ]] || { echo "staging 健康接口返回 HTTP $health_status" >&2; exit 1; }

  echo '[步骤 4] 执行真实 API、sr25519、staging D1 与 Stripe 测试矩阵'
  cd "$GMB_ROOT/citizenapp/cloudflare"
  BASE_URL="$base_url" "$GMB_NODE_BIN" --input-type=module <<'NODE'
import { spawnSync } from 'node:child_process';
import {
  blake2AsU8a,
  cryptoWaitReady,
  encodeAddress,
  randomAsU8a,
  sr25519PairFromSeed,
  sr25519Sign,
} from '@polkadot/util-crypto';
import { hexToU8a, u8aToHex } from '@polkadot/util';

const BASE = process.env.BASE_URL;
const ACCESS = process.env.CF_ACCESS_TOKEN;
const STRIPE_KEY = process.env.STRIPE_API_KEY;
const DAY_MS = 86_400_000;
const PRICE = {
  freedom: 'price_1Tswh9HSzSYWD2rFbguAbpmW',
  democracy: 'price_1TswhAHSzSYWD2rFN5JwqC83',
};
const results = [];
const stripeResources = { subscriptions: new Set(), customers: new Set(), checkouts: new Set() };
let failures = 0;
let owner;
let pair;

function compact(value) {
  return String(value ?? '').replace(/\s+/g, ' ').slice(0, 260);
}

function record(id, state, detail) {
  results.push({ id, state, detail: compact(detail) });
  if (state === 'FAIL') failures += 1;
  console.log(`[${state}] ${id} ${compact(detail)}`);
}

function errorCode(response) {
  return response?.json?.error_code ?? response?.json?.code ?? '';
}

async function post(path, body) {
  const response = await fetch(`${BASE}${path}`, {
    method: 'POST',
    headers: {
      'content-type': 'application/json',
      'cf-access-token': ACCESS,
    },
    body: JSON.stringify(body),
  });
  const text = await response.text();
  let json = null;
  try { json = JSON.parse(text); } catch {}
  return { status: response.status, json, text };
}

function signingMessage(payload) {
  return blake2AsU8a(new Uint8Array([0x47, 0x4d, 0x42, 0x1d, ...payload]), 256);
}

function signature(challenge) {
  const message = signingMessage(hexToU8a(challenge.signing_payload_hex));
  return u8aToHex(sr25519Sign(message, pair));
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

async function challenge(kind, fields = {}) {
  return post(operations[kind].challenge, { owner_account: owner, ...fields });
}

async function signedConfirm(kind, fields = {}) {
  const first = await challenge(kind, fields);
  if (first.status !== 200 || !first.json?.signing_payload_hex) return { challenge: first, confirm: null };
  const confirm = await post(operations[kind].confirm, {
    owner_account: owner,
    ...fields,
    challenge_id: first.json.challenge_id,
    signature: signature(first.json),
  });
  return { challenge: first, confirm };
}

function sqlText(value) {
  return `'${String(value).replaceAll("'", "''")}'`;
}

function d1(sql) {
  // 中文注释：部署低权限令牌无 D1 权限；真实测试只在本机控制台内复用 Wrangler OAuth 登录态播种和清理 staging。
  const wranglerEnv = { ...process.env };
  delete wranglerEnv.CF_ACCOUNT_ID;
  delete wranglerEnv.CF_API_TOKEN;
  delete wranglerEnv.CLOUDFLARE_ACCOUNT_ID;
  delete wranglerEnv.CLOUDFLARE_API_TOKEN;
  const result = spawnSync(
    process.env.GMB_NPX_BIN,
    ['wrangler', 'd1', 'execute', 'DB', '--env', 'staging', '--remote', '--command', sql, '--json'],
    { cwd: process.cwd(), env: wranglerEnv, encoding: 'utf8', timeout: 60_000, maxBuffer: 8 * 1024 * 1024 },
  );
  if (result.status !== 0) throw new Error(`staging D1 执行失败：${compact(result.stderr || result.stdout)}`);
  const parsed = JSON.parse(result.stdout);
  if (!Array.isArray(parsed) || parsed.some((item) => item.success !== true)) {
    throw new Error(`staging D1 返回失败：${compact(result.stdout)}`);
  }
  return parsed[0]?.results ?? [];
}

function clearD1() {
  d1(`DELETE FROM square_login_challenges WHERE owner_account = ${sqlText(owner)}; DELETE FROM square_memberships WHERE owner_account = ${sqlText(owner)};`);
}

function seedPrepaid(level, expiresAt) {
  const now = Date.now();
  d1(`INSERT OR REPLACE INTO square_memberships (
    owner_account, membership_level, expires_at, updated_at, subscription_source,
    stripe_customer_id, stripe_subscription_id, stripe_price_id, subscription_status,
    current_period_start, current_period_end, cancel_at_period_end, identity_level,
    identity_checked_at, entitlement_lapsed_at, frozen_at, collection_paused, prepaid_payment_ref
  ) VALUES (
    ${sqlText(owner)}, ${sqlText(level)}, ${expiresAt}, ${now}, 'usdc_prepaid',
    NULL, NULL, NULL, 'active', ${now}, ${expiresAt}, 0, 'visitor', ${now}, NULL, NULL, 0, 'audit_seed'
  );`);
}

function seedStripe(subscription) {
  const item = subscription.items?.data?.[0] ?? {};
  const periodStart = (subscription.current_period_start ?? item.current_period_start) * 1000;
  const periodEnd = (subscription.current_period_end ?? item.current_period_end) * 1000;
  d1(`INSERT OR REPLACE INTO square_memberships (
    owner_account, membership_level, expires_at, updated_at, subscription_source,
    stripe_customer_id, stripe_subscription_id, stripe_price_id, subscription_status,
    current_period_start, current_period_end, cancel_at_period_end, identity_level,
    identity_checked_at, entitlement_lapsed_at, frozen_at, collection_paused, prepaid_payment_ref
  ) VALUES (
    ${sqlText(owner)}, 'freedom', ${periodEnd}, ${Date.now()}, 'stripe',
    ${sqlText(subscription.customer)}, ${sqlText(subscription.id)}, ${sqlText(PRICE.freedom)}, ${sqlText(subscription.status)},
    ${periodStart}, ${periodEnd}, 0, 'visitor', ${Date.now()}, NULL, NULL, 0, NULL
  );`);
}

async function stripe(method, path, fields = null) {
  const options = { method, headers: { authorization: `Bearer ${STRIPE_KEY}` } };
  if (fields) {
    options.headers['content-type'] = 'application/x-www-form-urlencoded';
    options.body = new URLSearchParams(fields);
  }
  const response = await fetch(`https://api.stripe.com/v1${path}`, options);
  const text = await response.text();
  let json = {};
  try { json = JSON.parse(text); } catch {}
  if (!response.ok) throw new Error(`Stripe ${method} ${path} 返回 ${response.status}：${compact(json.error?.message || text)}`);
  return json;
}

async function createStripeFixture() {
  const customer = await stripe('POST', '/customers', {
    description: `GMB staging membership audit ${new Date().toISOString()}`,
    // 中文注释：测试账户不提供全局 pm_card_visa，使用 Stripe 官方 tok_visa 为本轮客户创建隔离测试卡。
    source: 'tok_visa',
  });
  stripeResources.customers.add(customer.id);
  const subscription = await stripe('POST', '/subscriptions', {
    customer: customer.id,
    'items[0][price]': PRICE.freedom,
    payment_behavior: 'error_if_incomplete',
    'metadata[owner_account]': owner,
    'metadata[membership_level]': 'freedom',
  });
  stripeResources.subscriptions.add(subscription.id);
  return subscription;
}

function rememberCheckout(response) {
  const direct = response?.json?.checkout_session_id;
  const fromUrl = response?.json?.checkout_url?.match(/(cs_(?:test|live)_[^#/?]+)/)?.[1];
  const id = direct || fromUrl;
  if (id) stripeResources.checkouts.add(id);
}

function expectHttp(id, response, status, code = null, extra = () => true) {
  const actualCode = errorCode(response);
  if (response?.status === status && (!code || actualCode === code) && extra(response.json)) {
    record(id, 'PASS', `HTTP ${response.status}${actualCode ? ` ${actualCode}` : ''}`);
  } else {
    record(id, 'FAIL', `期望 HTTP ${status}${code ? ` ${code}` : ''}，实测 HTTP ${response?.status ?? '无响应'} ${actualCode || compact(response?.text)}`);
  }
}

function stripeOutcome(id, response, blockedCode) {
  rememberCheckout(response);
  const code = errorCode(response);
  if (response?.status === 200) {
    record(id, 'PASS', `Worker 已真实完成 Stripe 操作：HTTP 200 ${response.json?.action ?? response.json?.checkout_session_id ?? ''}`);
  } else if (response?.status === 502 && code === blockedCode) {
    record(id, 'BLOCKED', `Worker→Stripe 出口实测失败：HTTP 502 ${code}；${response.json?.message ?? ''}`);
  } else {
    record(id, 'FAIL', `非预期结果：HTTP ${response?.status ?? '无响应'} ${code || compact(response?.text)}`);
  }
}

async function cleanupStripe() {
  const expected = stripeResources.checkouts.size + stripeResources.subscriptions.size + stripeResources.customers.size;
  let verified = 0;
  for (const id of stripeResources.checkouts) {
    try {
      const session = await stripe('POST', `/checkout/sessions/${encodeURIComponent(id)}/expire`);
      if (session.status !== 'expired') throw new Error(`Checkout ${id} 清理后状态=${session.status}`);
      verified += 1;
    } catch (error) {
      if (!String(error.message).includes('already expired')) record('清理-Checkout', 'FAIL', error.message);
      else verified += 1;
    }
  }
  for (const id of stripeResources.subscriptions) {
    try {
      const subscription = await stripe('DELETE', `/subscriptions/${encodeURIComponent(id)}`);
      if (subscription.status !== 'canceled') throw new Error(`Subscription ${id} 清理后状态=${subscription.status}`);
      verified += 1;
    } catch (error) {
      if (!String(error.message).includes('No such subscription')) record('清理-Subscription', 'FAIL', error.message);
      else verified += 1;
    }
  }
  for (const id of stripeResources.customers) {
    try {
      const customer = await stripe('DELETE', `/customers/${encodeURIComponent(id)}`);
      if (customer.deleted !== true) throw new Error(`Customer ${id} 未返回 deleted=true`);
      verified += 1;
    } catch (error) {
      if (!String(error.message).includes('No such customer')) record('清理-Customer', 'FAIL', error.message);
      else verified += 1;
    }
  }
  if (expected > 0 && verified === expected) {
    record('清理-Stripe', 'PASS', `已验证 ${verified} 个测试资源的取消、删除或过期响应`);
  }
}

async function run() {
  await cryptoWaitReady();
  pair = sr25519PairFromSeed(randomAsU8a(32));
  owner = encodeAddress(pair.publicKey, 2027);
  console.log(`测试钱包：${owner}（本轮临时随机钱包，不输出 seed）`);
  clearD1();

  let response = await challenge('subscribe', { membership_level: 'freedom' });
  expectHttp('A1 visitor 订阅 freedom 挑战', response, 200, null, (json) => json?.op_tag === 0x1d);
  response = await challenge('subscribe', { membership_level: 'voting' });
  expectHttp('A2 visitor 越级 voting', response, 403, 'membership_identity_mismatch');
  response = await challenge('subscribe', { membership_level: 'candidate' });
  expectHttp('A3 visitor 越级 candidate', response, 403, 'membership_identity_mismatch');
  response = await challenge('prepaid', { membership_level: 'freedom', duration: 'quarter' });
  expectHttp('A4 USDC freedom 季付挑战', response, 200, null, (json) => json?.op_tag === 0x1d && json?.months === 3);
  response = await challenge('prepaid', { membership_level: 'freedom', duration: 'month' });
  expectHttp('A5 USDC 非法时长', response, 400, 'invalid_prepaid_duration');

  clearD1();
  let signed = await signedConfirm('cancel');
  expectHttp('B1 无会员取消（真签名）', signed.confirm, 404, 'no_active_subscription');

  clearD1();
  seedPrepaid('freedom', Date.now() + 30 * DAY_MS);
  signed = await signedConfirm('cancel');
  expectHttp('B2 USDC 取消识别（真签名）', signed.confirm, 200, null, (json) => json?.cancel_kind === 'usdc_prepaid');

  clearD1();
  try {
    const fixture = await createStripeFixture();
    record('S0 本机→Stripe 测试环境', 'PASS', `已创建真实测试订阅 ${fixture.status}，用于验证 Worker 卡取消`);
    seedStripe(fixture);
    signed = await signedConfirm('cancel');
    stripeOutcome('B3 卡取消（真签名、真实 Stripe 订阅）', signed.confirm, 'stripe_cancel_failed');
    if (signed.confirm?.status === 200) {
      const remote = await stripe('GET', `/subscriptions/${encodeURIComponent(fixture.id)}`);
      const row = d1(`SELECT cancel_at_period_end FROM square_memberships WHERE owner_account = ${sqlText(owner)};`)[0];
      if (remote.cancel_at_period_end === true && row?.cancel_at_period_end === 1) {
        record('B3-落库', 'PASS', 'Stripe 与 staging D1 均为 cancel_at_period_end=true');
      } else {
        record('B3-落库', 'FAIL', `Stripe=${remote.cancel_at_period_end} D1=${row?.cancel_at_period_end}`);
      }
    }
  } catch (error) {
    record('B3-前置真实 Stripe fixture', 'FAIL', error.message);
  }

  clearD1();
  seedPrepaid('freedom', Date.now() + 30 * DAY_MS);
  response = await challenge('prepaid', { membership_level: 'democracy', duration: 'quarter' });
  expectHttp('C1 USDC 异档购买守卫', response, 409, 'prepaid_tier_change_required');
  response = await challenge('prepaid', { membership_level: 'freedom', duration: 'year' });
  expectHttp('C2 USDC 同档年付续费挑战', response, 200, null, (json) => json?.months === 12);
  response = await challenge('change', { membership_level: 'freedom' });
  expectHttp('D3 USDC 同档换档拒绝', response, 409, 'same_membership_level');
  response = await challenge('change', { membership_level: 'democracy' });
  expectHttp('D-升档预览', response, 200, null, (json) => json?.preview?.kind === 'upgrade' && json?.preview?.amount_cents === 667);

  clearD1();
  seedPrepaid('democracy', Date.now() + 30 * DAY_MS);
  response = await challenge('change', { membership_level: 'freedom' });
  expectHttp('D-降档预览', response, 200, null, (json) => json?.preview?.kind === 'downgrade' && json?.preview?.new_days === 97);
  signed = await signedConfirm('change', { membership_level: 'freedom' });
  expectHttp('D1 降档确认（真签名）', signed.confirm, 200, null, (json) => json?.action === 'downgraded' && json?.membership_level === 'freedom');
  if (signed.confirm?.status === 200) {
    const row = d1(`SELECT membership_level, expires_at FROM square_memberships WHERE owner_account = ${sqlText(owner)};`)[0];
    const expected = signed.confirm.json.expires_at;
    if (row?.membership_level === 'freedom' && Number(row?.expires_at) === Number(expected)) {
      record('D1-落库', 'PASS', 'staging D1 已即时切为 freedom，expires_at 与 API 返回一致');
    } else {
      record('D1-落库', 'FAIL', `D1 level=${row?.membership_level} expires_at=${row?.expires_at} API=${expected}`);
    }
  }

  clearD1();
  seedPrepaid('freedom', Date.now() + 30 * DAY_MS);
  signed = await signedConfirm('change', { membership_level: 'democracy' });
  stripeOutcome('D2 USDC 升档差价建单（真签名）', signed.confirm, 'stripe_checkout_failed');

  clearD1();
  signed = await signedConfirm('prepaid', { membership_level: 'freedom', duration: 'quarter' });
  stripeOutcome('E1 USDC 预付建单（真签名）', signed.confirm, 'stripe_checkout_failed');

  clearD1();
  signed = await signedConfirm('subscribe', { membership_level: 'freedom' });
  stripeOutcome('F1 卡订阅建单（真签名）', signed.confirm, 'stripe_checkout_failed');
}

try {
  await run();
} catch (error) {
  record('测试驱动', 'FAIL', error.stack ?? error.message);
} finally {
  try { if (owner) clearD1(); } catch (error) { record('清理-staging D1', 'FAIL', error.message); }
  await cleanupStripe();
  if (owner) {
    try {
      const rows = d1(`SELECT
        (SELECT COUNT(*) FROM square_login_challenges WHERE owner_account = ${sqlText(owner)}) AS challenge_count,
        (SELECT COUNT(*) FROM square_memberships WHERE owner_account = ${sqlText(owner)}) AS membership_count;`);
      const row = rows[0];
      if (Number(row?.challenge_count) === 0 && Number(row?.membership_count) === 0) {
        record('清理-D1', 'PASS', 'staging D1 挑战与会员测试行均为 0 残留');
      } else {
        record('清理验证', 'FAIL', `challenge=${row?.challenge_count} membership=${row?.membership_count}`);
      }
    } catch (error) { record('清理验证', 'FAIL', error.message); }
  }
}

const counts = results.reduce((map, item) => ({ ...map, [item.state]: (map[item.state] ?? 0) + 1 }), {});
console.log('');
console.log('========== 会员真实测试汇总 ==========');
console.log(`PASS=${counts.PASS ?? 0} BLOCKED=${counts.BLOCKED ?? 0} FAIL=${counts.FAIL ?? 0}`);
console.log('BLOCKED 表示真实请求已到达 Worker 的 Stripe 调用点，但 Worker→Stripe 出口失败；FAIL 表示逻辑、环境或清理不符合期望。');
if (failures > 0) process.exitCode = 1;
NODE
}

run_membership_e2e() {
  local cloudflared="$GMB_ROOT/deploy/.runtime/cloudflared"
  local base_url='https://www.crcfrcn.com/api-staging'
  local access_url="$base_url/health"
  local access_token health_response health_status

  echo '[步骤 1] 检查 Stripe Sandbox 全链路验收环境'
  [[ -x "$cloudflared" ]] || { echo '缺少 deploy/.runtime/cloudflared' >&2; exit 1; }
  [[ -x "${GMB_NODE_BIN:-}" ]] || { echo '部署控制台未传入可执行 Node 路径' >&2; exit 1; }
  [[ -x "${GMB_NPX_BIN:-}" ]] || { echo '部署控制台未传入可执行 npx 路径' >&2; exit 1; }
  [[ -n "${CF_ACCOUNT_ID:-}" && -n "${CF_API_TOKEN:-}" ]] || { echo '缺少 Cloudflare 配置' >&2; exit 1; }
  [[ "${STRIPE_API_KEY:-}" == sk_test_* ]] || { echo '全链路验收只允许 Stripe 测试密钥' >&2; exit 1; }
  [[ "${STRIPE_HOOK_SECRET:-}" == whsec_* ]] || { echo '缺少 Stripe Sandbox webhook secret' >&2; exit 1; }

  echo '[步骤 2] 取得 Cloudflare Access 登录态'
  access_token="$($cloudflared access token "$access_url" 2>/dev/null || true)"
  health_response=""
  if [[ -n "$access_token" ]]; then
    health_response="$(curl --silent --show-error --max-time 20 -w $'\n%{http_code}' \
      -H "cf-access-token: $access_token" "$base_url/health" || true)"
  fi
  health_status="${health_response##*$'\n'}"
  if [[ -z "$access_token" || "$health_status" != '200' ]]; then
    echo '当前没有有效 Access 会话，正在打开系统浏览器登录窗口；登录完成后验收自动继续。'
    access_token="$($cloudflared access login --no-verbose --auto-close "$access_url" 2>/dev/null)" || {
      echo 'Cloudflare Access 登录未完成或已超时' >&2
      exit 1
    }
  fi
  [[ "$access_token" == *.*.* ]] || { echo 'Cloudflare Access 未返回有效 JWT' >&2; exit 1; }
  export CF_ACCESS_TOKEN="$access_token"

  echo '[步骤 3] 验证 Access 后的真实 staging 健康接口'
  curl --fail --silent --show-error --max-time 20 \
    -H "cf-access-token: $CF_ACCESS_TOKEN" "$base_url/health" >/dev/null

  echo '[步骤 4] 启动真实卡、Stripe Crypto、webhook、D1、切换与取消全链路'
  cd "$GMB_ROOT/citizenapp/cloudflare"
  BASE_URL="$base_url" "$GMB_NODE_BIN" \
    "$GMB_ROOT/deploy/actions/cloudflare-membership-e2e.mjs"
}

if [[ "$environment" == 'membership-test' ]]; then
  run_membership_test
  exit 0
fi

if [[ "$environment" == 'membership-e2e' ]]; then
  run_membership_e2e
  exit 0
fi

case "$environment" in
  staging) health_url='https://www.crcfrcn.com/api-staging/health'; expected_prefix='sk_test_' ;;
  production) health_url='https://www.crcfrcn.com/api/health'; expected_prefix='sk_live_' ;;
  *) exit 2 ;;
esac
for secret_name in "${secret_names[@]}"; do
  [[ -n "${!secret_name:-}" ]] || { echo "缺少 ${secret_name}" >&2; exit 1; }
done
echo '[步骤 1] 检查部署环境和 Keychain 密钥'
[[ "$CHAIN_URL" == https://* ]] || { echo 'CHAIN_URL 必须使用 HTTPS' >&2; exit 1; }
[[ "$STRIPE_API_KEY" == "$expected_prefix"* ]] || { echo 'Stripe 环境与部署环境不匹配' >&2; exit 1; }

cd "$GMB_ROOT/citizenapp/cloudflare"
echo '[步骤 2] 安装锁定版本依赖'
npm ci
echo '[步骤 3] 执行 TypeScript 检查'
npm run typecheck
echo '[步骤 4] 执行 Worker 自动化测试'
npm test
echo '[步骤 5] 检查远端 D1 数据库和迁移状态'
# D1 管理固定复用本机 Wrangler OAuth；部署低权限 API Token 不具备 D1 权限，不能混用。
oauth_wrangler=(env -u CF_ACCOUNT_ID -u CF_API_TOKEN -u CLOUDFLARE_ACCOUNT_ID -u CLOUDFLARE_API_TOKEN npx wrangler)
"${oauth_wrangler[@]}" d1 execute DB --env "$environment" --remote --command 'SELECT 1 AS ready;' >/dev/null
if [[ "$environment" == 'staging' ]]; then
  # 本任务仅批准 staging 应用 Stripe webhook 幂等增量；SQL 使用 IF NOT EXISTS，可安全重跑。
  "${oauth_wrangler[@]}" d1 execute DB --env staging --remote \
    --file migrations/0002_stripe_webhook_events.sql >/dev/null
  "${oauth_wrangler[@]}" d1 execute DB --env staging --remote --command \
    "SELECT 1 FROM square_stripe_webhook_events LIMIT 1; SELECT 1 FROM square_stripe_payments LIMIT 1;" \
    >/dev/null
else
  pending="$("${oauth_wrangler[@]}" d1 migrations list DB --env "$environment" --remote 2>&1 || true)"
  if printf '%s\n' "$pending" | grep -E '0002_stripe_webhook_events\.sql' >/dev/null; then
    echo '生产 D1 存在未获本轮授权的 Stripe 增量迁移，停止部署' >&2
    exit 1
  fi
fi
echo '[步骤 6] 将 Keychain 密钥同步到目标 Worker'
for secret_name in "${secret_names[@]}"; do
  printf '%s' "${!secret_name}" | "${oauth_wrangler[@]}" secret put "$secret_name" --env "$environment" >/dev/null
  echo "已同步 ${environment} Secret: ${secret_name}"
done
echo '[步骤 7] 发布 Cloudflare Worker'
"${oauth_wrangler[@]}" deploy --env "$environment"
echo '[步骤 8] 检查真实健康接口'
curl --fail --silent --show-error "$health_url" >/dev/null
echo "CitizenApp Cloudflare ${environment} 部署与真实健康检查完成"
