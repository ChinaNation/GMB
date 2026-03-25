// 冷钱包管理：导入、列表、删除。
import { useEffect, useState, useCallback } from 'react';
import { api, sanitizeError } from '../api';
import type { ColdWallet } from './governance-types';

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

  const loadWallets = useCallback(() => {
    setLoading(true);
    api.getColdWallets()
      .then((list) => {
        setWallets(list.wallets);
        setError(null);
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
                <button
                  className="wallet-delete-button"
                  onClick={() => { setDeletingPubkey(w.pubkeyHex); setDeletePassword(''); setFormError(null); }}
                >
                  删除
                </button>
              </div>
              <div className="wallet-card-address">
                <code>{w.address}</code>
              </div>
              <div className="wallet-card-meta">
                <span className="wallet-card-pubkey">公钥: 0x{w.pubkeyHex.slice(0, 16)}…</span>
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
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
