#!/usr/bin/env bash
set -euo pipefail

CERT_DIR="/etc/cpms/certs"
ROOT_KEY="${CERT_DIR}/cpms-root-ca.key"
ROOT_CERT="${CERT_DIR}/cpms-root-ca.crt"
SERVER_KEY="${CERT_DIR}/www.cpms.com.key"
SERVER_CSR="${CERT_DIR}/www.cpms.com.csr"
SERVER_CERT="${CERT_DIR}/www.cpms.com.crt"
SERVER_EXT="${CERT_DIR}/www.cpms.com.ext"

if [[ "${EUID}" -ne 0 ]]; then
  echo "ERROR: please run as root"
  exit 1
fi

install -d -m 0750 "${CERT_DIR}"

if [[ ! -f "${ROOT_KEY}" || ! -f "${ROOT_CERT}" ]]; then
  # 中文注释：离线局域网部署使用本机私有 CA；客户端需信任该根证书。
  openssl genrsa -out "${ROOT_KEY}" 4096
  openssl req -x509 -new -nodes \
    -key "${ROOT_KEY}" \
    -sha256 \
    -days 3650 \
    -subj "/C=GM/O=GMB/OU=CPMS/CN=CPMS Local Root CA" \
    -out "${ROOT_CERT}"
fi

openssl genrsa -out "${SERVER_KEY}" 2048
openssl req -new \
  -key "${SERVER_KEY}" \
  -subj "/C=GM/O=GMB/OU=CPMS/CN=www.cpms.com" \
  -out "${SERVER_CSR}"

cat >"${SERVER_EXT}" <<'EOF'
authorityKeyIdentifier=keyid,issuer
basicConstraints=CA:FALSE
keyUsage=digitalSignature,keyEncipherment
extendedKeyUsage=serverAuth
subjectAltName=@alt_names

[alt_names]
DNS.1=www.cpms.com
EOF

openssl x509 -req \
  -in "${SERVER_CSR}" \
  -CA "${ROOT_CERT}" \
  -CAkey "${ROOT_KEY}" \
  -CAcreateserial \
  -out "${SERVER_CERT}" \
  -days 3650 \
  -sha256 \
  -extfile "${SERVER_EXT}"

rm -f "${SERVER_CSR}" "${SERVER_EXT}"
chmod 0600 "${ROOT_KEY}" "${SERVER_KEY}"
chmod 0644 "${ROOT_CERT}" "${SERVER_CERT}"
chown -R root:root "${CERT_DIR}"

echo "CPMS certificates generated under ${CERT_DIR}"
