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

PACKAGE_BASENAME="citizenpassport-ubuntu24-${PACKAGE_ARCH}"
OUT_DIR="${ROOT_DIR}/dist/${PACKAGE_BASENAME}"
PAYLOAD_DIR="${OUT_DIR}/payload"
RUN_FILE="${ROOT_DIR}/dist/${PACKAGE_BASENAME}.run"
SHA256_FILE="${RUN_FILE}.sha256"
INSTALL_GUIDE_MD="${ROOT_DIR}/docs/CITIZENPASSPORT_INSTALL_GUIDE.md"
# 中文注释：行政区唯一源是 CID 维护的 china.sqlite，安装包随附其只读拷贝。
CHINA_DB_SRC="${ROOT_DIR}/../citizencode/backend/china/china.sqlite"

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

      mkdir -p /tmp/citizenpassport-debs
      cd /tmp/citizenpassport-debs
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

TMP_DIR="$(mktemp -d /tmp/citizenpassport-installer.XXXXXX)"
cleanup() {
  rm -rf "${TMP_DIR}"
}
trap cleanup EXIT

ARCHIVE_LINE="$(awk '/^__CITIZENPASSPORT_PAYLOAD_BELOW__$/ {print NR + 1; exit 0}' "$0")"
if [[ -z "${ARCHIVE_LINE}" ]]; then
  echo "ERROR: installer payload marker not found"
  exit 1
fi

tail -n +"${ARCHIVE_LINE}" "$0" | tar -xz -C "${TMP_DIR}"
exec bash "${TMP_DIR}/install_host.sh" "$@"
exit 0
__CITIZENPASSPORT_PAYLOAD_BELOW__
HEADER
  tar -C "${OUT_DIR}" -cz . >>"${RUN_FILE}"
  chmod 0755 "${RUN_FILE}"
  (cd "$(dirname "${RUN_FILE}")" && sha256sum "$(basename "${RUN_FILE}")" >"$(basename "${SHA256_FILE}")")
}

# 中文注释：行政区唯一源是 CID 维护的 china.sqlite；安装包随附其只读拷贝，
# CPMS 运行时用 rusqlite 读，不在 CPMS 源码树保存第二套行政区数据。
echo "[1/8] Build backend release binary"
cargo build --release --manifest-path "${ROOT_DIR}/backend/Cargo.toml"

echo "[2/8] Build frontend static files"
if [[ ! -d "${ROOT_DIR}/frontend/node_modules" ]]; then
  (cd "${ROOT_DIR}/frontend" && npm ci)
fi
(cd "${ROOT_DIR}/frontend" && npm run build)

echo "[3/8] Prepare installer layout"
rm -rf "${OUT_DIR}" "${RUN_FILE}" "${SHA256_FILE}"
if [[ ! -f "${INSTALL_GUIDE_MD}" ]]; then
  echo "ERROR: missing CitizenPassport install guide Markdown at ${INSTALL_GUIDE_MD}"
  exit 1
fi
if [[ ! -f "${CHINA_DB_SRC}" ]]; then
  echo "ERROR: missing china administrative-area source at ${CHINA_DB_SRC}"
  exit 1
fi
# 中文注释：china.sqlite 走 Git LFS。若以 lfs:false / 未拉 LFS 的方式 checkout，
# 该路径会是一个 ~130 字节的“指针文件”，test -f 仍为真但内容不是数据库。
# 正向检测 LFS 指针特征(纯 ASCII，跨平台 grep 都稳)，绝不把指针当数据库打进安装包。
if head -c 64 "${CHINA_DB_SRC}" | grep -qa "git-lfs.github.com"; then
  echo "ERROR: ${CHINA_DB_SRC} 是未拉取的 Git LFS 指针，不是真实数据库。"
  echo "       请先执行: git lfs pull --include=\"citizencode/backend/china/china.sqlite\""
  exit 1
fi
mkdir -p \
  "${PAYLOAD_DIR}/bin" \
  "${PAYLOAD_DIR}/data" \
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
cp "${ROOT_DIR}/backend/target/release/citizenpassport-backend" "${PAYLOAD_DIR}/bin/citizenpassport-backend"
cp "${CHINA_DB_SRC}" "${PAYLOAD_DIR}/data/china.sqlite"
cp "${ROOT_DIR}/deploy/linux/backup_to_storage.sh" "${PAYLOAD_DIR}/bin/backup_to_storage.sh"
cp "${INSTALL_GUIDE_MD}" "${PAYLOAD_DIR}/docs/CitizenPassport安装配置手册.md"
cp -R "${ROOT_DIR}/frontend/dist/." "${PAYLOAD_DIR}/frontend/"
cp "${ROOT_DIR}/deploy/linux/systemd/citizenpassport-backend.service" "${PAYLOAD_DIR}/systemd/citizenpassport-backend.service"
cp "${ROOT_DIR}/deploy/linux/systemd/citizenpassport-backup.service" "${PAYLOAD_DIR}/systemd/citizenpassport-backup.service"
cp "${ROOT_DIR}/deploy/linux/systemd/citizenpassport-backup.timer" "${PAYLOAD_DIR}/systemd/citizenpassport-backup.timer"
cp "${ROOT_DIR}/deploy/linux/nginx/citizenpassport.conf" "${PAYLOAD_DIR}/nginx/citizenpassport.conf"
cp "${ROOT_DIR}/deploy/linux/certs/generate_citizenpassport_certs.sh" "${PAYLOAD_DIR}/certs/generate_citizenpassport_certs.sh"
cp "${ROOT_DIR}/deploy/linux/install_host.sh" "${OUT_DIR}/install_host.sh"
cp "${ROOT_DIR}/deploy/linux/uninstall_host.sh" "${OUT_DIR}/uninstall_host.sh"
cp "${ROOT_DIR}/deploy/linux/install_backup_timer.sh" "${OUT_DIR}/install_backup_timer.sh"

echo "[6/8] Normalize executable bits"
chmod +x \
  "${OUT_DIR}/install_host.sh" \
  "${OUT_DIR}/uninstall_host.sh" \
  "${OUT_DIR}/install_backup_timer.sh" \
  "${PAYLOAD_DIR}/bin/citizenpassport-backend" \
  "${PAYLOAD_DIR}/bin/backup_to_storage.sh" \
  "${PAYLOAD_DIR}/certs/generate_citizenpassport_certs.sh"

echo "[7/8] Validate payload"
test -f "${PAYLOAD_DIR}/manifest.env"
test -f "${PAYLOAD_DIR}/data/china.sqlite"
test -f "${PAYLOAD_DIR}/docs/CitizenPassport安装配置手册.md"
test -f "${PAYLOAD_DIR}/frontend/index.html"
test -f "${PAYLOAD_DIR}/nginx/citizenpassport.conf"
test -f "${PAYLOAD_DIR}/certs/generate_citizenpassport_certs.sh"
compgen -G "${PAYLOAD_DIR}/debs/*.deb" >/dev/null

create_run_installer

echo
echo "Done."
echo "Installer directory: ${OUT_DIR}"
echo "Installer package: ${RUN_FILE}"
echo "Installer checksum: ${SHA256_FILE}"
