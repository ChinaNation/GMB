#!/usr/bin/env bash
set -euo pipefail

base_ref="${BASE_REF:-origin/main}"

if ! git rev-parse --verify "${base_ref}" >/dev/null 2>&1; then
  branch="${base_ref#origin/}"
  git fetch origin "${branch}" --depth=1
fi

merge_base="$(git merge-base HEAD "${base_ref}")"
mapfile -t changed_files < <(git diff --name-only "${merge_base}...HEAD")

if [[ "${#changed_files[@]}" -eq 0 ]]; then
  echo "未检测到变更文件，跳过 AI 门禁检查。"
  exit 0
fi

doc_regex='^(memory/|docs/|README\.md$|GMB_TECHNICAL\.md$|CLAUDE\.md$|\.github/pull_request_template\.md$|.*_TECHNICAL\.md$)'
code_regex='^(\.github/workflows/|\.github/scripts/|citizenchain/|sfid/|cpms/|wuminapp/|primitives/|scripts/|Cargo\.toml$|Cargo\.lock$|.*\.(rs|dart|ts|tsx|js|jsx|sh|py|sql|toml|ya?ml|json|swift|kt|kts))'
scan_regex='^(\.github/workflows/|\.github/scripts/|citizenchain/|sfid/|cpms/|wuminapp/|primitives/|scripts/|.*\.(rs|dart|ts|tsx|js|jsx|sh|py|sql|toml))'
residual_regex='(console\.log\(|debugger;|dbg!\(|todo!\(|unimplemented!\(|\bTODO\b|\bFIXME\b)'

declare -a changed_code_files=()
declare -a changed_doc_files=()
declare -a residual_hits=()

for file in "${changed_files[@]}"; do
  if [[ "${file}" =~ ${doc_regex} ]]; then
    changed_doc_files+=("${file}")
  fi

  if [[ "${file}" =~ ${code_regex} ]]; then
    changed_code_files+=("${file}")
  fi
done

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
