#!/usr/bin/env bash
set -euo pipefail

mode="${1:-}"
base_ref="${BASE_REF:-origin/main}"

# 中文注释：全仓禁止中国国旗字符；用 UTF-8 八进制构造，避免门禁源码自身成为命中项。
forbidden_cn_flag="$(printf '\360\237\207\250\360\237\207\263')"
forbidden_cn_flag_hits="$(git grep -n -I -F "${forbidden_cn_flag}" -- . || true)"
if [[ -n "${forbidden_cn_flag_hits}" ]]; then
  echo "检测到禁止使用的中国国旗字符："
  printf '%s\n' "${forbidden_cn_flag_hits}"
  exit 1
fi

if ! git rev-parse --verify "${base_ref}" >/dev/null 2>&1; then
  branch="${base_ref#origin/}"
  git fetch origin "${branch}" --depth=1
fi

merge_base="$(git merge-base HEAD "${base_ref}")"
# 中文注释：本地验收必须包含未提交修改；CI 工作树干净时，该口径与 merge-base...HEAD 等价。
diff_target="${merge_base}"
declare -a changed_files=()
while IFS= read -r file; do
  changed_files+=("${file}")
done < <(git diff --name-only "${diff_target}")

declare -a status_lines=()
while IFS= read -r line; do
  status_lines+=("${line}")
done < <(git diff --name-status --find-renames "${diff_target}")

if [[ "${#changed_files[@]}" -eq 0 && "$mode" != "--startup-only" ]]; then
  echo "未检测到变更文件，跳过 AI 门禁检查。"
  exit 0
fi

# 中文注释：GitHub runner 不能读取被 Git 忽略的本机脚本，因此门禁直接验证已跟踪的启动协议真源。
require_startup_file() {
  local path="$1"

  if [[ ! -e "$path" ]]; then
    echo "缺少启动协议文件：$path" >&2
    exit 1
  fi
}

require_startup_symlink_target() {
  local path="$1"
  local target="$2"
  local actual

  if [[ ! -L "$path" ]]; then
    echo "根目录入口必须保持为软链接：$path" >&2
    exit 1
  fi

  actual="$(readlink "$path")"
  if [[ "$actual" != "$target" ]]; then
    echo "根目录入口指向错误：$path -> $actual（期望 $target）" >&2
    exit 1
  fi
}

require_startup_text() {
  local path="$1"
  local expected_text="$2"

  if ! grep -Fq "$expected_text" "$path"; then
    echo "启动协议缺少关键语句：$path -> $expected_text" >&2
    exit 1
  fi
}

require_startup_file "memory/AGENTS.md"
require_startup_file "memory/CODEX.md"
require_startup_file "memory/CLAUDE.md"
require_startup_file "memory/07-ai/chat-protocol.md"
require_startup_file "memory/07-ai/startup-acceptance.md"
require_startup_file "memory/07-ai/document-boundaries.md"

require_startup_symlink_target "AGENTS.md" "memory/AGENTS.md"
require_startup_symlink_target "CODEX.md" "memory/CODEX.md"
require_startup_symlink_target "CLAUDE.md" "memory/CLAUDE.md"

require_startup_text "memory/AGENTS.md" "第一轮必须先做需求分析"
require_startup_text "memory/AGENTS.md" "任务卡"
require_startup_text "memory/AGENTS.md" "检查为什么报错"
require_startup_text "memory/CODEX.md" "第一轮必须输出需求分析"
require_startup_text "memory/CODEX.md" "检查为什么报错"
require_startup_text "memory/07-ai/chat-protocol.md" "需求分析"
require_startup_text "memory/07-ai/chat-protocol.md" "检查为什么报错"
echo "启动协议检查通过。"

if [[ "$mode" == "--startup-only" ]]; then
  exit 0
fi

