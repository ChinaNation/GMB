#!/usr/bin/env bash
set -euo pipefail

base_ref="${BASE_REF:-origin/main}"

if ! git rev-parse --verify "${base_ref}" >/dev/null 2>&1; then
  branch="${base_ref#origin/}"
  git fetch origin "${branch}" --depth=1
fi

merge_base="$(git merge-base HEAD "${base_ref}")"
declare -a changed_files=()
while IFS= read -r file; do
  changed_files+=("${file}")
done < <(git diff --name-only "${merge_base}...HEAD")

declare -a status_lines=()
while IFS= read -r line; do
  status_lines+=("${line}")
done < <(git diff --name-status --find-renames "${merge_base}...HEAD")

if [[ "${#changed_files[@]}" -eq 0 ]]; then
  echo "未检测到变更文件，跳过 AI 门禁检查。"
  exit 0
fi

bash memory/scripts/check-startup-acceptance.sh --ci

doc_regex='^(memory/|docs/|README\.md$|GMB_TECHNICAL\.md$|CLAUDE\.md$|\.github/pull_request_template\.md$|.*_TECHNICAL\.md$)'
code_regex='^(\.github/workflows/|\.github/scripts/|citizenchain/|sfid/|cpms/|wuminapp/|primitives/|Cargo\.toml$|Cargo\.lock$|.*\.(rs|dart|ts|tsx|js|jsx|sh|py|sql|toml|ya?ml|json|swift|kt|kts))'
scan_regex='^(\.github/scripts/|citizenchain/|sfid/|cpms/|wuminapp/|primitives/|.*\.(rs|dart|ts|tsx|js|jsx|sh|py|sql|toml))'
task_card_regex='^memory/08-tasks/(open|done)/[^/]+\.md$'
todo_word="TO""DO"
fixme_word="FIX""ME"
residual_regex="(console\\.log\\(|debugger;|dbg!\\(|todo!\\(|unimplemented!\\(|\\b${todo_word}\\b|\\b${fixme_word}\\b)"
chinese_comment_regex='^\+.*(//|/\*|\*|#).*[一-龥]'

declare -a changed_code_files=()
declare -a changed_doc_files=()
declare -a changed_task_cards=()
declare -a residual_hits=()
declare -a protected_ai_hits=()
declare -a missing_task_card_hits=()
declare -a missing_module_doc_hits=()
declare -a chinese_comment_hits=()

has_changed_doc_prefix() {
  local prefix="$1"
  local file

  for file in "${changed_doc_files[@]}"; do
    if [[ "$file" == "$prefix"* ]]; then
      return 0
    fi
  done

  return 1
}

has_changed_doc_file() {
  local target="$1"
  local file

  for file in "${changed_doc_files[@]}"; do
    if [[ "$file" == "$target" ]]; then
      return 0
    fi
  done

  return 1
}

module_doc_requirement_for_file() {
  local file="$1"

  case "$file" in
    .github/workflows/*|.github/scripts/*)
      printf '%s' "memory/07-ai/"
      ;;
    citizenchain/*|primitives/*|Cargo.toml|Cargo.lock)
      printf '%s' "memory/05-modules/citizenchain/"
      ;;
    sfid/*)
      printf '%s' "memory/05-modules/sfid/"
      ;;
    cpms/*)
      printf '%s' "memory/05-modules/cpms/"
      ;;
    wuminapp/*)
      printf '%s' "memory/05-modules/wuminapp/"
      ;;
    *)
      printf '%s' ""
      ;;
  esac
}

has_matching_module_doc_update() {
  local file="$1"
  local required_prefix

  required_prefix="$(module_doc_requirement_for_file "$file")"

  if [[ -z "$required_prefix" ]]; then
    return 0
  fi

  if has_changed_doc_prefix "$required_prefix"; then
    return 0
  fi

  case "$file" in
    .github/workflows/*|.github/scripts/*)
      has_changed_doc_file "memory/01-architecture/repo-map.md" && return 0
      ;;
    citizenchain/*|primitives/*|Cargo.toml|Cargo.lock)
      has_changed_doc_file "memory/01-architecture/citizenchain-target-structure.md" && return 0
      has_changed_doc_file "memory/01-architecture/repo-map.md" && return 0
      has_changed_doc_file "memory/03-security/security-rules.md" && return 0
      ;;
    sfid/*|cpms/*|wuminapp/*)
      has_changed_doc_file "memory/01-architecture/repo-map.md" && return 0
      has_changed_doc_file "memory/03-security/security-rules.md" && return 0
      ;;
    *)
      ;;
  esac

  return 1
}

should_check_chinese_comment_gate() {
  local file="$1"

  case "$file" in
    *.rs|*.dart|*.ts|*.tsx|*.js|*.jsx|*.swift|*.kt|*.kts)
      ;;
    *)
      return 1
      ;;
  esac

  case "$file" in
    */test/*|*/tests/*|*.g.dart|*/GeneratedPluginRegistrant.*)
      return 1
      ;;
    *)
      return 0
      ;;
  esac
}

