#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "${script_dir}/../.." && pwd)"

app_path="${1:-${repo_root}/citizenchain/nodeui/target/release/bundle/macos/citizenchain.app}"
dmg_path="${2:-${repo_root}/citizenchain/nodeui/target/release/bundle/dmg/citizenchain-macos-arm64.dmg}"
volume_name="${3:-citizenchain}"

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "DMG packaging is only supported on macOS" >&2
  exit 1
fi

if [[ ! -d "${app_path}" ]]; then
  echo "App bundle not found: ${app_path}" >&2
  exit 1
fi

stage_dir="$(mktemp -d)"
mkdir -p "$(dirname "${dmg_path}")"
trap 'rm -rf "${stage_dir}"' EXIT

cp -R "${app_path}" "${stage_dir}/"
hdiutil create \
  -volname "${volume_name}" \
  -srcfolder "${stage_dir}" \
  -ov \
  -format UDZO \
  "${dmg_path}"

echo "Created DMG: ${dmg_path}"
