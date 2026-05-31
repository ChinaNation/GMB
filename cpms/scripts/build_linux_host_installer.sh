#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PACKAGE_ARCH="amd64"
RUNTIME_PACKAGES=(
  postgresql
  postgresql-client
  nginx
  openssl
  ca-certificates
  rsync
  openssh-client
)

usage() {
  echo "Usage: $0 [--arch amd64|arm64]"
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --arch)
      PACKAGE_ARCH="${2:-}"
      shift 2
      ;;
    --arch=*)
      PACKAGE_ARCH="${1#--arch=}"
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "ERROR: unknown argument: $1"
      usage
      exit 1
      ;;
  esac
done

case "${PACKAGE_ARCH}" in
  amd64)
    DOCKER_PLATFORM="linux/amd64"
    ;;
  arm64)
    DOCKER_PLATFORM="linux/arm64"
    ;;
  *)
    echo "ERROR: unsupported package arch: ${PACKAGE_ARCH}"
    usage
    exit 1
    ;;
esac

PACKAGE_BASENAME="cpms-ubuntu24-${PACKAGE_ARCH}"
OUT_DIR="${ROOT_DIR}/dist/${PACKAGE_BASENAME}"
PAYLOAD_DIR="${OUT_DIR}/payload"
RUN_FILE="${ROOT_DIR}/dist/${PACKAGE_BASENAME}.run"
SHA256_FILE="${RUN_FILE}.sha256"
INSTALL_GUIDE_PDF="${ROOT_DIR}/docs/CPMS安装配置手册.pdf"

