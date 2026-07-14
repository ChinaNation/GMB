#!/usr/bin/env bash
set -euo pipefail

# 中文注释：远端流水线只能针对已推送且工作区干净的当前提交执行。
require_clean_remote_commit() {
  echo '[步骤 1] 检查本地工作区和远端提交一致性'
  cd "$GMB_ROOT"
  [[ -z "$(git status --porcelain)" ]] || { echo '工作区存在未提交修改，停止触发远端任务' >&2; exit 1; }
  branch="$(git branch --show-current)"
  head_sha="$(git rev-parse HEAD)"
  remote_sha="$(git ls-remote origin "refs/heads/$branch" | awk '{print $1}')"
  [[ -n "$branch" && "$head_sha" == "$remote_sha" ]] || { echo '当前提交尚未完整推送到 origin' >&2; exit 1; }
}

run_workflow() {
  local workflow="$1" mode="${2:-}" before run_id=''
  echo "[步骤 2] 准备触发 GitHub 工作流：${workflow}"
  before="$(gh run list --workflow "$workflow" --branch "$branch" --event workflow_dispatch --limit 1 --json databaseId --jq '.[0].databaseId // 0')"
  if [[ -n "$mode" ]]; then
    gh workflow run "$workflow" --ref "$branch" -f mode="$mode"
  else
    gh workflow run "$workflow" --ref "$branch"
  fi
  echo '[步骤 3] 等待本次远端任务创建'
  for _ in {1..60}; do
    run_id="$(gh run list --workflow "$workflow" --branch "$branch" --event workflow_dispatch --limit 10 --json databaseId,headSha --jq "map(select(.headSha == \"$head_sha\" and .databaseId != $before))[0].databaseId // empty")"
    [[ -z "$run_id" ]] || break
    sleep 2
  done
  [[ -n "$run_id" ]] || { echo '未找到本次新触发的远端任务' >&2; exit 1; }
  echo "[步骤 4] 跟踪远端任务直到完成：${run_id}"
  gh run watch "$run_id" --exit-status
}
