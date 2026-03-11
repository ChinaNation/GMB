#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "${script_dir}/../.." && pwd)"

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "This packaging script is intended for macOS hosts only" >&2
  exit 1
fi

target="${TARGET_TRIPLE:-aarch64-apple-darwin}"
frontend_dir="${repo_root}/citizenchain/nodeui/frontend"
app_path="${repo_root}/citizenchain/nodeui/target/release/bundle/macos/citizenchain.app"
dmg_path="${repo_root}/citizenchain/nodeui/target/release/bundle/dmg/citizenchain-${target}.dmg"

"${script_dir}/prepare-nodeui-sidecar.sh" "${target}"
npm --prefix "${frontend_dir}" ci
npm --prefix "${frontend_dir}" run tauri:build -- --bundles app
"${script_dir}/create-macos-dmg.sh" "${app_path}" "${dmg_path}" "citizenchain"

echo "Installer outputs:"
echo "  app=${app_path}"
echo "  dmg=${dmg_path}"
