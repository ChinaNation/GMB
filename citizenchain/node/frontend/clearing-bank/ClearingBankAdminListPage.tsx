// 清算行 tab 专用管理员列表页:列表行有"激活"+"解密"两个按钮 + 状态绿点。
//
// "激活"复用治理流程(activate_admin),保留 activated-admins.json 持久化记录。
// "解密"是清算行 tab 独有的语义:wumin 签 challenge → 节点验签 → 把 (pubkey, sfid_id)
//          标记为内存内"已解密",packer 攒批前 cross-check 该入口存在才会签名提交。
//          内存状态节点重启自动清空,无 TTL。
//
// 此组件**不**复用治理 AdminListPage,以避免污染 NRC/PRC/PRB 的"激活"语义。

import { useCallback, useEffect, useRef, useState } from 'react';
import { QRCodeSVG } from 'qrcode.react';
import { api, sanitizeError } from '../api';
import { hexToSs58, formatBalance } from '../format';
import { QrScanner } from '../governance/QrScanner';
import type {
  ActivatedAdmin,
  InstitutionDetail,
} from '../governance/governance-types';
import type { DecryptAdminRequestResult, DecryptedAdminInfo } from './clearing-bank-types';

type Props = {
  shenfenId: string;
  onBack: () => void;
};

type ActivateStep = 'idle' | 'qr' | 'scan' | 'verifying' | 'done' | 'error';
type DecryptStep = 'idle' | 'qr' | 'scan' | 'verifying' | 'done' | 'error';

