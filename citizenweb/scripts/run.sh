#!/usr/bin/env bash
# 官网本地启动:自动安装缺失依赖后启动 vite dev server。
# 用法:cd ~/GMB && ./citizenweb/scripts/run.sh
#
# 默认端口 5199(固定端口,被占用时明确报错退出,不静默换端口);
# 可用环境变量覆盖:CITIZENWEB_PORT=5299 ./citizenweb/scripts/run.sh
# 生产构建产物预览请直接用:cd citizenweb && npm run build && npm run preview
set -euo pipefail

CITIZENWEB_PORT="${CITIZENWEB_PORT:-5199}"
DEV_PID=""

cleanup() {
    echo ""
    echo "==> 正在关闭官网 dev server..."
    [ -n "$DEV_PID" ] && kill "$DEV_PID" 2>/dev/null || true
    lsof -ti:"$CITIZENWEB_PORT" 2>/dev/null | xargs kill -9 2>/dev/null || true
    echo "    已关闭"
}
trap cleanup EXIT INT TERM HUP

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
CITIZENWEB_ROOT="$(dirname "$SCRIPT_DIR")"   # citizenweb/

# package.json engines 要求 node >=24,版本不够 vite 会以晦涩方式挂掉,这里提前明确报错。
NODE_MAJOR="$(node -v 2>/dev/null | sed 's/^v\([0-9]*\).*/\1/')"
if [ -z "$NODE_MAJOR" ]; then
    echo "[error] 未找到 node,请先安装 Node.js >= 24。" >&2
    exit 1
fi
if [ "$NODE_MAJOR" -lt 24 ]; then
    echo "[error] node 版本过低(当前 $(node -v),需要 >= 24)。" >&2
    exit 1
fi

cd "$CITIZENWEB_ROOT"

if [ ! -d node_modules ]; then
    echo "==> 首次启动,安装依赖(npm ci)..."
    npm ci
fi

echo "==> 启动官网 dev server: http://localhost:$CITIZENWEB_PORT"
# 后台启动 + wait:保证收到 INT/TERM 时 trap 能立即执行清理,
# 而不是等 npm 前台进程自己退出(直接 kill 脚本 PID 的场景也能干净收尾)。
npm run dev -- --port "$CITIZENWEB_PORT" --strictPort &
DEV_PID=$!
wait "$DEV_PID"
