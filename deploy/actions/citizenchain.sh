#!/usr/bin/env bash
set -euo pipefail
# shellcheck disable=SC1091
source "$(dirname "${BASH_SOURCE[0]}")/common.sh"
mode="${1:?缺少模式}"
[[ "$mode" == ci || "$mode" == release || "$mode" == deploy ]] || exit 2
genesis_deploy="${GMB_GENESIS_DEPLOY:-0}"
[[ "$genesis_deploy" == 0 || "$genesis_deploy" == 1 ]] || { echo 'GMB_GENESIS_DEPLOY 必须为 0 或 1' >&2; exit 2; }
require_clean_remote_commit
if [[ "$mode" != deploy ]]; then
  run_workflow citizenchain-ci.yml "$mode"
  exit 0
fi

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
scp "${scp_opts[@]}" "$GMB_ROOT/citizenchain/scripts/citizenchain-node.service" "$ssh_target:/tmp/citizenchain-node.service"
scp "${scp_opts[@]}" "$node_key_file" "$ssh_target:/tmp/citizenchain-node-key"
scp "${scp_opts[@]}" "$grandpa_file" "$ssh_target:/tmp/citizenchain-grandpa-key"

if [[ "$genesis_deploy" == 1 ]]; then
  echo '[步骤 7] 创世部署：停止服务并清空远端 CitizenChain 数据'
  remote_clear_data='sudo rm -rf /opt/citizenchain/data/chains/citizenchain'
else
  echo '[步骤 7] 普通部署：保留远端链数据，安装节点身份、验证密钥和最新软件'
  remote_clear_data=':'
fi
# 公开 GRANDPA key 由本机校验后展开进远端目标文件名。
# shellcheck disable=SC2029
ssh "${ssh_opts[@]}" "$ssh_target" "
  set -euo pipefail
  sudo systemctl stop citizenchain-node 2>/dev/null || true
  ${remote_clear_data}
  sudo useradd --system --no-create-home --shell /usr/sbin/nologin citizenchain 2>/dev/null || true
  sudo install -d -m 700 -o citizenchain -g citizenchain /opt/citizenchain/data/node-key
  sudo install -d -m 700 -o citizenchain -g citizenchain /opt/citizenchain/data/chains/citizenchain/keystore
  sudo iptables -C INPUT -p tcp --dport 30333 -j ACCEPT 2>/dev/null || sudo iptables -I INPUT -p tcp --dport 30333 -j ACCEPT
  sudo sh -c 'iptables-save > /etc/iptables/rules.v4' 2>/dev/null || true
  sudo install -m 600 -o citizenchain -g citizenchain /tmp/citizenchain-node-key /opt/citizenchain/data/node-key/secret_ed25519
  sudo find /opt/citizenchain/data/chains/citizenchain/keystore -maxdepth 1 -type f -name '6772616e*' -delete
  sudo install -m 600 -o citizenchain -g citizenchain /tmp/citizenchain-grandpa-key '/opt/citizenchain/data/chains/citizenchain/keystore/6772616e${GMB_NODE_GRANDPA_PUBKEY}'
  sudo install -m 644 /tmp/citizenchain-node.service /etc/systemd/system/citizenchain-node.service
  sudo dpkg -i /tmp/citizenchain.deb || sudo apt-get install -f -y
  sudo rm -f /tmp/citizenchain.deb /tmp/citizenchain-node.service /tmp/citizenchain-node-key /tmp/citizenchain-grandpa-key
  sudo systemctl daemon-reload
  sudo systemctl enable --now citizenchain-node
"

echo '[步骤 8] 验证节点服务、P2P 身份和验证节点角色'
# shellcheck disable=SC2029
ssh "${ssh_opts[@]}" "$ssh_target" "
  set -euo pipefail
  sudo systemctl is-active --quiet citizenchain-node
  for _ in \$(seq 1 40); do
    health=\$(curl --silent --max-time 2 -H 'content-type: application/json' --data '{\"id\":1,\"jsonrpc\":\"2.0\",\"method\":\"system_health\",\"params\":[]}' http://127.0.0.1:9944 || true)
    roles=\$(curl --silent --max-time 2 -H 'content-type: application/json' --data '{\"id\":1,\"jsonrpc\":\"2.0\",\"method\":\"system_nodeRoles\",\"params\":[]}' http://127.0.0.1:9944 || true)
    if echo \"\$health\" | grep -q '\"result\"' && echo \"\$roles\" | grep -Eq 'Authority|Validator'; then
      exit 0
    fi
    sleep 1
  done
  echo '节点未在规定时间内通过服务和角色验收' >&2
  exit 1
"
echo "${GMB_NODE_LABEL} 部署完成：${GMB_NODE_IP}"
