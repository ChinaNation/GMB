#!/usr/bin/env bash
set -euo pipefail
# shellcheck disable=SC1091
source "$(dirname "${BASH_SOURCE[0]}")/common.sh"
mode="${1:?缺少模式}"
[[ "$mode" == ci || "$mode" == release ]] || exit 2
require_clean_remote_commit
run_workflow citizenapp-ci.yml "$mode"
