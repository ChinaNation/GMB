#!/usr/bin/env bash
set -euo pipefail
# shellcheck disable=SC1091
source "$(dirname "${BASH_SOURCE[0]}")/common.sh"
mode="${1:?缺少模式}"
[[ "$mode" == ci || "$mode" == release || "$mode" == deploy ]] || exit 2
genesis_deploy="${GMB_GENESIS_DEPLOY:-0}"
[[ "$genesis_deploy" == 0 || "$genesis_deploy" == 1 ]] || { echo 'GMB_GENESIS_DEPLOY 必须为 0 或 1' >&2; exit 2; }
if [[ "$mode" != deploy ]]; then
  # 中文注释：只有主动触发 CI/Release 才检查本地提交；部署服务器只消费 GitHub 最新成功 CI。
  require_clean_remote_commit
  run_workflow citizenchain-ci.yml "$mode"
  exit 0
fi

# 中文注释：服务器部署由本机控制台选择单个节点，节点私钥只从 macOS Keychain 注入本进程。
required=(GMB_NODE_ID GMB_NODE_LABEL GMB_NODE_IP GMB_NODE_PEER_ID GMB_NODE_GRANDPA_PUBKEY GMB_NODE_BOOTNODE_KEY GMB_NODE_VALIDATOR_KEY GMB_NODE_SSH_KEY)
for name in "${required[@]}"; do
  [[ -n "${!name:-}" ]] || { echo "缺少节点部署参数：$name" >&2; exit 1; }
done

echo '[步骤 2] 查找 GitHub main 最新成功的 CitizenChain CI'
repo="$(gh repo view --json nameWithOwner --jq '.nameWithOwner')"
run_json="$(gh run list --workflow citizenchain-ci.yml --branch main --status success --limit 1 --json databaseId,headSha)"
run_id="$(printf '%s' "$run_json" | jq -r '.[0].databaseId // empty')"
run_head_sha="$(printf '%s' "$run_json" | jq -r '.[0].headSha // empty')"
[[ -n "$run_id" && -n "$run_head_sha" ]] || { echo 'GitHub main 没有成功的 CitizenChain CI，停止部署' >&2; exit 1; }
manifest_json="$(gh api "repos/${repo}/contents/citizenapp/assets/public_institutions/manifest.json?ref=${run_head_sha}" --jq '.content' | tr -d '\n' | base64 --decode)"
expected_genesis_hash="$(printf '%s' "$manifest_json" | jq -r '.genesis_hash // empty')"
[[ "$expected_genesis_hash" =~ ^0x[0-9a-f]{64}$ ]] || { echo '成功 CI 提交缺少有效的冻结创世哈希' >&2; exit 1; }

# 中文注释：本机只取得 GitHub 短期签名下载地址和 SHA-256，不下载节点软件。
artifact_json="$(gh api "repos/${repo}/actions/runs/${run_id}/artifacts")"
artifact_id="$(printf '%s' "$artifact_json" | jq -r '[.artifacts[] | select(.name == "公民链-Linux-amd" and .expired == false)][0].id // empty')"
artifact_digest="$(printf '%s' "$artifact_json" | jq -r '.artifacts[] | select(.id == '"${artifact_id:-0}"') | .digest // empty')"
[[ -n "$artifact_id" ]] || { echo '最新成功 CI 缺少未过期的 Linux amd 产物' >&2; exit 1; }
[[ "$artifact_digest" == sha256:* ]] || { echo 'Linux amd CI 产物缺少 SHA-256 摘要' >&2; exit 1; }
artifact_sha256="${artifact_digest#sha256:}"
github_token="$(gh auth token)"
redirect_headers="$(curl --silent --show-error --dump-header - --output /dev/null \
  -H 'Accept: application/vnd.github+json' \
  -H "Authorization: Bearer ${github_token}" \
  -H 'X-GitHub-Api-Version: 2022-11-28' \
  "https://api.github.com/repos/${repo}/actions/artifacts/${artifact_id}/zip")"
