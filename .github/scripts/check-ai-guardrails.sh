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

doc_regex='^(memory/|docs/|README\.md$|GMB_TECHNICAL\.md$|CLAUDE\.md$|\.github/pull_request_template\.md$|.*_TECHNICAL\.md$)'
code_regex='^(\.github/workflows/|\.github/scripts/|citizenchain/|sfid/|cpms/|wuminapp/|primitives/|Cargo\.toml$|Cargo\.lock$|.*\.(rs|dart|ts|tsx|js|jsx|sh|py|sql|toml|ya?ml|json|swift|kt|kts))'
scan_regex='^(\.github/scripts/|citizenchain/|sfid/|cpms/|wuminapp/|primitives/|.*\.(rs|dart|ts|tsx|js|jsx|sh|py|sql|toml))'
todo_word="TO""DO"
fixme_word="FIX""ME"
residual_regex="(console\\.log\\(|debugger;|dbg!\\(|todo!\\(|unimplemented!\\(|\\b${todo_word}\\b|\\b${fixme_word}\\b)"

declare -a changed_code_files=()
declare -a changed_doc_files=()
declare -a residual_hits=()
declare -a protected_ai_hits=()

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
    citizenchain/nodeui/linux/flutter/CMakeLists.txt|citizenchain/nodeui/windows/flutter/CMakeLists.txt)
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
done

rm -f /tmp/gmb_guardrail_hit.txt

if [[ "${#residual_hits[@]}" -gt 0 ]]; then
  echo "检测到可能未清理的开发残留："
  printf '  - %s\n' "${residual_hits[@]}"
  echo "请清理后重新提交。"
  exit 1
fi

echo "AI 门禁检查通过。"
if [[ "${#changed_doc_files[@]}" -gt 0 ]]; then
  echo "本次已检测到文档更新："
  printf '  - %s\n' "${changed_doc_files[@]}"
fi