collect_offline_debs() {
  if ! command -v docker >/dev/null 2>&1; then
    echo "ERROR: ${PACKAGE_BASENAME}.run must be built with Docker available."
    exit 1
  fi

  echo "[4/8] Download Ubuntu 24.04 ${PACKAGE_ARCH} offline deb closure in clean container"
  mkdir -p "${PAYLOAD_DIR}/debs"
  rm -f "${PAYLOAD_DIR}/debs"/*.deb

  # 中文注释：依赖闭包必须在官方 ubuntu:24.04 容器内解析，不能读取 GitHub runner 主机 apt 源。
  docker run --rm --platform "${DOCKER_PLATFORM}" \
    -e CPMS_RUNTIME_PACKAGES="${RUNTIME_PACKAGES[*]}" \
    -e HOST_UID="$(id -u)" \
    -e HOST_GID="$(id -g)" \
    -v "${PAYLOAD_DIR}/debs:/out" \
    ubuntu:24.04 \
    bash -lc '
      set -euo pipefail
      export DEBIAN_FRONTEND=noninteractive

      apt-get update

      candidate_for() {
        apt-cache policy "$1" | awk "/Candidate:/ {print \$2; exit}"
      }

      deps_for() {
        apt-cache depends \
          --no-recommends \
          --no-suggests \
          --no-conflicts \
          --no-breaks \
          --no-replaces \
          --no-enhances \
          "$1" \
        | sed -n "s/^[[:space:]]*PreDepends:[[:space:]]*//p; s/^[[:space:]]*Depends:[[:space:]]*//p" \
        | sed "s/[<>]//g; s/|//g" \
        | awk "{print \$1}" \
        | sed "s/:any$//" \
        | sort -u
      }

      read -r -a queue <<<"${CPMS_RUNTIME_PACKAGES}"
      declare -A seen=()
      resolved=()

      while ((${#queue[@]} > 0)); do
        pkg="${queue[0]}"
        queue=("${queue[@]:1}")
        pkg="${pkg#<}"
        pkg="${pkg%>}"
        pkg="${pkg%:any}"
        [[ -z "${pkg}" || -n "${seen[$pkg]:-}" ]] && continue

        candidate="$(candidate_for "${pkg}")"
        if [[ -z "${candidate}" || "${candidate}" == "(none)" ]]; then
          continue
        fi

        seen["${pkg}"]=1
        resolved+=("${pkg}")

        while read -r dep; do
          [[ -n "${dep}" ]] && queue+=("${dep}")
        done < <(deps_for "${pkg}")
      done

      mkdir -p /tmp/cpms-debs
      cd /tmp/cpms-debs
      apt-get download "${resolved[@]}"
      mv ./*.deb /out/
      chown -R "${HOST_UID}:${HOST_GID}" /out
    '
}

create_run_installer() {
  echo "[8/8] Create ${PACKAGE_BASENAME}.run"
  rm -f "${RUN_FILE}" "${SHA256_FILE}"
  cat >"${RUN_FILE}" <<'HEADER'
#!/usr/bin/env bash
set -euo pipefail

TMP_DIR="$(mktemp -d /tmp/cpms-installer.XXXXXX)"
cleanup() {
  rm -rf "${TMP_DIR}"
}
trap cleanup EXIT

ARCHIVE_LINE="$(awk '/^__CPMS_PAYLOAD_BELOW__$/ {print NR + 1; exit 0}' "$0")"
if [[ -z "${ARCHIVE_LINE}" ]]; then
  echo "ERROR: installer payload marker not found"
  exit 1
fi

tail -n +"${ARCHIVE_LINE}" "$0" | tar -xz -C "${TMP_DIR}"
exec bash "${TMP_DIR}/install_host.sh" "$@"
exit 0
__CPMS_PAYLOAD_BELOW__
HEADER
  tar -C "${OUT_DIR}" -cz . >>"${RUN_FILE}"
  chmod 0755 "${RUN_FILE}"
  (cd "$(dirname "${RUN_FILE}")" && sha256sum "$(basename "${RUN_FILE}")" >"$(basename "${SHA256_FILE}")")
}

# 中文注释：行政区数据由 CPMS 后端编译期直接引用 sfid/backend/sfid 唯一源，
# 打包脚本不得再把 province.rs 或 city_codes 写入 CPMS 源码树。
echo "[1/8] Build backend release binary"
cargo build --release --manifest-path "${ROOT_DIR}/backend/Cargo.toml"

echo "[2/8] Build frontend static files"
if [[ ! -d "${ROOT_DIR}/frontend/node_modules" ]]; then
  (cd "${ROOT_DIR}/frontend" && npm ci)
fi
(cd "${ROOT_DIR}/frontend" && npm run build)

echo "[3/8] Prepare installer layout"
rm -rf "${OUT_DIR}" "${RUN_FILE}" "${SHA256_FILE}"
if [[ ! -f "${INSTALL_GUIDE_PDF}" ]]; then
  echo "ERROR: missing CPMS install guide PDF at ${INSTALL_GUIDE_PDF}"
  exit 1
fi
mkdir -p \
  "${PAYLOAD_DIR}/bin" \
  "${PAYLOAD_DIR}/docs" \
  "${PAYLOAD_DIR}/frontend" \
  "${PAYLOAD_DIR}/systemd" \
  "${PAYLOAD_DIR}/nginx" \
  "${PAYLOAD_DIR}/certs"
cat >"${PAYLOAD_DIR}/manifest.env" <<EOF
CPMS_PACKAGE_NAME=${PACKAGE_BASENAME}.run
CPMS_PACKAGE_OS=ubuntu24
CPMS_PACKAGE_ARCH=${PACKAGE_ARCH}
EOF

collect_offline_debs

echo "[5/8] Copy payload files"
cp "${ROOT_DIR}/backend/target/release/cpms-backend" "${PAYLOAD_DIR}/bin/cpms-backend"
cp "${ROOT_DIR}/deploy/linux/backup_to_storage.sh" "${PAYLOAD_DIR}/bin/backup_to_storage.sh"
cp "${INSTALL_GUIDE_PDF}" "${PAYLOAD_DIR}/docs/CPMS安装配置手册.pdf"
cp -R "${ROOT_DIR}/frontend/dist/." "${PAYLOAD_DIR}/frontend/"
cp "${ROOT_DIR}/deploy/linux/systemd/cpms-backend.service" "${PAYLOAD_DIR}/systemd/cpms-backend.service"
cp "${ROOT_DIR}/deploy/linux/systemd/cpms-backup.service" "${PAYLOAD_DIR}/systemd/cpms-backup.service"
cp "${ROOT_DIR}/deploy/linux/systemd/cpms-backup.timer" "${PAYLOAD_DIR}/systemd/cpms-backup.timer"
cp "${ROOT_DIR}/deploy/linux/nginx/cpms.conf" "${PAYLOAD_DIR}/nginx/cpms.conf"
cp "${ROOT_DIR}/deploy/linux/certs/generate_cpms_certs.sh" "${PAYLOAD_DIR}/certs/generate_cpms_certs.sh"
cp "${ROOT_DIR}/deploy/linux/install_host.sh" "${OUT_DIR}/install_host.sh"
cp "${ROOT_DIR}/deploy/linux/uninstall_host.sh" "${OUT_DIR}/uninstall_host.sh"
cp "${ROOT_DIR}/deploy/linux/install_backup_timer.sh" "${OUT_DIR}/install_backup_timer.sh"

echo "[6/8] Normalize executable bits"
chmod +x \
  "${OUT_DIR}/install_host.sh" \
  "${OUT_DIR}/uninstall_host.sh" \
  "${OUT_DIR}/install_backup_timer.sh" \
  "${PAYLOAD_DIR}/bin/cpms-backend" \
  "${PAYLOAD_DIR}/bin/backup_to_storage.sh" \
  "${PAYLOAD_DIR}/certs/generate_cpms_certs.sh"

echo "[7/8] Validate payload"
test -f "${PAYLOAD_DIR}/manifest.env"
test -f "${PAYLOAD_DIR}/docs/CPMS安装配置手册.pdf"
test -f "${PAYLOAD_DIR}/frontend/index.html"
test -f "${PAYLOAD_DIR}/nginx/cpms.conf"
test -f "${PAYLOAD_DIR}/certs/generate_cpms_certs.sh"
compgen -G "${PAYLOAD_DIR}/debs/*.deb" >/dev/null

create_run_installer

echo
echo "Done."
echo "Installer directory: ${OUT_DIR}"
echo "Installer package: ${RUN_FILE}"
echo "Installer checksum: ${SHA256_FILE}"
