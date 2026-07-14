#!/usr/bin/env bash
set -euo pipefail

# 中文注释：Worker 部署从进程环境读取 Keychain 注入值，不读写任何明文 Secret 文件。
environment="${1:?缺少环境}"
secret_names=(
  CF_ACCOUNT_ID CF_API_TOKEN CHAIN_ID CHAIN_SECRET CHAIN_URL
  FCM_EMAIL FCM_KEY FCM_PROJECT HASH_KEY IMAGES_SIGNING_KEY
  R2_ACCESS_ID R2_SECRET_KEY STREAM_HOOK_SECRET STRIPE_API_KEY
  STRIPE_HOOK_SECRET TURNSTILE_SECRET
)
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
npx wrangler d1 execute DB --env "$environment" --remote --command 'SELECT 1 AS ready;' >/dev/null
pending="$(npx wrangler d1 migrations list DB --env "$environment" --remote 2>&1 || true)"
if printf '%s\n' "$pending" | grep -E '[0-9]{4}_.+\.sql' | grep -v '0001_square_core.sql' >/dev/null; then
  echo '发现未审核的增量 D1 迁移，停止部署' >&2
  exit 1
fi
echo '[步骤 6] 将 Keychain 密钥同步到目标 Worker'
for secret_name in "${secret_names[@]}"; do
  printf '%s' "${!secret_name}" | npx wrangler secret put "$secret_name" --env "$environment" >/dev/null
  echo "已同步 ${environment} Secret: ${secret_name}"
done
echo '[步骤 7] 发布 Cloudflare Worker'
npx wrangler deploy --env "$environment"
echo '[步骤 8] 检查真实健康接口'
curl --fail --silent --show-error "$health_url" >/dev/null
echo "CitizenApp Cloudflare ${environment} 部署与真实健康检查完成"
