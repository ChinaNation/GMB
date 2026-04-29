// 管理员列表页：两列网格展示所有管理员，每个管理员一个卡片。
// 从机构详情页点击"管理员列表"入口卡片进入。
import { useEffect, useState, useCallback } from 'react';
import { QRCodeSVG } from 'qrcode.react';
import { sanitizeError } from '../core/tauri';
import { formatBalance } from '../shared/format';
import { hexToSs58 } from '../shared/ss58';
import { QrScanner } from '../shared/qr/QrScanner';
import { governanceApi as api } from './api';
import type {
  ActivateRequestResult,
  ActivatedAdmin,
  InstitutionDetail,
} from './types';

type Props = {
  shenfenId: string;
  onBack: () => void;
};

type ActivateStep = 'idle' | 'qr' | 'scan' | 'verifying' | 'done' | 'error';

export function AdminListPage({ shenfenId, onBack }: Props) {
  const [detail, setDetail] = useState<InstitutionDetail | null>(null);
  const [activatedAdmins, setActivatedAdmins] = useState<ActivatedAdmin[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  // 激活流程状态
  const [activateStep, setActivateStep] = useState<ActivateStep>('idle');
  const [activatePubkey, setActivatePubkey] = useState('');
  const [activateRequest, setActivateRequest] = useState<ActivateRequestResult | null>(null);
  const [activateCountdown, setActivateCountdown] = useState(90);
  const [activateError, setActivateError] = useState<string | null>(null);

  useEffect(() => {
    setLoading(true);
    Promise.all([
      api.getInstitutionDetail(shenfenId),
      api.getActivatedAdmins(shenfenId).catch(() => [] as ActivatedAdmin[]),
    ])
      .then(([d, aa]) => {
        setDetail(d);
        setActivatedAdmins(aa);
        setError(null);
      })
      .catch((e) => setError(sanitizeError(e)))
      .finally(() => setLoading(false));
  }, [shenfenId]);

  // 激活倒计时
  useEffect(() => {
    if (activateStep !== 'qr') return;
    if (activateCountdown <= 0) {
      setActivateError('签名请求已过期，请重新操作');
      setActivateStep('error');
      return;
    }
    const timer = setTimeout(() => setActivateCountdown(c => c - 1), 1000);
    return () => clearTimeout(timer);
  }, [activateStep, activateCountdown]);

  const startActivation = useCallback(async (pubkeyHex: string) => {
    setActivatePubkey(pubkeyHex);
    setActivateError(null);
    try {
      const result = await api.buildActivateAdminRequest(pubkeyHex, shenfenId);
      setActivateRequest(result);
      setActivateCountdown(90);
      setActivateStep('qr');
    } catch (e) {
      setActivateError(sanitizeError(e));
      setActivateStep('error');
    }
  }, [shenfenId]);

  const handleActivateScan = useCallback(async (responseJson: string) => {
    if (!activateRequest) return;
    setActivateStep('verifying');
    try {
      const result = await api.verifyActivateAdmin(
        activateRequest.requestId,
        activatePubkey,
        activateRequest.expectedPayloadHash,
        activateRequest.payloadHex,
        responseJson,
      );
      setActivatedAdmins(prev => [...prev.filter(a => a.pubkeyHex !== result.pubkeyHex), result]);
      setActivateStep('done');
      setTimeout(() => setActivateStep('idle'), 2000);
    } catch (e) {
      setActivateError(sanitizeError(e));
      setActivateStep('error');
    }
  }, [activateRequest, activatePubkey]);

  if (loading) {
    return <div className="governance-section"><p>加载中…</p></div>;
  }

  if (error || !detail) {
    return (
      <div className="governance-section">
        <button className="back-button" onClick={onBack}>← 返回</button>
        {error && <div className="error">{error}</div>}
      </div>
    );
  }

  const activatedCount = activatedAdmins.length;

  return (
    <div className="governance-section">
      <button className="back-button" onClick={onBack}>← 返回机构详情</button>

      <div className="admin-list-header">
        <h2>管理员列表</h2>
        <span className="admin-list-summary">
          共 {detail.admins.length} 人{activatedCount > 0 && `，已激活 ${activatedCount}`}
        </span>
      </div>

      {detail.admins.length === 0 ? (
        <p className="no-data">暂无数据（需节点运行后查询链上数据）</p>
      ) : (
        <div className="admin-grid">
          {detail.admins.map((admin, i) => {
            const pubkey = admin.pubkeyHex;
            const isActivated = activatedAdmins.some(
              a => a.pubkeyHex.toLowerCase() === pubkey.toLowerCase()
            );
            const ss58 = hexToSs58(pubkey);
            const balanceDisplay = admin.balanceFen != null
              ? formatBalance(admin.balanceFen)
              : '—';
            return (
              <div key={pubkey} className={`metric-card admin-card ${isActivated ? 'admin-card-activated' : ''}`}>
                <span className="admin-card-index">{i + 1}</span>
                <code className="admin-card-address">{ss58}</code>
                <span className="admin-card-balance">｜ {balanceDisplay}</span>
                <div className="admin-card-actions">
                  {isActivated ? (
                    <span className="activated-tag">已激活</span>
                  ) : (
                    <button
                      className="activate-button"
                      onClick={() => startActivation(pubkey)}
                    >激活</button>
                  )}
                </div>
              </div>
            );
          })}
        </div>
      )}

      {/* 激活签名弹窗 */}
      {activateStep !== 'idle' && (
        <div className="modal-overlay" onClick={() => activateStep !== 'verifying' && setActivateStep('idle')}>
          <div className="modal-content" onClick={e => e.stopPropagation()}>
            {activateStep === 'qr' && activateRequest && (
              <>
                <h3>扫码激活管理员</h3>
                <p>使用 wumin 冷钱包扫描以下二维码完成身份验证</p>
                <div className="qr-container">
                  <QRCodeSVG value={activateRequest.requestJson} size={280} level="L" />
                </div>
                <p className="countdown">有效时间：{activateCountdown} 秒</p>
                <button className="primary-button" onClick={() => setActivateStep('scan')}>
                  已签名，扫描回执
                </button>
                <button className="secondary-button" onClick={() => setActivateStep('idle')}>
                  取消
                </button>
              </>
            )}
            {activateStep === 'scan' && (
              <>
                <h3>扫描签名回执</h3>
                <QrScanner
                  onScan={handleActivateScan}
                  onError={(e) => { setActivateError(e); setActivateStep('error'); }}
                />
                <button className="secondary-button" onClick={() => setActivateStep('qr')}>
                  返回二维码
                </button>
              </>
            )}
            {activateStep === 'verifying' && (
              <div className="loading-section"><p>正在验证签名…</p></div>
            )}
            {activateStep === 'done' && (
              <div className="success-section"><p>管理员激活成功</p></div>
            )}
            {activateStep === 'error' && (
              <>
                <div className="error">{activateError}</div>
                <button className="secondary-button" onClick={() => setActivateStep('idle')}>关闭</button>
              </>
            )}
          </div>
        </div>
      )}

    </div>
  );
}
