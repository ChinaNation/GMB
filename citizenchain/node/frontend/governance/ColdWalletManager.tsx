// 冷钱包管理：导入、列表、删除、设置签名管理员。
import { useEffect, useState, useCallback } from 'react';
import { api, sanitizeError } from '../api';
import type { ColdWallet, SigningAdminInfo } from './governance-types';

export function ColdWalletManager() {
  const [wallets, setWallets] = useState<ColdWallet[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  // 导入表单状态
  const [showForm, setShowForm] = useState(false);
  const [address, setAddress] = useState('');
  const [name, setName] = useState('');
  const [password, setPassword] = useState('');
  const [submitting, setSubmitting] = useState(false);
  const [formError, setFormError] = useState<string | null>(null);

  // 删除确认状态
  const [deletingPubkey, setDeletingPubkey] = useState<string | null>(null);
  const [deletePassword, setDeletePassword] = useState('');

  // 签名管理员状态
  const [signingAdmin, setSigningAdmin] = useState<SigningAdminInfo | null>(null);
  const [signingPubkey, setSigningPubkey] = useState<string | null>(null);
  const [signingPrivateKey, setSigningPrivateKey] = useState('');
  const [signingPassword, setSigningPassword] = useState('');
  const [signingSuccess, setSigningSuccess] = useState(false);
  // 签名管理员失效警告
  const [adminExpired, setAdminExpired] = useState(false);

  const loadWallets = useCallback(() => {
    setLoading(true);
    Promise.all([api.getColdWallets(), api.getSigningAdmin()])
      .then(async ([list, admin]) => {
        setWallets(list.wallets);
        setSigningAdmin(admin);
        setError(null);
        // 检测签名管理员是否仍在链上管理员列表中
        if (admin) {
          try {
            const matches = await api.checkAdminWallets(admin.shenfenId);
            const stillValid = matches.some(
              (m) => m.pubkeyHex.toLowerCase() === admin.pubkeyHex.toLowerCase()
            );
            setAdminExpired(!stillValid);
          } catch {
            // 节点未运行时无法查链上，不报警
            setAdminExpired(false);
          }
        } else {
          setAdminExpired(false);
        }
      })
      .catch((e) => setError(sanitizeError(e)))
      .finally(() => setLoading(false));
  }, []);

  useEffect(() => { loadWallets(); }, [loadWallets]);

  // 导入冷钱包
  const handleImport = () => {
    if (!address.trim() || !name.trim() || !password) return;
    setSubmitting(true);
    setFormError(null);
    api.addColdWallet(address.trim(), name.trim(), password)
      .then((list) => {
        setWallets(list.wallets);
        setShowForm(false);
        setAddress('');
        setName('');
        setPassword('');
      })
      .catch((e) => setFormError(sanitizeError(e)))
      .finally(() => setSubmitting(false));
  };

  // 删除冷钱包
  const handleDelete = (pubkeyHex: string) => {
    if (!deletePassword) return;
    setSubmitting(true);
    setFormError(null);
    api.removeColdWallet(pubkeyHex, deletePassword)
      .then((list) => {
        setWallets(list.wallets);
        setDeletingPubkey(null);
        setDeletePassword('');
      })
      .catch((e) => setFormError(sanitizeError(e)))
      .finally(() => setSubmitting(false));
  };

  // 设置签名管理员
  const handleSetSigningAdmin = () => {
    if (!signingPubkey || !signingPrivateKey || !signingPassword) return;
    setSubmitting(true);
    setFormError(null);
    setSigningSuccess(false);
    api.setSigningAdmin(signingPubkey, signingPrivateKey, signingPassword)
      .then((info) => {
        setSigningAdmin(info);
        setSigningPubkey(null);
        setSigningPrivateKey('');
        setSigningPassword('');
        setSigningSuccess(true);
      })
      .catch((e) => setFormError(sanitizeError(e)))
      .finally(() => setSubmitting(false));
  };

  if (loading) {
    return <div className="governance-section"><p>加载钱包…</p></div>;
  }

  if (error) {
    return (
      <div className="governance-section">
        <div className="error">{error}</div>
      </div>
    );
  }

  return (
    <div className="governance-section">
      <div className="wallet-header">
        <h2>冷钱包管理</h2>
        {!showForm && (
          <button className="wallet-add-button" onClick={() => setShowForm(true)}>
            + 导入冷钱包
          </button>
        )}
      </div>

      <p className="wallet-hint">
        冷钱包只存储公钥地址，不存储私钥。治理投票时通过二维码扫码由离线设备签名。
      </p>

      {/* 签名管理员信息 */}
      {signingAdmin && (
        <div className="signing-admin-info">
          <strong>当前签名管理员：</strong>
          <span>{signingAdmin.shenfenName}</span>
          <code style={{ marginLeft: 8 }}>0x{signingAdmin.pubkeyHex.slice(0, 8)}…</code>
        </div>
      )}
      {/* 签名管理员失效警告 */}
      {adminExpired && signingAdmin && (
        <div className="error" style={{ marginTop: 8, padding: '8px 12px', borderRadius: 6 }}>
          ⚠ 签名管理员已失效（已被链上治理更换），请重新设置签名管理员。
        </div>
      )}

      {/* 设置成功提示 */}
      {signingSuccess && (
        <div className="success-message">
          签名管理员设置成功，重启节点后生效。
        </div>
      )}

      {/* 导入表单 */}
      {showForm && (
        <div className="wallet-form">
          <h3>导入冷钱包</h3>
          {formError && <div className="error">{formError}</div>}
          <div className="wallet-form-field">
            <label>地址（SS58 或 0x 公钥）</label>
            <input
              type="text"
              value={address}
              onChange={(e) => setAddress(e.target.value)}
              placeholder="SS58 地址或 0x + 64位十六进制公钥"
              disabled={submitting}
            />
          </div>
          <div className="wallet-form-field">
            <label>名称</label>
            <input
              type="text"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="给钱包起个名字"
              maxLength={50}
              disabled={submitting}
            />
          </div>
          <div className="wallet-form-field">
            <label>设备开机密码</label>
            <input
              type="password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              placeholder="验证身份"
              disabled={submitting}
            />
          </div>
          <div className="wallet-form-actions">
            <button onClick={handleImport} disabled={submitting || !address.trim() || !name.trim() || !password}>
              {submitting ? '导入中…' : '确认导入'}
            </button>
            <button className="cancel-button" onClick={() => { setShowForm(false); setFormError(null); }} disabled={submitting}>
              取消
            </button>
          </div>
        </div>
      )}

      {/* 钱包列表 */}
      {wallets.length === 0 ? (
        <p className="no-data">暂未导入冷钱包</p>
      ) : (
        <div className="wallet-list">
          {wallets.map((w) => (
            <div key={w.pubkeyHex} className="wallet-card">
              <div className="wallet-card-header">
                <span className="wallet-card-name">{w.name}</span>
                <span style={{ display: 'flex', gap: 8, alignItems: 'center' }}>
                  {signingAdmin?.pubkeyHex === w.pubkeyHex ? (
                    <span className="signing-admin-badge">签名账户</span>
                  ) : (
                    <button
                      className="wallet-signing-button"
                      onClick={() => {
                        setSigningPubkey(w.pubkeyHex);
                        setSigningPrivateKey('');
                        setSigningPassword('');
                        setFormError(null);
                        setSigningSuccess(false);
                      }}
                    >
                      设为签名管理员
                    </button>
                  )}
                  <button
                    className="wallet-delete-button"
                    onClick={() => { setDeletingPubkey(w.pubkeyHex); setDeletePassword(''); setFormError(null); }}
                  >
                    删除
                  </button>
                </span>
              </div>
              <div className="wallet-card-address">
                <code>{w.address}</code>
              </div>
              <div className="wallet-card-meta">
                <span className="wallet-card-pubkey">公钥: 0x{w.pubkeyHex}</span>
                <span className="wallet-card-time">
                  {new Date(w.createdAtMs).toLocaleDateString()}
                </span>
              </div>

              {/* 删除确认 */}
              {deletingPubkey === w.pubkeyHex && (
                <div className="wallet-delete-confirm">
                  {formError && <div className="error">{formError}</div>}
                  <p>确认删除钱包 "{w.name}"？需要设备密码验证。</p>
                  <input
                    type="password"
                    value={deletePassword}
                    onChange={(e) => setDeletePassword(e.target.value)}
                    placeholder="设备开机密码"
                    disabled={submitting}
                  />
                  <div className="wallet-form-actions">
                    <button
                      className="danger-button"
                      onClick={() => handleDelete(w.pubkeyHex)}
                      disabled={submitting || !deletePassword}
                    >
                      {submitting ? '删除中…' : '确认删除'}
                    </button>
                    <button className="cancel-button" onClick={() => setDeletingPubkey(null)} disabled={submitting}>
                      取消
                    </button>
                  </div>
                </div>
              )}

              {/* 设置签名管理员对话框 */}
              {signingPubkey === w.pubkeyHex && (
                <div className="wallet-signing-confirm">
                  {formError && <div className="error">{formError}</div>}
                  <p>将 "{w.name}" 设为离线清算签名管理员。需要提供该钱包的私钥种子和设备密码。</p>
                  <div className="wallet-form-field">
                    <label>私钥种子（64 位十六进制）</label>
                    <input
                      type="password"
                      value={signingPrivateKey}
                      onChange={(e) => setSigningPrivateKey(e.target.value)}
                      placeholder="0x 或 64 位十六进制私钥种子"
                      disabled={submitting}
                    />
                  </div>
                  <div className="wallet-form-field">
                    <label>设备开机密码</label>
                    <input
                      type="password"
                      value={signingPassword}
                      onChange={(e) => setSigningPassword(e.target.value)}
                      placeholder="验证身份"
                      disabled={submitting}
                    />
                  </div>
                  <div className="wallet-form-actions">
                    <button
                      onClick={handleSetSigningAdmin}
                      disabled={submitting || !signingPrivateKey || !signingPassword}
                    >
                      {submitting ? '设置中…' : '确认设置'}
                    </button>
                    <button className="cancel-button" onClick={() => { setSigningPubkey(null); setFormError(null); }} disabled={submitting}>
                      取消
                    </button>
                  </div>
                </div>
              )}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