artifact_url="$(printf '%s' "$redirect_headers" | awk 'BEGIN{IGNORECASE=1} /^location:/ {sub(/^[^:]+:[[:space:]]*/, ""); sub(/\r$/, ""); print; exit}')"
unset github_token redirect_headers
[[ "$artifact_url" == https://*.blob.core.windows.net/* ]] || { echo 'GitHub CI 产物下载地址校验失败' >&2; exit 1; }

runtime_dir="$GMB_ROOT/deploy/.runtime"
mkdir -p "$runtime_dir"
chmod 700 "$runtime_dir"
work_dir="$(mktemp -d "$runtime_dir/node-deploy.XXXXXX")"
cleanup() { rm -rf "$work_dir"; }
trap cleanup EXIT

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

echo '[步骤 3] 预检查目标服务器和 SSH 权限'
ssh "${ssh_opts[@]}" "$ssh_target" 'sudo -n true && echo deploy-ready:$(hostname):$(whoami)'

echo "[步骤 4] 目标服务器直接下载并校验 CI ${run_id} 的 Linux amd 安装包"
# 中文注释：短期签名地址通过 SSH 标准输入传递，不进入命令参数、日志或服务器持久配置。
printf '%s\n%s\n' "$artifact_url" "$artifact_sha256" | \
  ssh "${ssh_opts[@]}" "$ssh_target" 'umask 077; cat > /tmp/citizenchain-artifact-source'
ssh "${ssh_opts[@]}" "$ssh_target" '
  set -euo pipefail
  source_file=/tmp/citizenchain-artifact-source
  artifact_url="$(sed -n "1p" "$source_file")"
  artifact_sha256="$(sed -n "2p" "$source_file")"
  rm -f "$source_file"
  rm -rf /tmp/citizenchain-artifact /tmp/citizenchain-artifact.zip /tmp/citizenchain.deb
  curl --fail --location --silent --show-error --retry 3 --output /tmp/citizenchain-artifact.zip "$artifact_url"
  printf "%s  %s\n" "$artifact_sha256" /tmp/citizenchain-artifact.zip | sha256sum -c -
  mkdir -p /tmp/citizenchain-artifact
  python3 -m zipfile -e /tmp/citizenchain-artifact.zip /tmp/citizenchain-artifact
  deb_file="$(find /tmp/citizenchain-artifact -type f -name "公民链-Linux-amd.deb" -print -quit)"
  [[ -n "$deb_file" ]]
  mv "$deb_file" /tmp/citizenchain.deb
  dpkg-deb --info /tmp/citizenchain.deb >/dev/null
  rm -rf /tmp/citizenchain-artifact /tmp/citizenchain-artifact.zip
'

echo '[步骤 5] 上传服务配置和节点密钥临时文件'
scp "${scp_opts[@]}" "$GMB_ROOT/citizenchain/scripts/citizenchain-node.service" "$ssh_target:/tmp/citizenchain-node.service"
scp "${scp_opts[@]}" "$node_key_file" "$ssh_target:/tmp/citizenchain-node-key"
scp "${scp_opts[@]}" "$grandpa_file" "$ssh_target:/tmp/citizenchain-grandpa-key"

if [[ "$genesis_deploy" == 1 ]]; then
  echo '[步骤 6] 创世部署：停止服务并清空远端 CitizenChain 数据'
  remote_clear_data='sudo rm -rf /opt/citizenchain/data/chains/citizenchain'
else
  echo '[步骤 6] 普通部署：保留远端链数据，安装节点身份、验证密钥和最新软件'
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
  # 清库后必须先显式创建并授权链父目录，否则仅创建 keystore 会留下 root 所有的中间目录。
  sudo install -d -m 700 -o citizenchain -g citizenchain /opt/citizenchain/data/chains/citizenchain
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

echo '[步骤 7] 验证节点服务、P2P 身份和验证节点角色'
# shellcheck disable=SC2029
ssh "${ssh_opts[@]}" "$ssh_target" "
  set -euo pipefail
  for _ in \$(seq 1 180); do
    health=\$(curl --silent --max-time 2 -H 'content-type: application/json' --data '{\"id\":1,\"jsonrpc\":\"2.0\",\"method\":\"system_health\",\"params\":[]}' http://127.0.0.1:9944 || true)
    roles=\$(curl --silent --max-time 2 -H 'content-type: application/json' --data '{\"id\":1,\"jsonrpc\":\"2.0\",\"method\":\"system_nodeRoles\",\"params\":[]}' http://127.0.0.1:9944 || true)
    genesis=\$(curl --silent --max-time 2 -H 'content-type: application/json' --data '{\"id\":1,\"jsonrpc\":\"2.0\",\"method\":\"chain_getBlockHash\",\"params\":[0]}' http://127.0.0.1:9944 || true)
    peer_id=\$(curl --silent --max-time 2 -H 'content-type: application/json' --data '{\"id\":1,\"jsonrpc\":\"2.0\",\"method\":\"system_localPeerId\",\"params\":[]}' http://127.0.0.1:9944 || true)
    if sudo systemctl is-active --quiet citizenchain-node \
      && echo \"\$health\" | grep -q '\"result\"' \
      && echo \"\$roles\" | grep -Eq 'Authority|Validator' \
      && echo \"\$genesis\" | grep -Fq '${expected_genesis_hash}' \
      && echo \"\$peer_id\" | grep -Fq '${GMB_NODE_PEER_ID}'; then
      echo '节点服务、创世哈希、验证角色和 P2P 身份验收通过'
      exit 0
    fi
    sleep 1
  done
  sudo systemctl status citizenchain-node --no-pager --full >&2 || true
  sudo journalctl -u citizenchain-node -n 80 --no-pager >&2 || true
  echo '节点未在规定时间内通过服务、创世哈希、验证角色和 P2P 身份验收' >&2
  exit 1
"
echo "${GMB_NODE_LABEL} 部署完成：${GMB_NODE_IP}（CI ${run_id} / ${run_head_sha}）"