export function ClearingBankAdminListPage({ shenfenId, onBack }: Props) {
  const [detail, setDetail] = useState<InstitutionDetail | null>(null);
  const [activated, setActivated] = useState<ActivatedAdmin[]>([]);
  const [decrypted, setDecrypted] = useState<DecryptedAdminInfo[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  // 激活流程
  const [actStep, setActStep] = useState<ActivateStep>('idle');
  const [actPubkey, setActPubkey] = useState('');
  const [actRequest, setActRequest] = useState<{ requestJson: string; requestId: string; expectedPayloadHash: string; payloadHex: string } | null>(null);
  const [actCountdown, setActCountdown] = useState(90);
  const [actError, setActError] = useState<string | null>(null);

  // 解密流程
  const [decStep, setDecStep] = useState<DecryptStep>('idle');
  const [decPubkey, setDecPubkey] = useState('');
  const [decRequest, setDecRequest] = useState<DecryptAdminRequestResult | null>(null);
  const [decCountdown, setDecCountdown] = useState(90);
  const [decError, setDecError] = useState<string | null>(null);
  const decRequestRef = useRef(decRequest);
  decRequestRef.current = decRequest;

  const refresh = useCallback(() => {
    setLoading(true);
    Promise.all([
      api.getInstitutionDetail(shenfenId),
      api.getActivatedAdmins(shenfenId).catch(() => [] as ActivatedAdmin[]),
      api.listDecryptedAdmins(shenfenId).catch(() => [] as DecryptedAdminInfo[]),
    ])
      .then(([d, aa, dd]) => {
        setDetail(d);
        setActivated(aa);
        setDecrypted(dd);
        setError(null);
      })
      .catch((e) => setError(sanitizeError(e)))
      .finally(() => setLoading(false));
  }, [shenfenId]);

  useEffect(() => { refresh(); }, [refresh]);

  // 倒计时
  useEffect(() => {
    if (actStep !== 'qr') return;
    if (actCountdown <= 0) { setActError('请求已过期'); setActStep('error'); return; }
    const t = setTimeout(() => setActCountdown(c => c - 1), 1000);
    return () => clearTimeout(t);
  }, [actStep, actCountdown]);
  useEffect(() => {
    if (decStep !== 'qr') return;
    if (decCountdown <= 0) { setDecError('请求已过期'); setDecStep('error'); return; }
    const t = setTimeout(() => setDecCountdown(c => c - 1), 1000);
    return () => clearTimeout(t);
  }, [decStep, decCountdown]);

  const startActivation = useCallback(async (pubkey: string) => {
    setActPubkey(pubkey); setActError(null);
    try {
      const r = await api.buildActivateAdminRequest(pubkey, shenfenId);
      setActRequest(r); setActCountdown(90); setActStep('qr');
    } catch (e) { setActError(sanitizeError(e)); setActStep('error'); }
  }, [shenfenId]);

  const handleActivateScan = useCallback(async (responseJson: string) => {
    if (!actRequest) return;
    setActStep('verifying');
    try {
      await api.verifyActivateAdmin(
        actRequest.requestId, actPubkey, actRequest.expectedPayloadHash,
        actRequest.payloadHex, responseJson,
      );
      setActStep('done');
      setTimeout(() => { setActStep('idle'); refresh(); }, 1500);
    } catch (e) { setActError(sanitizeError(e)); setActStep('error'); }
  }, [actRequest, actPubkey, refresh]);

  const startDecrypt = useCallback(async (pubkey: string) => {
    setDecPubkey(pubkey); setDecError(null);
    try {
      const r = await api.buildDecryptAdminRequest(pubkey, shenfenId);
      setDecRequest(r); setDecCountdown(90); setDecStep('qr');
    } catch (e) { setDecError(sanitizeError(e)); setDecStep('error'); }
  }, [shenfenId]);

  const handleDecryptScan = useCallback(async (responseJson: string) => {
    const r = decRequestRef.current;
    if (!r) return;
    setDecStep('verifying');
    try {
      await api.verifyAndDecryptAdmin(
        r.requestId, decPubkey, r.expectedPayloadHash, responseJson,
      );
      setDecStep('done');
      setTimeout(() => { setDecStep('idle'); refresh(); }, 1500);
    } catch (e) { setDecError(sanitizeError(e)); setDecStep('error'); }
  }, [decPubkey, refresh]);

  const lockAdmin = useCallback(async (pubkeyHex: string) => {
    try {
      await api.lockDecryptedAdmin(pubkeyHex);
      refresh();
    } catch (e) {
      setError(sanitizeError(e));
    }
  }, [refresh]);

  if (loading) return <div className="governance-section"><p>加载中…</p></div>;
  if (error || !detail) {
    return (
      <div className="governance-section">
        <button className="back-button" onClick={onBack}>← 返回</button>
        {error && <div className="error">{error}</div>}
      </div>
    );
  }

  return (
    <div className="governance-section">
      <button className="back-button" onClick={onBack}>← 返回机构详情</button>

      <div className="admin-list-header">
        <h2>清算行管理员</h2>
        <span className="admin-list-summary">
          共 {detail.admins.length} 人 · 已激活 {activated.length} · 已解密 {decrypted.length}
        </span>
      </div>

      {detail.admins.length === 0 ? (
        <p className="no-data">暂无管理员</p>
      ) : (
        <div className="admin-grid">
          {detail.admins.map((admin, i) => {
            const pk = admin.pubkeyHex;
            const isActivated = activated.some(a => a.pubkeyHex.toLowerCase() === pk.toLowerCase());
            const isDecrypted = decrypted.some(a => a.pubkeyHex.toLowerCase().replace(/^0x/, '') === pk.toLowerCase().replace(/^0x/, ''));
            return (
              <div key={pk} className={`metric-card admin-card ${isDecrypted ? 'admin-card-decrypted' : ''}`}>
                <span className="admin-card-index">{i + 1}</span>
                <code className="admin-card-address">{hexToSs58(pk)}</code>
                <span className="admin-card-balance">｜ {admin.balanceFen != null ? formatBalance(admin.balanceFen) : '—'}</span>
                <div className="admin-card-actions">
                  {isActivated
                    ? <span className="activated-tag">已激活</span>
                    : <button className="activate-button" onClick={() => startActivation(pk)}>激活</button>
                  }
                  {isDecrypted ? (
                    <>
                      <span className="decrypted-tag" title="管理员私钥已解密到内存,packer 可签批"><span className="green-dot" />已解密</span>
                      <button className="secondary-button" onClick={() => lockAdmin(pk)}>加锁</button>
                    </>
                  ) : (
                    <button className="decrypt-button" disabled={!isActivated} onClick={() => startDecrypt(pk)}>
                      解密
                    </button>
                  )}
                </div>
              </div>
            );
          })}
        </div>
      )}

      {/* 激活弹窗 */}
      {actStep !== 'idle' && (
        <div className="modal-overlay" onClick={() => actStep !== 'verifying' && setActStep('idle')}>
          <div className="modal-content" onClick={(e) => e.stopPropagation()}>
            {actStep === 'qr' && actRequest && (
              <>
                <h3>扫码激活管理员</h3>
                <div className="qr-container"><QRCodeSVG value={actRequest.requestJson} size={280} level="L" /></div>
                <p className="countdown">有效时间:{actCountdown} 秒</p>
                <button className="primary-button" onClick={() => setActStep('scan')}>已签名,扫描回执</button>
                <button className="secondary-button" onClick={() => setActStep('idle')}>取消</button>
              </>
            )}
            {actStep === 'scan' && (
              <>
                <h3>扫描签名回执</h3>
                <QrScanner onScan={handleActivateScan} onError={(e) => { setActError(e); setActStep('error'); }} />
                <button className="secondary-button" onClick={() => setActStep('qr')}>返回二维码</button>
              </>
            )}
            {actStep === 'verifying' && <p>正在验证签名…</p>}
            {actStep === 'done' && <p>激活成功</p>}
            {actStep === 'error' && (<><div className="error">{actError}</div><button className="secondary-button" onClick={() => setActStep('idle')}>关闭</button></>)}
          </div>
        </div>
      )}

      {/* 解密弹窗 */}
      {decStep !== 'idle' && (
        <div className="modal-overlay" onClick={() => decStep !== 'verifying' && setDecStep('idle')}>
          <div className="modal-content" onClick={(e) => e.stopPropagation()}>
            {decStep === 'qr' && decRequest && (
              <>
                <h3>扫码解密管理员密钥</h3>
                <p>使用 wumin 冷钱包扫码,验签通过后私钥进入节点内存(节点重启自动清空)。</p>
                <div className="qr-container"><QRCodeSVG value={decRequest.requestJson} size={280} level="L" /></div>
                <p className="countdown">有效时间:{decCountdown} 秒</p>
                <button className="primary-button" onClick={() => setDecStep('scan')}>已签名,扫描回执</button>
                <button className="secondary-button" onClick={() => setDecStep('idle')}>取消</button>
              </>
            )}
            {decStep === 'scan' && (
              <>
                <h3>扫描签名回执</h3>
                <QrScanner onScan={handleDecryptScan} onError={(e) => { setDecError(e); setDecStep('error'); }} />
                <button className="secondary-button" onClick={() => setDecStep('qr')}>返回二维码</button>
              </>
            )}
            {decStep === 'verifying' && <p>正在验证签名…</p>}
            {decStep === 'done' && <p>解密成功 · packer 可签批</p>}
            {decStep === 'error' && (<><div className="error">{decError}</div><button className="secondary-button" onClick={() => setDecStep('idle')}>关闭</button></>)}
          </div>
        </div>
      )}
    </div>
  );
}
