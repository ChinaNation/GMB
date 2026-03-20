#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "${script_dir}/../.." && pwd)"
default_bundle_dir="${repo_root}/citizenchain/nodeuitauri/target/release/bundle/macos"

resolve_default_app_path() {
  local matches=()
  shopt -s nullglob
  matches=("${default_bundle_dir}"/*.app)
  shopt -u nullglob

  if [[ "${#matches[@]}" -eq 1 ]]; then
    printf '%s\n' "${matches[0]}"
    return 0
  fi

  if [[ "${#matches[@]}" -eq 0 ]]; then
    echo "No .app bundle found in ${default_bundle_dir}" >&2
  else
    echo "Multiple .app bundles found in ${default_bundle_dir}; pass the app path explicitly" >&2
    printf '  %s\n' "${matches[@]}" >&2
  fi
  return 1
}

app_path="${1:-$(resolve_default_app_path)}"
dmg_path="${2:-${repo_root}/citizenchain/nodeuitauri/target/release/bundle/dmg/citizenchain-macos-arm64.dmg}"
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
ln -s /Applications "${stage_dir}/Applications"
hdiutil create \
  -volname "${volume_name}" \
  -srcfolder "${stage_dir}" \
  -ov \
  -format UDZO \
  "${dmg_path}"

echo "Created DMG: ${dmg_path}"