doc_regex='^(memory/|docs/|README\.md$|GMB_TECHNICAL\.md$|CLAUDE\.md$|\.github/pull_request_template\.md$|.*_TECHNICAL\.md$)'
code_regex='^(\.github/workflows/|scripts/|citizenchain/|citizenapp/|primitives/|Cargo\.toml$|Cargo\.lock$|.*\.(rs|dart|ts|tsx|js|jsx|sh|py|sql|toml|ya?ml|json|swift|kt|kts))'
scan_regex='^(scripts/|citizenchain/|citizenapp/|primitives/|.*\.(rs|dart|ts|tsx|js|jsx|sh|py|sql|toml))'
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
    .github/workflows/*|.github/scripts/check-ai-guardrails.sh|scripts/analyze-requirement.sh|scripts/architect-entry.sh|scripts/complete-task.sh|scripts/index-tasks.sh|scripts/load-context.sh|scripts/module-router.sh|scripts/new-task.sh|scripts/start-task.sh)
      printf '%s' "memory/07-ai/"
      ;;
    citizenchain/*|primitives/*|Cargo.toml|Cargo.lock)
      printf '%s' "memory/05-modules/citizenchain/"
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
    .github/workflows/*|.github/scripts/check-ai-guardrails.sh|scripts/analyze-requirement.sh|scripts/architect-entry.sh|scripts/complete-task.sh|scripts/index-tasks.sh|scripts/load-context.sh|scripts/module-router.sh|scripts/new-task.sh|scripts/start-task.sh)
      has_changed_doc_file "memory/01-architecture/repo-map.md" && return 0
      ;;
    citizenchain/*|primitives/*|Cargo.toml|Cargo.lock)
      has_changed_doc_file "memory/01-architecture/citizenchain-target-structure.md" && return 0
      has_changed_doc_file "memory/01-architecture/repo-map.md" && return 0
      has_changed_doc_file "memory/03-security/security-rules.md" && return 0
      ;;
    citizenapp/*)
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
    */test/*|*/tests/*|*.g.dart|*.pb.dart|*.pbjson.dart|*.pbenum.dart|*/GeneratedPluginRegistrant.*|citizenapp/cloudflare/worker-configuration.d.ts|citizenchain/onchina/frontend/dist/assets/*.js)
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

  # 中文注释：统一使用 ERE 匹配字面加号，兼容 macOS BSD grep 与 GNU grep。
  added_lines="$(git diff --unified=0 "${diff_target}" -- "$file" \
    | grep -E '^\+' \
    | grep -vE '^\+\+\+' \
    | grep -E '([A-Za-z0-9][._:-]v[0-9]+|/(api/)?v[0-9]+|[A-Za-z0-9]_V[0-9]+|schema_version|cache_version|protocol_version|tag[[:space:]]*=[[:space:]]*["]v[0-9]+)' \
    || true)"

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

# 中文注释：协议版本标识门禁。全仓自定义协议只有 QR_V1 可以带版本号；
# 一方 API、schema、缓存键、字符串域和文件路径均不得另造版本后缀。
sanitize_official_version_names() {
  local line="$1"

  line="${line//QR_V1/}"
  line="${line//QrProtocol.qrV1/}"
  line="${line//QrProtocols.qrV1/}"
  line="$(printf '%s\n' "$line" | sed -E \
    -e 's/Uuid::new_v[45]//g' \
    -e 's/arm64-v8a//g' \
    -e 's/armeabi-v7a//g' \
    -e 's/libbarhopper_v[0-9]+//g' \
    -e 's/RSASSA-PKCS1-v1_5//g' \
    -e 's/sc-rpc-spec-v2//g')"

  # 中文注释：只消去明确的第三方官方 URL 版本段，绝不豁免任意 https URL。
  if [[ "$line" == *"https://api.cloudflare.com/"* ]]; then
    line="$(printf '%s\n' "$line" | sed -E \
      -e 's#https://api\.cloudflare\.com/client/v[0-9]+#https://api.cloudflare.com/client#g' \
      -e 's#/images/v[0-9]+#/images#g')"
  fi
  if [[ "$line" == *"https://challenges.cloudflare.com/"* ]]; then
    line="$(printf '%s\n' "$line" | sed -E \
      's#https://challenges\.cloudflare\.com/turnstile/v[0-9]+#https://challenges.cloudflare.com/turnstile#g')"
  fi
  if [[ "$line" == *"https://fcm.googleapis.com/"* ]]; then
    line="$(printf '%s\n' "$line" | sed -E \
      's#https://fcm\.googleapis\.com/v[0-9]+#https://fcm.googleapis.com#g')"
  fi
  if [[ "$line" == *"https://docs.substrate.io/"* ]]; then
    line="$(printf '%s\n' "$line" | sed -E \
      's#https://docs\.substrate\.io/[^ )]*/v[0-9]+#https://docs.substrate.io/official#g')"
  fi

  printf '%s\n' "$line"
}

