#!/usr/bin/env bash
set -euo pipefail

# 中文注释：所有部署 Secret 使用固定服务名和“环境:字段”账户名存入 macOS Keychain。
SERVICE='GMB Deploy'
command_name="${1:-}"
environment="${2:-}"
secret_name="${3:-}"
account="${environment}:${secret_name}"

case "$command_name" in
  put)
    IFS= read -r secret_value
    [[ -n "$environment" && -n "$secret_name" && -n "$secret_value" ]] || exit 2
    security add-generic-password -U -a "$account" -s "$SERVICE" -w "$secret_value" >/dev/null
    ;;
  put-multiline)
    # 中文注释：macOS security 对原始换行值会返回十六进制文本；多行 Secret 先封装为单行 Base64。
    secret_value=''
    IFS= read -r -d '' secret_value || true
    [[ -n "$environment" && -n "$secret_name" && -n "$secret_value" ]] || exit 2
    encoded_value="$(printf '%s' "$secret_value" | base64)"
    security add-generic-password -U -a "$account" -s "$SERVICE" -w "base64:$encoded_value" >/dev/null
    ;;
  get)
    [[ -n "$environment" && -n "$secret_name" ]] || exit 2
    security find-generic-password -a "$account" -s "$SERVICE" -w
    ;;
  get-multiline)
    [[ -n "$environment" && -n "$secret_name" ]] || exit 2
    encoded_value="$(security find-generic-password -a "$account" -s "$SERVICE" -w)"
    [[ "$encoded_value" == base64:* ]] || exit 3
    printf '%s' "${encoded_value#base64:}" | base64 --decode
    ;;
  exists)
    [[ -n "$environment" && -n "$secret_name" ]] || exit 2
    security find-generic-password -a "$account" -s "$SERVICE" >/dev/null 2>&1
    ;;
  delete)
    [[ -n "$environment" && -n "$secret_name" ]] || exit 2
    security delete-generic-password -a "$account" -s "$SERVICE" >/dev/null
    ;;
  *)
    echo '用法: keychain.sh put|put-multiline|get|get-multiline|exists|delete <环境> <字段>' >&2
    exit 2
    ;;
esac