check_chinese_comment_gate() {
  local file="$1"
  local added_lines
  local added_count

  added_lines="$(git diff --unified=0 "${merge_base}...HEAD" -- "$file" | grep '^+' | grep -v '^\+\+\+' || true)"

  if [[ -z "$added_lines" ]]; then
    return 0
  fi

  added_count="$(printf '%s\n' "$added_lines" | sed '/^[[:space:]]*$/d' | wc -l | tr -d ' ')"

  if [[ -z "$added_count" || "$added_count" -lt 12 ]]; then
    return 0
  fi

  if printf '%s\n' "$added_lines" | grep -Eq "$chinese_comment_regex"; then
    return 0
  fi

  chinese_comment_hits+=("${file}: 新增 ${added_count} 行实现，但未检测到新增中文注释")
}

is_protected_ai_path() {
  local file="$1"

  case "$file" in
    # 中文注释：根目录入口别名本身也是启动协议的一部分，不能删除或迁出。
    AGENTS.md|CODEX.md|CLAUDE.md)
      return 0
      ;;
    # 中文注释：memory/ 是 AI 编程系统唯一实体目录，下列路径属于核心基础设施。
    memory/README.md|memory/AGENTS.md|memory/CODEX.md|memory/CLAUDE.md|\
    memory/08-tasks/README.md|memory/08-tasks/index.md|\
    memory/08-tasks/open/README.md|memory/08-tasks/done/README.md)
      return 0
      ;;
    memory/00-vision/*|memory/01-architecture/*|memory/03-security/*|\
    memory/04-decisions/*|memory/05-modules/*|memory/06-quality/*|memory/07-ai/*|\
    memory/scripts/*|memory/08-tasks/templates/*)
      return 0
      ;;
    *)
      return 1
      ;;
  esac
}

should_skip_residual_scan() {
  local file="$1"

  case "$file" in
    # 中文注释：门禁脚本自身包含残留关键字匹配规则，不能把规则文本再视为命中结果。
    .github/scripts/check-ai-guardrails.sh)
      return 0
      ;;
    # 中文注释：Flutter 生成目录里的 CMake 文件带默认模板注释，属于框架产物，不应拦截 PR。
    citizenchain/node/linux/flutter/CMakeLists.txt|citizenchain/node/windows/flutter/CMakeLists.txt)
      return 0
      ;;
    *)
      return 1
      ;;
  esac
}

for file in "${changed_files[@]}"; do
  if [[ "${file}" =~ ${doc_regex} ]]; then
    changed_doc_files+=("${file}")
  fi

  if [[ "${file}" =~ ${task_card_regex} ]]; then
    changed_task_cards+=("${file}")
  fi

  if [[ "${file}" =~ ${code_regex} ]]; then
    changed_code_files+=("${file}")
  fi
done

for line in "${status_lines[@]}"; do
  IFS=$'\t' read -r status old_path new_path <<< "${line}"

  case "${status}" in
    D)
      if is_protected_ai_path "${old_path}"; then
        protected_ai_hits+=("禁止删除 AI 编程系统核心基础设施: ${old_path}")
      fi
      ;;
    R*)
      if is_protected_ai_path "${old_path}"; then
        protected_ai_hits+=("禁止迁出 AI 编程系统核心基础设施: ${old_path} -> ${new_path}")
      fi
      ;;
    *)
      ;;
  esac
done

if [[ "${#protected_ai_hits[@]}" -gt 0 ]]; then
  echo "检测到对 AI 编程系统核心基础设施的删除或迁移操作。"
  echo "以下路径受保护，禁止通过 PR 删除、迁出或重命名："
  printf '  - %s\n' "${protected_ai_hits[@]}"
  echo ""
  echo "请保留这些路径，或仅在原位修改其内容。"
  exit 1
fi

if [[ "${#changed_code_files[@]}" -gt 0 && "${#changed_doc_files[@]}" -eq 0 ]]; then
  echo "检测到代码或自动化变更，但没有同步更新文档。"
  echo "请至少更新以下任一类型文档："
  echo "- memory/"
  echo "- *_TECHNICAL.md"
  echo "- README.md / GMB_TECHNICAL.md / CLAUDE.md"
  echo ""
  printf '代码变更文件:\n'
  printf '  - %s\n' "${changed_code_files[@]}"
  exit 1
fi

if [[ "${#changed_code_files[@]}" -gt 0 && "${#changed_task_cards[@]}" -eq 0 ]]; then
  echo "检测到真实开发变更，但没有同步任务卡。"
  echo "真实开发任务必须至少更新一张任务卡："
  echo "- memory/08-tasks/open/<任务卡>.md"
  echo "- memory/08-tasks/done/<任务卡>.md"
  echo ""
  printf '代码变更文件:\n'
  printf '  - %s\n' "${changed_code_files[@]}"
  exit 1
fi

for file in "${changed_code_files[@]}"; do
  if ! has_matching_module_doc_update "$file"; then
    required_prefix="$(module_doc_requirement_for_file "$file")"
    if [[ -n "$required_prefix" ]]; then
      missing_module_doc_hits+=("${file}: 缺少对应模块文档更新（期望更新 ${required_prefix}）")
    fi
  fi
done

if [[ "${#missing_module_doc_hits[@]}" -gt 0 ]]; then
  echo "检测到更细粒度的文档回写缺失。"
  echo "以下代码变更没有同步到对应模块文档："
  printf '  - %s\n' "${missing_module_doc_hits[@]}"
  exit 1
fi

for file in "${changed_code_files[@]}"; do
  if [[ ! -f "${file}" ]]; then
    continue
  fi

  if [[ ! "${file}" =~ ${scan_regex} ]]; then
    continue
  fi

  if should_skip_residual_scan "${file}"; then
    continue
  fi

  if grep -nE "${residual_regex}" "${file}" >/tmp/gmb_guardrail_hit.txt; then
    while IFS= read -r line; do
      residual_hits+=("${file}:${line}")
    done < /tmp/gmb_guardrail_hit.txt
  fi

  if should_check_chinese_comment_gate "$file"; then
    check_chinese_comment_gate "$file"
  fi
done

rm -f /tmp/gmb_guardrail_hit.txt

if [[ "${#residual_hits[@]}" -gt 0 ]]; then
  echo "检测到可能未清理的开发残留："
  printf '  - %s\n' "${residual_hits[@]}"
  echo "请清理后重新提交。"
  exit 1
fi

if [[ "${#chinese_comment_hits[@]}" -gt 0 ]]; then
  echo "检测到较大代码改动，但没有同步新增中文注释："
  printf '  - %s\n' "${chinese_comment_hits[@]}"
  echo "请至少为关键逻辑补充轻量中文注释后重新提交。"
  exit 1
fi

echo "AI 门禁检查通过。"
if [[ "${#changed_doc_files[@]}" -gt 0 ]]; then
  echo "本次已检测到文档更新："
  printf '  - %s\n' "${changed_doc_files[@]}"
fi
