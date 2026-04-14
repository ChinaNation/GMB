// 安全基金转账提案页面：表单 + QR 签名流程。
import { useState, useRef, useEffect, useCallback } from 'react';
import { QRCodeSVG } from 'qrcode.react';
import { api, sanitizeError } from '../api';
import { hexToSs58 } from '../format';
import { QrScanner } from './QrScanner';
import { AddressScanModal } from './AddressScanModal';
import type { AdminWalletMatch, VoteSignRequestResult } from './governance-types';

type Props = {
  adminWallets: AdminWalletMatch[];
  onBack: () => void;
  onSuccess: () => void;
};

type Step = 'form' | 'qr' | 'scan' | 'submit' | 'done' | 'error';

export function SafetyFundProposalPage({ adminWallets, onBack, onSuccess }: Props) {
  const [step, setStep] = useState<Step>('form');

  const [selectedWallet, setSelectedWallet] = useState<AdminWalletMatch | null>(
    adminWallets.length === 1 ? adminWallets[0] : null
  );
  const [beneficiary, setBeneficiary] = useState('');
  const [amountYuan, setAmountYuan] = useState('');
  const [remark, setRemark] = useState('');
  const [formError, setFormError] = useState<string | null>(null);
  const [submitting, setSubmitting] = useState(false);
  const [showAddressScan, setShowAddressScan] = useState(false);

  const [signRequest, setSignRequest] = useState<VoteSignRequestResult | null>(null);
  const [requestJson, setRequestJson] = useState('');
  const [countdown, setCountdown] = useState(90);
  const [error, setError] = useState<string | null>(null);
  const [txHash, setTxHash] = useState<string | null>(null);

  const formValuesRef = useRef({ beneficiary: '', amountYuan: 0, remark: '' });
  const signRequestRef = useRef(signRequest);
  const selectedWalletRef = useRef(selectedWallet);
  signRequestRef.current = signRequest;
  selectedWalletRef.current = selectedWallet;

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

  const validateForm = (): string | null => {
    if (!selectedWallet) return '请选择管理员钱包';
    if (!beneficiary.trim()) return '请输入收款地址';
    const amount = parseFloat(amountYuan.replace(/,/g, ''));
    if (isNaN(amount) || amount < 1.11) return '转账金额不能低于 1.11 元';
    const remarkBytes = new TextEncoder().encode(remark);
    if (remarkBytes.length > 256) return `备注超过 256 字节（当前 ${remarkBytes.length}）`;
    return null;
  };

  const handleSubmit = async () => {
    const err = validateForm();
    if (err) { setFormError(err); return; }
    setFormError(null);
    setSubmitting(true);

    try {
      const amount = parseFloat(amountYuan.replace(/,/g, ''));
      formValuesRef.current = { beneficiary: beneficiary.trim(), amountYuan: amount, remark };

      const result = await api.buildProposeSafetyFundRequest(
        selectedWallet!.pubkeyHex, beneficiary.trim(), amount, remark,
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
    if (!req || !wallet) {
      setError('签名请求数据丢失，请重试');
      setStep('error');
      return;
    }
    setStep('submit');
    try {
      const { beneficiary: ben, amountYuan: amt, remark: rmk } = formValuesRef.current;
      const result = await api.submitProposeSafetyFund(
        req.requestId, wallet.pubkeyHex, req.expectedPayloadHash,
        ben, amt, rmk,
        req.signNonce, req.signBlockNumber, responseText,
      );
      setTxHash(result.txHash);
      setStep('done');
    } catch (e) {
      setError(sanitizeError(e));
      setStep('error');
    }
  }, []);

  return (
    <div className="governance-section">
      <button className="back-button" onClick={onBack}>← 返回</button>
      <h2>安全基金转账提案</h2>
      <p className="proposal-institution-name">国家储备委员会</p>

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
            <label>收款地址（SS58）</label>
            <div className="address-input-row">
              <input
                type="text" value={beneficiary}
                onChange={(e) => setBeneficiary(e.target.value)}
                placeholder="输入 SS58 格式收款地址"
                disabled={submitting}
              />
              <button type="button" className="scan-icon-btn" onClick={() => setShowAddressScan(true)} disabled={submitting} title="扫码填入">
                <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                  <path d="M3 7V5a2 2 0 0 1 2-2h2"/><path d="M17 3h2a2 2 0 0 1 2 2v2"/><path d="M21 17v2a2 2 0 0 1-2 2h-2"/><path d="M7 21H5a2 2 0 0 1-2-2v-2"/>
                  <rect x="7" y="7" width="10" height="10" rx="1"/>
                </svg>
              </button>
            </div>
          </div>

          <div className="wallet-form-field">
            <label>转账金额（元，最少 1.11）</label>
            <input
              type="text" inputMode="decimal" value={amountYuan}
              onChange={(e) => {
                const v = e.target.value.replace(/[^0-9.,]/g, '');
                const clean = v.replace(/,/g, '');
                const dot = clean.indexOf('.');
                const int = dot >= 0 ? clean.slice(0, dot) : clean;
                const dec = dot >= 0 ? clean.slice(dot) : '';
                setAmountYuan(int.replace(/\B(?=(\d{3})+(?!\d))/g, ',') + dec);
              }}
              placeholder="0.00"
              disabled={submitting}
            />
          </div>

          <div className="wallet-form-field">
            <label>备注（可选，最长 256 字节）</label>
            <input
              type="text" value={remark}
              onChange={(e) => setRemark(e.target.value)}
              placeholder="转账备注" disabled={submitting}
            />
          </div>

          <button
            className="vote-signing-confirm"
            onClick={handleSubmit}
            disabled={submitting || !selectedWallet || !beneficiary.trim() || !amountYuan}
          >
            {submitting ? '生成中…' : '生成签名请求'}
          </button>

          {showAddressScan && (
            <AddressScanModal
              onResult={({ address, amount, memo }) => {
                setBeneficiary(address);
                if (amount !== undefined) setAmountYuan(String(amount));
                if (memo !== undefined) setRemark(memo);
                setShowAddressScan(false);
              }}
              onClose={() => setShowAddressScan(false)}
            />
          )}
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
            <p>安全基金转账提案已提交</p>
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