has_custom_version_name() {
  local line
  line="$(sanitize_official_version_names "$1")"
  printf '%s\n' "$line" | grep -Eq \
    '([A-Za-z0-9][._:-]v[0-9]+|/(api/)?v[0-9]+(/|([^A-Za-z0-9]|$))|[A-Za-z0-9]_V[0-9]+|(^|[^A-Za-z0-9_])(schema_version|cache_version|protocol_version)([^A-Za-z0-9_]|$)|tag[[:space:]]*=[[:space:]]*["]v[0-9]+)'
}

check_version_tag_gate() {
  local file="$1"
  local added_lines
  local line

  case "$file" in
    citizenapp/smoldotpow/*|citizenapp/cloudflare/worker-configuration.d.ts|citizenapp/assets/topup/walletconnect.bundle.js|citizenchain/onchina/frontend/dist/*|citizenchain/node/frontend/dist/*) return 0 ;;
    *.rs|*.dart|*.ts|*.tsx|*.js|*.jsx|*.swift|*.kt|*.kts|*.proto|*.sql|*.ya?ml|*.json|*.toml|*.sh|*.py|*.md) ;;
    *) return 0 ;;
  esac

  # 中文注释：`\+` 在 BRE 下是重复算子，必须用 -E（ERE）才是字面加号，否则严格 grep 直接报错。
  added_lines="$(git diff --unified=0 "${diff_target}" -- "$file" | grep -E '^\+' | grep -vE '^\+\+\+' || true)"

  if [[ -z "$added_lines" ]]; then
    return 0
  fi

  while IFS= read -r line; do
    if has_custom_version_name "$line"; then
      version_tag_hits+=("${file}: 出现非 QR_V1 的一方版本化标识或路由")
      return
    fi
  done <<< "$added_lines"
}

# 中文注释：不仅拦新增行，还扫描全部受控真源和文件路径，防止历史协议残留长期躲藏。
check_repository_version_tags() {
  local file
  local line

  # 中文注释：先由 git grep 预筛候选，再逐条消去官方名称；禁止对全仓每一行启动子进程。
  while IFS= read -r line; do
    # 中文注释：Cloudflare legacy Durable Objects migrations 的 tag 是官方生命周期迁移字段，
    # 不是 GMB 业务协议标识；只豁免当前唯一生产 wrangler 文件中的精确既有迁移标签。
    if [[ "$line" =~ ^citizenapp/cloudflare/wrangler\.toml:[0-9]+:tag[[:space:]]*=[[:space:]]*\"v1\"$ ]]; then
      continue
    fi
    if has_custom_version_name "$line"; then
      version_tag_hits+=("${line}")
    fi
  done < <(
    git grep --untracked -n -I -E \
      '([A-Za-z0-9][._:-]v[0-9]+|/(api/)?v[0-9]+|[A-Za-z0-9]_V[0-9]+|schema_version|cache_version|protocol_version|tag[[:space:]]*=[[:space:]]*["]v[0-9]+)' \
      -- \
      ':!citizenapp/smoldotpow/**' \
      ':!citizenapp/cloudflare/worker-configuration.d.ts' \
      ':!citizenapp/assets/topup/walletconnect.bundle.js' \
      ':!**/package-lock.json' \
      ':!docs/logo.svg' \
      ':!citizenchain/node/src/core/constitution/constitution_shell.html' \
      ':!citizenchain/onchina/frontend/dist/**' \
      ':!citizenchain/node/frontend/dist/**' \
      ':!.github/scripts/check-ai-guardrails.sh' \
      || true
  )

  # 中文注释：路径单独扫描，拦截源码或文档通过文件名恢复版本后缀。
  while IFS= read -r file; do
    # 中文注释：状态中已删除的旧路径不是仓库目标态；Android ABI 与资源限定符属于官方目录语法。
    [[ -e "$file" ]] || continue
    case "$file" in
      citizenapp/smoldotpow/*|citizenapp/cloudflare/worker-configuration.d.ts|citizenapp/assets/topup/walletconnect.bundle.js|citizenchain/onchina/frontend/dist/*|citizenchain/node/frontend/dist/*|*/target/*|*/build/*|*/android/app/src/main/jniLibs/arm64-v8a/*|*/android/app/src/main/res/*-v[0-9]*/*|.github/scripts/check-ai-guardrails.sh) continue ;;
    esac
    if printf '%s\n' "$file" | grep -Eq '(^|/)[^/]*[._-]v[0-9]+([^0-9]|$)'; then
      version_tag_hits+=("${file}: 文件路径含非 QR_V1 的版本后缀")
    fi
  done < <(git ls-files -co --exclude-standard)
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

  blocks="$(git diff --unified=2 "${diff_target}" -- "$file" \
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
    .github/scripts/check-ai-guardrails.sh)
      return 0
      ;;
    # 中文注释:AI 工作流脚本已统一收敛到根 scripts/(原 memory/scripts/),逐个保护,避免误伤同目录通用工具脚本。
    scripts/analyze-requirement.sh|scripts/architect-entry.sh|\
    scripts/complete-task.sh|scripts/index-tasks.sh|scripts/load-context.sh|\
    scripts/module-router.sh|scripts/new-task.sh|scripts/start-task.sh)
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
    # 中文注释：固定版本的 WalletConnect 浏览器 bundle 是第三方生成物，只跳过开发残留关键字扫描。
    citizenapp/assets/topup/walletconnect.bundle.js)
      return 0
      ;;
    # 中文注释：Wrangler 固定生成类型含 Web API 英文注释和日志示例，不按手写源码规则扫描。
    citizenapp/cloudflare/worker-configuration.d.ts)
      return 0
      ;;
    # 中文注释：OnChina dist 哈希资产由 Vite 压缩生成，源码注释与残留门禁应检查其源文件。
    citizenchain/onchina/frontend/dist/assets/*.js)
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
      # 中文注释：机构岗位权限 fixture 已获明确批准去掉版本后缀；目标仍在同一受保护目录。
      if [[ "${old_path}" == "memory/06-quality/fixtures/institution_role_permission_v1.json" \
        && -f "memory/06-quality/fixtures/institution_role_permission.json" ]]; then
        continue
      fi
      if is_protected_ai_path "${old_path}"; then
        protected_ai_hits+=("禁止删除 AI 编程系统核心基础设施: ${old_path}")
      fi
      ;;
    R*)
      # 中文注释：同一受保护体系内的明确重命名允许；只禁止把基础设施迁出保护范围。
      if is_protected_ai_path "${old_path}" && ! is_protected_ai_path "${new_path}"; then
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
  "citizenapp/lib/wallet/core/wallet_manager.dart|miniSecretFromEntropy|助记词到 account_root_seed 派生"
  "citizenwallet/lib/wallet/wallet_manager.dart|miniSecretFromEntropy|助记词到 account_root_seed 派生"
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
  anchor_diff="$(git diff "${diff_target}" -- "${anchor_file}" || true)"
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

if [[ "${#changed_code_files[@]}" -gt 0 ]]; then
  for file in "${changed_code_files[@]}"; do
    if ! has_matching_module_doc_update "$file"; then
      required_prefix="$(module_doc_requirement_for_file "$file")"
      if [[ -n "$required_prefix" ]]; then
        missing_module_doc_hits+=("${file}: 缺少对应模块文档更新（期望更新 ${required_prefix}）")
      fi
    fi
  done
fi

if [[ "${#missing_module_doc_hits[@]}" -gt 0 ]]; then
  echo "检测到更细粒度的文档回写缺失。"
  echo "以下代码变更没有同步到对应模块文档："
  printf '  - %s\n' "${missing_module_doc_hits[@]}"
  exit 1
fi

if [[ "${#changed_code_files[@]}" -gt 0 ]]; then
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
fi

rm -f /tmp/gmb_guardrail_hit.txt

check_repository_version_tags

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
