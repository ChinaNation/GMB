// 创建费率设置提案页面：选择费率 + QR 签名流程。
import { useState, useEffect, useCallback, useRef } from 'react';
import { QRCodeSVG } from 'qrcode.react';
import { api, sanitizeError } from '../api';
import { QrScanner } from './QrScanner';
import type { AdminWalletMatch, VoteSignRequestResult } from './governance-types';
import { hexToSs58 } from '../format';

type Props = {
  shenfenId: string;
  institutionName: string;
  adminWallets: AdminWalletMatch[];
  onBack: () => void;
  onSuccess: () => void;
};

type Step = 'form' | 'qr' | 'scan' | 'submit' | 'done' | 'error';

export function FeeRateProposalPage({
  shenfenId, institutionName, adminWallets, onBack, onSuccess,
}: Props) {
  const [step, setStep] = useState<Step>('form');

  // 表单
  const [selectedWallet, setSelectedWallet] = useState<AdminWalletMatch | null>(
    adminWallets.length === 1 ? adminWallets[0] : null
  );
  const [currentRateBp, setCurrentRateBp] = useState<number | null>(null);
  const [newRateBp, setNewRateBp] = useState<number>(5);
  const [formError, setFormError] = useState<string | null>(null);
  const [submitting, setSubmitting] = useState(false);
  const [loadingRate, setLoadingRate] = useState(true);

  // QR 签名
  const [signRequest, setSignRequest] = useState<VoteSignRequestResult | null>(null);
  const [requestJson, setRequestJson] = useState('');
  const [countdown, setCountdown] = useState(90);
  const [error, setError] = useState<string | null>(null);
  const [txHash, setTxHash] = useState<string | null>(null);

  const signRequestRef = useRef(signRequest);
  const selectedWalletRef = useRef(selectedWallet);
  const newRateBpRef = useRef(newRateBp);
  signRequestRef.current = signRequest;
  selectedWalletRef.current = selectedWallet;
  newRateBpRef.current = newRateBp;

  // 加载当前费率
  useEffect(() => {
    setLoadingRate(true);
    api.queryInstitutionRateBp(shenfenId)
      .then((rate) => {
        setCurrentRateBp(rate);
        setNewRateBp(rate);
      })
      .catch(() => setCurrentRateBp(null))
      .finally(() => setLoadingRate(false));
  }, [shenfenId]);

  // 倒计时
  useEffect(() => {
    if (step !== 'qr') return;
    if (countdown <= 0) {
      setError('签名请求已过期，请重新操作');
      setStep('error');
      return;
    }
    const timer = setTimeout(() => setCountdown((c) => c - 1), 1000);
    return () => clearTimeout(timer);
  }, [step, countdown]);

  const formatRate = (bp: number) => `${bp} bp (${(bp / 100).toFixed(2)}%)`;

  const handleSubmit = async () => {
    if (!selectedWallet) { setFormError('请选择管理员钱包'); return; }
    if (newRateBp < 1 || newRateBp > 10) { setFormError('费率必须在 1-10 bp 范围内'); return; }
    if (currentRateBp !== null && newRateBp === currentRateBp) {
      setFormError('新费率与当前费率相同'); return;
    }
    setFormError(null);
    setSubmitting(true);

    try {
      const result = await api.buildProposeRateRequest(
        selectedWallet.pubkeyHex, shenfenId, newRateBp,
      );
      setSignRequest(result);
      setRequestJson(result.requestJson);
      setCountdown(90);
      setStep('qr');
    } catch (e) {
      setFormError(sanitizeError(e));
    } finally {
      setSubmitting(false);
    }
  };

  const handleScanResult = useCallback(async (responseText: string) => {
    const req = signRequestRef.current;
    const wallet = selectedWalletRef.current;
    const rateBp = newRateBpRef.current;
    if (!req || !wallet) {
      setError('签名请求数据丢失，请重试');
      setStep('error');
      return;
    }
    setStep('submit');
    try {
      const result = await api.submitProposeRate(
        req.requestId, wallet.pubkeyHex, req.expectedPayloadHash,
        shenfenId, rateBp,
        req.signNonce, req.signBlockNumber, responseText,
      );
      setTxHash(result.txHash);
      setStep('done');
    } catch (e) {
      setError(sanitizeError(e));
      setStep('error');
    }
  }, [shenfenId]);

  return (
    <div className="governance-section">
      <button className="back-button" onClick={onBack}>← 返回</button>
      <h2>费率设置提案</h2>
      <p className="proposal-institution-name">{institutionName}</p>

      {step === 'form' && (
        <div className="create-proposal-form">
          {formError && <div className="error">{formError}</div>}

          <div className="wallet-form-field">
            <label>发起管理员</label>
            <select
              value={selectedWallet?.pubkeyHex || ''}
              onChange={(e) => {
                const w = adminWallets.find((w) => w.pubkeyHex === e.target.value);
                setSelectedWallet(w || null);
              }}
              disabled={adminWallets.length <= 1}
            >
              {adminWallets.length === 0 && <option value="">无已激活管理员</option>}
              {adminWallets.length === 1 ? (
                <option value={adminWallets[0].pubkeyHex}>{hexToSs58(adminWallets[0].pubkeyHex)}</option>
              ) : (
                <>
                  <option value="">请选择…</option>
                  {adminWallets.map((w) => (
                    <option key={w.pubkeyHex} value={w.pubkeyHex}>{hexToSs58(w.pubkeyHex)}</option>
                  ))}
                </>
              )}
            </select>
          </div>

          <div className="wallet-form-field">
            <label>当前费率</label>
            <input
              type="text"
              value={loadingRate ? '查询中…' : (currentRateBp != null ? formatRate(currentRateBp) : '未设置')}
              disabled
            />
          </div>

          <div className="wallet-form-field">
            <label>新费率（1-10 bp，即 0.01%-0.1%）</label>
            <select
              value={newRateBp}
              onChange={(e) => setNewRateBp(parseInt(e.target.value))}
              disabled={submitting || loadingRate}
            >
              {[1, 2, 3, 4, 5, 6, 7, 8, 9, 10].map((bp) => (
                <option key={bp} value={bp}>{formatRate(bp)}</option>
              ))}
            </select>
          </div>

          <button
            className="vote-signing-confirm"
            onClick={handleSubmit}
            disabled={submitting || !selectedWallet || loadingRate}
          >
            {submitting ? '生成中…' : '生成签名请求'}
          </button>
        </div>
      )}

      {step === 'qr' && (
        <div className="vote-signing-body qr-step">
          <p className="qr-instruction">用 wumin 离线设备扫描此二维码完成签名</p>
          <div className="qr-container">
            <QRCodeSVG value={requestJson} size={280} level="L" />
          </div>
          <p className="qr-countdown">剩余 <strong>{countdown}</strong> 秒</p>
          <button className="vote-signing-confirm" onClick={() => setStep('scan')}>已签名，扫描回执</button>
        </div>
      )}

      {step === 'scan' && (
        <div className="vote-signing-body">
          <p className="qr-instruction">将签名回执二维码对准摄像头</p>
          <QrScanner onScan={handleScanResult} onError={(e) => { setError(e); setStep('error'); }} />
          <button className="cancel-button" onClick={() => setStep('qr')}>返回</button>
        </div>
      )}

      {step === 'submit' && (
        <div className="vote-signing-body"><p className="qr-instruction">正在提交提案到链…</p></div>
      )}

      {step === 'done' && (
        <div className="vote-signing-body">
          <div className="vote-success">
            <p>费率设置提案已提交</p>
            {txHash && <code className="tx-hash">交易哈希: {txHash}</code>}
          </div>
          <button className="vote-signing-confirm" onClick={onSuccess}>完成</button>
        </div>
      )}

      {step === 'error' && (
        <div className="vote-signing-body">
          <div className="error">{error}</div>
          <button className="vote-signing-confirm" onClick={() => { setError(null); setStep('form'); }}>重试</button>
        </div>
      )}
    </div>
  );
}
