#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "${script_dir}/../.." && pwd)"

detect_host_target() {
  local os arch
  os="$(uname -s)"
  arch="$(uname -m)"

  case "${os}:${arch}" in
    Darwin:arm64)
      echo "aarch64-apple-darwin"
      ;;
    Darwin:x86_64)
      echo "x86_64-apple-darwin"
      ;;
    Linux:x86_64)
      echo "x86_64-unknown-linux-gnu"
      ;;
    Linux:aarch64)
      echo "aarch64-unknown-linux-gnu"
      ;;
    *)
      echo "Unsupported host for automatic node sidecar preparation: ${os}/${arch}" >&2
      exit 1
      ;;
  esac
}

configure_protoc() {
  if command -v protoc >/dev/null 2>&1; then
    export PROTOC
    PROTOC="$(command -v protoc)"
    return
  fi

  if [[ -x "/opt/homebrew/bin/protoc" ]]; then
    export PROTOC="/opt/homebrew/bin/protoc"
    return
  fi

  if [[ -x "/usr/local/bin/protoc" ]]; then
    export PROTOC="/usr/local/bin/protoc"
    return
  fi

  echo "Could not find protoc. Install protobuf or export PROTOC before running this script." >&2
  exit 1
}

target="${1:-${TARGET_TRIPLE:-}}"
if [[ -z "${target}" ]]; then
  target="$(detect_host_target)"
fi

configure_protoc

node_manifest="${repo_root}/citizenchain/node/Cargo.toml"
node_binary="${repo_root}/citizenchain/target/${target}/release/node"
# 旧版桌面节点壳已迁移到 nodeuitauri；新版 Flutter nodeui 暂未接管 sidecar 打包。
bin_dir="${repo_root}/citizenchain/nodeuitauri/backend/binaries"
sidecar_name="citizenchain-node-${target}"
plain_name="citizenchain-node"

mkdir -p "${bin_dir}"

if [[ "${PREPARE_NODEUI_SKIP_BUILD:-0}" != "1" ]]; then
  cargo build --manifest-path "${node_manifest}" --release --target "${target}"
fi

if [[ ! -f "${node_binary}" ]]; then
  echo "Expected node binary not found: ${node_binary}" >&2
  exit 1
fi

cp "${node_binary}" "${bin_dir}/${sidecar_name}"
cp "${node_binary}" "${bin_dir}/${plain_name}"
chmod +x "${bin_dir}/${sidecar_name}" "${bin_dir}/${plain_name}"

if command -v shasum >/dev/null 2>&1; then
  shasum -a 256 "${bin_dir}/${plain_name}" | awk '{print $1}' > "${bin_dir}/citizenchain-node.sha256"
elif command -v sha256sum >/dev/null 2>&1; then
  sha256sum "${bin_dir}/${plain_name}" | awk '{print $1}' > "${bin_dir}/citizenchain-node.sha256"
else
  echo "Neither shasum nor sha256sum is available for checksum generation" >&2
  exit 1
fi

echo "Prepared bundled node binary:"
echo "  target=${target}"
echo "  sidecar=${bin_dir}/${sidecar_name}"
echo "  plain=${bin_dir}/${plain_name}"
echo "  sha256=${bin_dir}/citizenchain-node.sha256"
