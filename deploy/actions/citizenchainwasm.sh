#!/usr/bin/env bash
set -euo pipefail
# shellcheck disable=SC1091
source "$(dirname "${BASH_SOURCE[0]}")/common.sh"
[[ "${1:?缺少模式}" == ci ]] || exit 2
require_clean_remote_commit
run_workflow citizenchain-wasm.yml
