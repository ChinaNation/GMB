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

bash scripts/check-startup-acceptance.sh --ci

doc_regex='^(memory/|docs/|README\.md$|GMB_TECHNICAL\.md$|CLAUDE\.md$|\.github/pull_request_template\.md$|.*_TECHNICAL\.md$)'
code_regex='^(\.github/workflows/|scripts/|citizenchain/|citizencode/|citizenpassport/|citizenapp/|primitives/|Cargo\.toml$|Cargo\.lock$|.*\.(rs|dart|ts|tsx|js|jsx|sh|py|sql|toml|ya?ml|json|swift|kt|kts))'
scan_regex='^(scripts/|citizenchain/|citizencode/|citizenpassport/|citizenapp/|primitives/|.*\.(rs|dart|ts|tsx|js|jsx|sh|py|sql|toml))'
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
declare -a version_tag_hits=()
declare -a lint_suppression_hits=()

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
    .github/workflows/*|scripts/check-ai-guardrails.sh|scripts/analyze-requirement.sh|scripts/architect-entry.sh|scripts/check-startup-acceptance.sh|scripts/complete-task.sh|scripts/index-tasks.sh|scripts/load-context.sh|scripts/module-router.sh|scripts/new-task.sh|scripts/start-task.sh)
      printf '%s' "memory/07-ai/"
      ;;
    citizenchain/*|primitives/*|Cargo.toml|Cargo.lock)
      printf '%s' "memory/05-modules/citizenchain/"
      ;;
    citizencode/*)
      printf '%s' "memory/05-modules/citizencode/"
      ;;
    citizenpassport/*)
      printf '%s' "memory/05-modules/citizenpassport/"
      ;;
    citizenapp/*)
      printf '%s' "memory/05-modules/citizenapp/"
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
    .github/workflows/*|scripts/check-ai-guardrails.sh|scripts/analyze-requirement.sh|scripts/architect-entry.sh|scripts/check-startup-acceptance.sh|scripts/complete-task.sh|scripts/index-tasks.sh|scripts/load-context.sh|scripts/module-router.sh|scripts/new-task.sh|scripts/start-task.sh)
      has_changed_doc_file "memory/01-architecture/repo-map.md" && return 0
      ;;
    citizenchain/*|primitives/*|Cargo.toml|Cargo.lock)
      has_changed_doc_file "memory/01-architecture/citizenchain-target-structure.md" && return 0
      has_changed_doc_file "memory/01-architecture/repo-map.md" && return 0
      has_changed_doc_file "memory/03-security/security-rules.md" && return 0
      ;;
    citizencode/*|citizenpassport/*|citizenapp/*)
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

# 中文注释：协议版本标识门禁。全仓唯一允许的版本化协议标识是 QR_V1；
# 先把 QR_V1 从新增行里剔除，再看是否还剩别的 *_V1 标识符，剩下即拦截。
check_version_tag_gate() {
  local file="$1"
  local added_lines
  local offending

  # 中文注释：只查代码文件。规则文档必须能点名被禁标识符（例如「禁止 GMB_ROLE_V1」），
  # 把 Markdown 一并纳入会让规则正文自己触发门禁。
  case "$file" in
    *.rs|*.dart|*.ts|*.tsx|*.js|*.jsx|*.swift|*.kt|*.kts|*.proto|*.sql|*.ya?ml|*.json|*.toml|*.sh|*.py) ;;
    *) return 0 ;;
  esac

  # 中文注释：`\+` 在 BRE 下是重复算子，必须用 -E（ERE）才是字面加号，否则严格 grep 直接报错。
  added_lines="$(git diff --unified=0 "${merge_base}...HEAD" -- "$file" | grep -E '^\+' | grep -vE '^\+\+\+' || true)"

  if [[ -z "$added_lines" ]]; then
    return 0
  fi

  offending="$(printf '%s\n' "$added_lines" | sed 's/QR_V1//g' | grep -E '[A-Za-z0-9]_V1\b' || true)"

  if [[ -n "$offending" ]]; then
    version_tag_hits+=("${file}: 新增行出现非 QR_V1 的版本化标识（签名域走 signing_message(op_tag)，非签名哈希域用 MODULE_TAG）")
  fi
}

# 中文注释：编译器抑制门禁。新增 allow(dead_code)/allow(unused...) 必须写明中文理由，
# 否则等于又把编译器静音一处，扫描下一轮无从判断该保留还是该删。
# 理由写在同一行或紧邻上方（仓库既有惯例两种都用），故连取 allow 行前两行一起判定。
check_lint_suppression_gate() {
  local file="$1"
  local blocks
  local block=""
  local line
  local has_missing=0

  case "$file" in
    *.rs) ;;
    *) return 0 ;;
  esac

  blocks="$(git diff --unified=2 "${merge_base}...HEAD" -- "$file" \
    | grep -B2 -E '^\+.*#!?\[allow\((dead_code|unused)' || true)"

  if [[ -z "$blocks" ]]; then
    return 0
  fi

  # 中文注释：grep -B2 用 `--` 分隔各命中块；逐块要求块内出现中文注释。
  check_one_block() {
    if [[ -z "$1" ]]; then
      return 0
    fi
    if printf '%s\n' "$1" | grep -qE '(//|/\*|\*|#).*[一-龥]'; then
      return 0
    fi
    has_missing=1
  }

  while IFS= read -r line; do
    if [[ "$line" == "--" ]]; then
      check_one_block "$block"
      block=""
    else
      block+="${line}"$'\n'
    fi
  done <<< "$blocks"
  check_one_block "$block"

  if [[ "$has_missing" -eq 1 ]]; then
    lint_suppression_hits+=("${file}: 新增 allow(dead_code)/allow(unused) 缺中文理由注释（同行或紧邻上方均可）")
  fi
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
    memory/08-tasks/templates/*)
      return 0
      ;;
    # 中文注释:AI 工作流脚本已统一收敛到根 scripts/(原 memory/scripts/),逐个保护,避免误伤同目录通用工具脚本。
    scripts/analyze-requirement.sh|scripts/architect-entry.sh|scripts/check-startup-acceptance.sh|\
    scripts/complete-task.sh|scripts/index-tasks.sh|scripts/load-context.sh|\
    scripts/module-router.sh|scripts/new-task.sh|scripts/start-task.sh|\
    scripts/check-ai-guardrails.sh)
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
    .github/scripts/check-ai-guardrails.sh|scripts/check-ai-guardrails.sh)
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

# ── PQC 前向兼容守则（ADR-016）──
# 中文注释：当前阶段只用 sr25519、暂不接入 PQC。以下 sr25519 锚点保障将来无感接入
# PQC（不换钱包/账户/地址/金额）。锚点被“净删除/改值”时必须同步更新 ADR-016 守则章节确认。
pqc_guard_ack_doc="memory/04-decisions/ADR-016-account-key-pqc-migration.md"
pqc_guard_ack=false
for file in "${changed_files[@]}"; do
  if [[ "${file}" == "${pqc_guard_ack_doc}" ]]; then
    pqc_guard_ack=true
    break
  fi
done

# 中文注释：每条 = 受保护文件|受保护文本|说明；按“净删除（删除数 > 新增数）”判定,避免误伤纯改写。
declare -a pqc_anchor_specs=(
  "citizenchain/runtime/src/lib.rs|Signature = MultiSignature|账户签名模型 AccountId=sr25519 公钥"
  "citizenchain/runtime/src/lib.rs|AuthorizeCall|general-transaction 授权入口（PQC 挂载钩子）"
  "citizenchain/runtime/primitives/src/core_const.rs|SS58_FORMAT|SS58 前缀常量"
  "citizenchain/runtime/primitives/src/core_const.rs|2027|SS58 前缀值（地址不变）"
  "citizenapp/lib/wallet/core/wallet_manager.dart|miniSecretFromEntropy|助记词到 AccountRootSeedV1 派生"
  "citizenwallet/lib/wallet/wallet_manager.dart|miniSecretFromEntropy|助记词到 AccountRootSeedV1 派生"
  "citizenapp/lib/qr/bodies/sign_request_body.dart|sig_alg|QR 签名算法字段（PQC 扩展位）"
  "citizenapp/lib/qr/bodies/sign_response_body.dart|sig_alg|QR 签名算法字段（PQC 扩展位）"
  "citizenapp/lib/qr/bodies/login_receipt_body.dart|sig_alg|QR 签名算法字段（PQC 扩展位）"
  "citizenwallet/lib/qr/bodies/sign_request_body.dart|sig_alg|QR 签名算法字段（PQC 扩展位）"
  "citizenwallet/lib/qr/bodies/sign_response_body.dart|sig_alg|QR 签名算法字段（PQC 扩展位）"
  "citizenwallet/lib/qr/bodies/login_receipt_body.dart|sig_alg|QR 签名算法字段（PQC 扩展位）"
)

declare -a pqc_guard_hits=()
for spec in "${pqc_anchor_specs[@]}"; do
  IFS='|' read -r anchor_file anchor_pat anchor_desc <<< "${spec}"
  anchor_diff="$(git diff "${merge_base}...HEAD" -- "${anchor_file}" || true)"
  if [[ -z "${anchor_diff}" ]]; then
    continue
  fi
  # 中文注释：用 awk 按字面子串统计“删除行/新增行”中锚点出现次数（跳过 +++/--- 文件头），
  # 避免 grep 正则方言差异（GNU grep 与 ugrep 对 \+ 处理不同）导致误判。
  pqc_counts="$(printf '%s\n' "${anchor_diff}" | awk -v pat="${anchor_pat}" '
    { if (substr($0,1,3)=="+++" || substr($0,1,3)=="---") next;
      if (index($0,pat)==0) next;
      c=substr($0,1,1);
      if (c=="-") rem++; else if (c=="+") add++; }
    END { printf "%d %d", rem+0, add+0 }')"
  removed_count="${pqc_counts%% *}"
  added_count="${pqc_counts##* }"
  if [[ "${removed_count}" -gt "${added_count}" ]]; then
    pqc_guard_hits+=("${anchor_file}: 锚点「${anchor_pat}」被删改（${anchor_desc}）")
  fi
done

if [[ "${#pqc_guard_hits[@]}" -gt 0 && "${pqc_guard_ack}" == false ]]; then
  echo "检测到改动 PQC 前向兼容守则保护的 sr25519 锚点（ADR-016）。"
  echo "当前阶段只用 sr25519、暂不接入 PQC；以下锚点保障将来无感接入（不换钱包/账户/地址/金额），不得随意删改："
  printf '  - %s\n' "${pqc_guard_hits[@]}"
  echo ""
  echo "若确属有意变更，请同步更新 ${pqc_guard_ack_doc} 的「当前 sr25519 阶段：前向兼容守则」章节后再提交。"
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

  check_version_tag_gate "$file"
  check_lint_suppression_gate "$file"
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

if [[ "${#version_tag_hits[@]}" -gt 0 ]]; then
  echo "检测到新增的版本化协议标识："
  printf '  - %s\n' "${version_tag_hits[@]}"
  echo "全仓唯一允许的版本化协议标识是 QR_V1，请改用 op_tag 或 MODULE_TAG 后重新提交。"
  exit 1
fi

if [[ "${#lint_suppression_hits[@]}" -gt 0 ]]; then
  echo "检测到新增的编译器抑制且缺中文理由："
  printf '  - %s\n' "${lint_suppression_hits[@]}"
  echo "请在 allow 同一行写明中文理由（例如 SCALE 字段序占位），或直接删除死代码后重新提交。"
  exit 1
fi

echo "AI 门禁检查通过。"
if [[ "${#changed_doc_files[@]}" -gt 0 ]]; then
  echo "本次已检测到文档更新："
  printf '  - %s\n' "${changed_doc_files[@]}"
fi
