#!/usr/bin/env bash
set -euo pipefail
# shellcheck disable=SC1091
source "$(dirname "${BASH_SOURCE[0]}")/common.sh"
mode="${1:?缺少模式}"
[[ "$mode" == ci || "$mode" == release || "$mode" == deploy ]] || exit 2
require_clean_remote_commit
if [[ "$mode" != deploy ]]; then
  run_workflow citizenchain-ci.yml "$mode"
  exit 0
fi

validate_genesis_state_package() {
  local package_root="$1" path relative
  [[ -f "$package_root/manifest.json" && -d "$package_root/chains/citizenchain/db" ]] || {
    echo "正式创世状态包缺少 manifest.json 或链数据库:$package_root" >&2
    return 1
  }
  while IFS= read -r -d '' path; do
    relative="${path#"$package_root"/}"
    [[ ! -L "$path" ]] || { echo "正式创世状态包禁止符号链接:$relative" >&2; return 1; }
    case "$relative" in
      manifest.json|chains|chains/citizenchain|chains/citizenchain/db|chains/citizenchain/db/*) ;;
      *) echo "正式创世状态包包含白名单外残留:$relative" >&2; return 1 ;;
    esac
  done < <(find "$package_root" -mindepth 1 -print0)
}

# 中文注释：服务器部署由本机控制台选择单个节点，节点私钥只从 macOS Keychain 注入本进程。
required=(GMB_NODE_ID GMB_NODE_LABEL GMB_NODE_IP GMB_NODE_PEER_ID GMB_NODE_GRANDPA_PUBKEY GMB_NODE_BOOTNODE_KEY GMB_NODE_VALIDATOR_KEY GMB_NODE_SSH_KEY)
for name in "${required[@]}"; do
  [[ -n "${!name:-}" ]] || { echo "缺少节点部署参数：$name" >&2; exit 1; }
done

echo '[步骤 2] 查找当前提交最新成功的 CitizenChain CI'
# `branch` / `head_sha` 由 require_clean_remote_commit 在当前 shell 中赋值。
# shellcheck disable=SC2154
run_id="$(gh run list --workflow citizenchain-ci.yml --branch "$branch" --status success --limit 50 --json databaseId,headSha --jq "map(select(.headSha == \"$head_sha\"))[0].databaseId // empty")"
[[ -n "$run_id" ]] || { echo '当前提交没有成功的 CitizenChain CI，停止部署' >&2; exit 1; }

runtime_dir="$GMB_ROOT/deploy/.runtime"
mkdir -p "$runtime_dir"
chmod 700 "$runtime_dir"
work_dir="$(mktemp -d "$runtime_dir/node-deploy.XXXXXX")"
cleanup() { rm -rf "$work_dir"; }
trap cleanup EXIT

echo "[步骤 3] 下载 CI ${run_id} 的 Linux amd 安装包"
gh run download "$run_id" --name '公民链-Linux-amd' --dir "$work_dir/package"
deb_file="$work_dir/package/公民链-Linux-amd.deb"
[[ -f "$deb_file" ]] || { echo 'CI 产物缺少 Linux amd 安装包' >&2; exit 1; }

echo "[步骤 4] 下载 CI ${run_id} 的唯一正式创世状态包"
genesis_state_dir="$work_dir/genesis-state"
gh run download "$run_id" --name 'citizenchain-genesis-state' --dir "$genesis_state_dir"
validate_genesis_state_package "$genesis_state_dir"
genesis_metadata="$(python3 - "$genesis_state_dir/manifest.json" <<'PY'
import json
import re
import sys

with open(sys.argv[1], encoding="utf-8") as f:
    manifest = json.load(f)
if manifest.get("package_format") != "citizenchain-genesis-state-v1" or manifest.get("chain_id") != "citizenchain":
    raise SystemExit("正式创世状态包 manifest 身份无效")
if manifest.get("included_paths") != ["chains/citizenchain/db"]:
    raise SystemExit("正式创世状态包 included_paths 无效")
for key in ("genesis_hash", "state_root"):
    value = manifest.get(key)
    if not isinstance(value, str) or not re.fullmatch(r"0x[0-9a-f]{64}", value):
        raise SystemExit(f"正式创世状态包 {key} 无效")
for key in ("chainspec_hash", "runtime_wasm_hash", "light_sync_state_hash", "public_institution_root"):
    value = manifest.get(key)
    if not isinstance(value, str) or not re.fullmatch(r"[0-9a-f]{64}", value):
        raise SystemExit(f"正式创世状态包 {key} 无效")
run_id = str(manifest.get("runtime_wasm_ci_run_id", ""))
head_sha = manifest.get("runtime_wasm_ci_head_sha", "")
if not run_id.isdigit() or not isinstance(head_sha, str) or not re.fullmatch(r"[0-9a-f]{40}", head_sha):
    raise SystemExit("正式创世状态包 WASM CI 来源无效")
print(f'{manifest["genesis_hash"]}\t{manifest["state_root"]}')
PY
)"
IFS=$'\t' read -r expected_genesis_hash expected_state_root <<< "$genesis_metadata"
genesis_archive="$work_dir/citizenchain-genesis-state.tar.gz"
# macOS bsdtar 默认会把扩展属性写成 `._*` AppleDouble 成员；生产归档必须彻底排除。
COPYFILE_DISABLE=1 tar -C "$genesis_state_dir" -czf "$genesis_archive" manifest.json chains/citizenchain/db
while IFS= read -r member; do
  member="${member%/}"
  case "$member" in
    manifest.json|chains/citizenchain/db|chains/citizenchain/db/*) ;;
    *) echo "正式创世部署归档包含白名单外成员:$member" >&2; exit 1 ;;
  esac
done < <(tar -tzf "$genesis_archive")

ssh_key_file="$work_dir/ssh-key"
node_key_file="$work_dir/secret_ed25519"
grandpa_file="$work_dir/6772616e${GMB_NODE_GRANDPA_PUBKEY}"
known_hosts_file="$work_dir/known_hosts"
printf '%s\n' "$GMB_NODE_SSH_KEY" | tr -d '\r' > "$ssh_key_file"
printf '%s' "$GMB_NODE_BOOTNODE_KEY" | xxd -r -p > "$node_key_file"
printf '"0x%s"\n' "$GMB_NODE_VALIDATOR_KEY" > "$grandpa_file"
chmod 600 "$ssh_key_file" "$node_key_file" "$grandpa_file"
ssh-keyscan -p 22 -H "$GMB_NODE_IP" > "$known_hosts_file"

ssh_opts=(-i "$ssh_key_file" -p 22 -o BatchMode=yes -o StrictHostKeyChecking=yes -o UserKnownHostsFile="$known_hosts_file" -o ConnectTimeout=30 -o ServerAliveInterval=10 -o ServerAliveCountMax=6)
scp_opts=(-i "$ssh_key_file" -P 22 -o BatchMode=yes -o StrictHostKeyChecking=yes -o UserKnownHostsFile="$known_hosts_file" -o ConnectTimeout=30)
ssh_target="ubuntu@$GMB_NODE_IP"

echo '[步骤 5] 预检查目标服务器和 SSH 权限'
ssh "${ssh_opts[@]}" "$ssh_target" 'sudo -n true && echo deploy-ready:$(hostname):$(whoami)'

echo '[步骤 6] 上传安装包、服务配置和节点密钥临时文件'
scp "${scp_opts[@]}" "$deb_file" "$ssh_target:/tmp/citizenchain.deb"
scp "${scp_opts[@]}" "$genesis_archive" "$ssh_target:/tmp/citizenchain-genesis-state.tar.gz"
scp "${scp_opts[@]}" "$GMB_ROOT/citizenchain/scripts/citizenchain-node.service" "$ssh_target:/tmp/citizenchain-node.service"
scp "${scp_opts[@]}" "$node_key_file" "$ssh_target:/tmp/citizenchain-node-key"
scp "${scp_opts[@]}" "$grandpa_file" "$ssh_target:/tmp/citizenchain-grandpa-key"

echo '[步骤 7] 安装节点身份、验证密钥和最新软件'
# 公开 GRANDPA key 由本机校验后展开进远端目标文件名。
# shellcheck disable=SC2029
ssh "${ssh_opts[@]}" "$ssh_target" "
  set -euo pipefail
  sudo systemctl stop citizenchain-node 2>/dev/null || true
  sudo useradd --system --no-create-home --shell /usr/sbin/nologin citizenchain 2>/dev/null || true
  sudo install -d -m 700 -o citizenchain -g citizenchain /opt/citizenchain/data/node-key
  sudo install -d -m 700 -o citizenchain -g citizenchain /opt/citizenchain/data/chains/citizenchain/keystore
  sudo rm -rf /tmp/citizenchain-genesis-state
  sudo install -d -m 700 /tmp/citizenchain-genesis-state
  sudo tar --extract --gzip --file /tmp/citizenchain-genesis-state.tar.gz --directory /tmp/citizenchain-genesis-state --no-same-owner --no-same-permissions
  sudo test -f /tmp/citizenchain-genesis-state/manifest.json
  sudo test -d /tmp/citizenchain-genesis-state/chains/citizenchain/db
  sudo iptables -C INPUT -p tcp --dport 30333 -j ACCEPT 2>/dev/null || sudo iptables -I INPUT -p tcp --dport 30333 -j ACCEPT
  sudo sh -c 'iptables-save > /etc/iptables/rules.v4' 2>/dev/null || true
  sudo install -m 600 -o citizenchain -g citizenchain /tmp/citizenchain-node-key /opt/citizenchain/data/node-key/secret_ed25519
  sudo find /opt/citizenchain/data/chains/citizenchain/keystore -maxdepth 1 -type f -name '6772616e*' -delete
  sudo install -m 600 -o citizenchain -g citizenchain /tmp/citizenchain-grandpa-key '/opt/citizenchain/data/chains/citizenchain/keystore/6772616e${GMB_NODE_GRANDPA_PUBKEY}'
  sudo install -m 644 /tmp/citizenchain-node.service /etc/systemd/system/citizenchain-node.service
  sudo dpkg -i /tmp/citizenchain.deb || sudo apt-get install -f -y
  # 新旧 genesis 不兼容：先在目标文件系统完整暂存新库，再切换并删除旧库；保留独立 node key 与 GRANDPA keystore。
  sudo rm -rf /opt/citizenchain/data/chains/citizenchain/db.installing /opt/citizenchain/data/chains/citizenchain/db.previous /opt/citizenchain/data/chains/citizenchain/network
  sudo cp -a /tmp/citizenchain-genesis-state/chains/citizenchain/db /opt/citizenchain/data/chains/citizenchain/db.installing
  sudo chown -R citizenchain:citizenchain /opt/citizenchain/data/chains/citizenchain/db.installing
  if sudo test -d /opt/citizenchain/data/chains/citizenchain/db; then
    sudo mv /opt/citizenchain/data/chains/citizenchain/db /opt/citizenchain/data/chains/citizenchain/db.previous
  fi
  sudo mv /opt/citizenchain/data/chains/citizenchain/db.installing /opt/citizenchain/data/chains/citizenchain/db
  sudo rm -rf /opt/citizenchain/data/chains/citizenchain/db.previous
  sudo install -m 600 -o citizenchain -g citizenchain /tmp/citizenchain-genesis-state/manifest.json /opt/citizenchain/data/chains/citizenchain/genesis-state-manifest.json
  sudo rm -rf /tmp/citizenchain-genesis-state
  sudo rm -f /tmp/citizenchain.deb /tmp/citizenchain-genesis-state.tar.gz /tmp/citizenchain-node.service /tmp/citizenchain-node-key /tmp/citizenchain-grandpa-key
  sudo systemctl daemon-reload
  sudo systemctl enable --now citizenchain-node
"

echo '[步骤 8] 验证节点服务、P2P 身份和验证节点角色'
# 期望 genesis 由本机已验证 manifest 展开进远端 RPC 请求和结果断言。
# shellcheck disable=SC2029
ssh "${ssh_opts[@]}" "$ssh_target" "
  set -euo pipefail
  sudo systemctl is-active --quiet citizenchain-node
  for _ in \$(seq 1 40); do
    health=\$(curl --silent --max-time 2 -H 'content-type: application/json' --data '{\"id\":1,\"jsonrpc\":\"2.0\",\"method\":\"system_health\",\"params\":[]}' http://127.0.0.1:9944 || true)
    roles=\$(curl --silent --max-time 2 -H 'content-type: application/json' --data '{\"id\":1,\"jsonrpc\":\"2.0\",\"method\":\"system_nodeRoles\",\"params\":[]}' http://127.0.0.1:9944 || true)
    genesis=\$(curl --silent --max-time 2 -H 'content-type: application/json' --data '{\"id\":1,\"jsonrpc\":\"2.0\",\"method\":\"chain_getBlockHash\",\"params\":[0]}' http://127.0.0.1:9944 | python3 -c 'import json,sys; print((json.load(sys.stdin).get(\"result\") or \"\"))' 2>/dev/null || true)
    state_root=\$(curl --silent --max-time 2 -H 'content-type: application/json' --data '{\"id\":1,\"jsonrpc\":\"2.0\",\"method\":\"chain_getHeader\",\"params\":[\"${expected_genesis_hash}\"]}' http://127.0.0.1:9944 | python3 -c 'import json,sys; print(((json.load(sys.stdin).get(\"result\") or {}).get(\"stateRoot\") or \"\"))' 2>/dev/null || true)
    if echo \"\$health\" | grep -q '\"result\"' && echo \"\$roles\" | grep -Eq 'Authority|Validator' && [ \"\$genesis\" = '${expected_genesis_hash}' ] && [ \"\$state_root\" = '${expected_state_root}' ]; then
      exit 0
    fi
    sleep 1
  done
  echo '节点未在规定时间内通过服务、角色、genesis 与 state root 验收' >&2
  exit 1
"
echo "${GMB_NODE_LABEL} 部署完成：${GMB_NODE_IP}"
