#!/usr/bin/env bash
set -euo pipefail

# 中文注释：官网测试部署只启动本地构建网站；生产部署只更新现有 Cloudflare Pages 项目。
environment="${1:?缺少环境}"
deploy_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
runtime_dir="$deploy_dir/.runtime"
pid_file="$runtime_dir/citizenweb-test.pid"
log_file="$runtime_dir/citizenweb-test.log"
local_url='http://127.0.0.1:41732'
mkdir -p "$runtime_dir"
chmod 700 "$runtime_dir"

stop_local_site() {
  if [[ -f "$pid_file" ]]; then
    local pid
    pid="$(<"$pid_file")"
    if [[ "$pid" =~ ^[0-9]+$ ]] && kill -0 "$pid" 2>/dev/null; then
      kill "$pid"
      for _ in {1..30}; do
        kill -0 "$pid" 2>/dev/null || break
        sleep 0.1
      done
    fi
    rm -f "$pid_file"
  fi
}

case "$environment" in
  local-start)
    echo '[步骤 1] 关闭可能残留的本地测试网站'
    stop_local_site
    cd "$GMB_ROOT/citizenweb"
    echo '[步骤 2] 安装锁定版本依赖'
    npm ci
    echo '[步骤 3] 执行官网代码检查'
    npm run lint
    echo '[步骤 4] 生成官网正式构建产物'
    npm run build
    # 中文注释：用构建产物启动固定回环端口，PID 只写入被 Git 忽略的运行目录。
    nohup npx vite preview --host 127.0.0.1 --port 41732 --strictPort >"$log_file" 2>&1 &
    preview_pid=$!
    printf '%s\n' "$preview_pid" > "$pid_file"
    echo '[步骤 5] 启动本地测试网站并检查页面'
    for _ in {1..40}; do
      if curl --fail --silent --max-time 1 "$local_url" >/dev/null 2>&1; then
        echo "CitizenWeb 本地测试网站已启动：$local_url"
        exit 0
      fi
      kill -0 "$preview_pid" 2>/dev/null || { tail -n 20 "$log_file" >&2; exit 1; }
      sleep 0.25
    done
    echo 'CitizenWeb 本地测试网站启动超时' >&2
    exit 1
    ;;
  local-stop)
    echo '[步骤 1] 停止本地测试网站进程'
    stop_local_site
    echo '[步骤 2] 检查本地测试端口已经关闭'
    if curl --fail --silent --max-time 1 "$local_url" >/dev/null 2>&1; then
      echo '本地测试网站端口仍被其他进程占用' >&2
      exit 1
    fi
    echo 'CitizenWeb 本地测试网站已关闭'
    ;;
  production)
    echo '[步骤 1] 检查 Cloudflare 账户和 Wrangler 登录'
    [[ -n "${CF_ACCOUNT_ID:-}" ]] || { echo '缺少 CF_ACCOUNT_ID' >&2; exit 1; }
    export CLOUDFLARE_ACCOUNT_ID="$CF_ACCOUNT_ID"
    npx wrangler whoami >/dev/null || { echo '请先完成 Wrangler 登录' >&2; exit 1; }
    cd "$GMB_ROOT/citizenweb"
    echo '[步骤 2] 安装锁定版本依赖'
    npm ci
    echo '[步骤 3] 执行官网代码检查'
    npm run lint
    echo '[步骤 4] 生成官网正式构建产物'
    npm run build
    echo '[步骤 5] 确认现有 Cloudflare Pages 生产项目'
    npx wrangler pages project list | grep -Eq '^[│|] citizenweb ' || { echo '缺少现有生产 Pages 项目 citizenweb' >&2; exit 1; }
    echo '[步骤 6] 将最新构建部署到生产官网'
    npx wrangler pages deploy dist --project-name citizenweb --branch main
    echo '[步骤 7] 检查线上官网真实响应'
    curl --fail --silent --show-error 'https://www.crcfrcn.com' >/dev/null
    echo 'CitizenWeb 生产部署与真实健康检查完成'
    ;;
  *) exit 2 ;;
esac
