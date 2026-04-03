// Runtime 升级提案页：选择 WASM 文件 → 填写理由 → 选冷钱包 → QR 签名 → 提交 propose_runtime_upgrade。
import { useState, useEffect, useRef, useCallback } from 'react';
import { open } from '@tauri-apps/plugin-dialog';
import { QRCodeSVG } from 'qrcode.react';
import { api, sanitizeError } from '../api';
import { hexToSs58 } from '../format';
import { QrScanner } from './QrScanner';
import type { AdminWalletMatch, ProposeUpgradeRequestResult } from './governance-types';

type FlowStep = 'form' | 'qr' | 'scan' | 'submit' | 'done' | 'error';

type Props = {
  adminWallets: AdminWalletMatch[];
  onBack: () => void;
  onSuccess: () => void;
};

export function RuntimeUpgradeProposalPage({ adminWallets, onBack, onSuccess }: Props) {
  const [wasmPath, setWasmPath] = useState('');
  const [wasmFileName, setWasmFileName] = useState('');
  const [reason, setReason] = useState('');
  const [selectedPubkey, setSelectedPubkey] = useState(
    adminWallets.length === 1 ? adminWallets[0].pubkeyHex : ''
  );
  const [step, setStep] = useState<FlowStep>('form');
  const [signRequest, setSignRequest] = useState<ProposeUpgradeRequestResult | null>(null);
  const [requestJson, setRequestJson] = useState('');
  const [countdown, setCountdown] = useState(90);
  const [error, setError] = useState<string | null>(null);
  const [txHash, setTxHash] = useState<string | null>(null);
  const [building, setBuilding] = useState(false);

  const signRequestRef = useRef(signRequest);
  const selectedPubkeyRef = useRef(selectedPubkey);
  const wasmPathRef = useRef(wasmPath);
  const reasonRef = useRef(reason);
  signRequestRef.current = signRequest;
  selectedPubkeyRef.current = selectedPubkey;
  wasmPathRef.current = wasmPath;
  reasonRef.current = reason;

  // QR 倒计时
  useEffect(() => {
    if (step !== 'qr') return;
    if (countdown <= 0) { setError('签名请求已过期，请重新操作'); setStep('error'); return; }
    const timer = setTimeout(() => setCountdown((c) => c - 1), 1000);
    return () => clearTimeout(timer);
  }, [step, countdown]);

  const handlePickFile = useCallback(async () => {
    try {
      const selected = await open({
        title: '选择 runtime WASM 文件',
        filters: [{ name: 'WASM', extensions: ['wasm'] }],
        multiple: false,
        directory: false,
      });
      if (selected) {
        setWasmPath(selected);
        const parts = selected.replace(/\\/g, '/').split('/');
        setWasmFileName(parts[parts.length - 1] || selected);
      }
    } catch (e) {
      setError(sanitizeError(e));
    }
  }, []);

  const handleBuildRequest = useCallback(async () => {
    if (!wasmPath.trim() || !selectedPubkey || !reason.trim()) return;
    setBuilding(true);
    setError(null);
    try {
      const result = await api.buildProposeUpgradeRequest(selectedPubkey, wasmPath.trim(), reason.trim());
      setSignRequest(result);
      setRequestJson(result.requestJson);
      setCountdown(90);
      setStep('qr');
    } catch (e) {
      setError(sanitizeError(e));
      setStep('error');
    } finally {
      setBuilding(false);
    }
  }, [wasmPath, selectedPubkey, reason]);

  const handleScanResult = useCallback(async (responseText: string) => {
    const req = signRequestRef.current;
    const pubkey = selectedPubkeyRef.current;
    const path = wasmPathRef.current;
    const reasonVal = reasonRef.current;
    if (!req || !pubkey) { setError('签名请求数据丢失，请重试'); setStep('error'); return; }
    setStep('submit');
    try {
      const result = await api.submitProposeUpgrade(
        req.requestId, pubkey, req.expectedPayloadHash,
        path, reasonVal, req.eligibleTotal,
        req.snapshotNonce, req.snapshotSignature,
        req.signNonce, req.signBlockNumber, responseText,
      );
      setTxHash(result.txHash);
      setStep('done');
    } catch (e) {
      setError(sanitizeError(e));
      setStep('error');
    }
  }, []);

  const canSubmit = wasmPath.trim() && selectedPubkey && reason.trim();

  return (
    <div className="governance-section">
      <button className="back-button" onClick={onBack}>&larr; 返回</button>
      <h2>Runtime 升级提案</h2>
      <p className="upgrade-proposal-hint">
        提交运行时升级提案，需经联合投票 + 公民投票通过后执行。
      </p>

      {step === 'form' && (
        <div className="create-proposal-form">
          {error && <div className="error">{error}</div>}

          <div className="wallet-form-field">
            <label>升级理由</label>
            <textarea
              value={reason}
              onChange={(e) => setReason(e.target.value)}
              placeholder="请输入本次升级的理由说明"
              rows={6}
              maxLength={1024}
              disabled={building}
              style={{ minHeight: '120px', resize: 'vertical' }}
            />
          </div>

          <div className="wallet-form-field">
            <label>Runtime WASM 文件</label>
            <div className="upgrade-file-picker">
              <button className="upgrade-file-button" onClick={handlePickFile} disabled={building}>
                选择文件
              </button>
              <span className="upgrade-file-name">
                {wasmFileName || '未选择文件'}
              </span>
            </div>
          </div>

          <div className="wallet-form-field">
            <label>发起管理员</label>
            {adminWallets.length === 0 ? (
              <p className="upgrade-no-wallet">无已激活管理员</p>
            ) : (
              <select
                value={selectedPubkey}
                onChange={(e) => setSelectedPubkey(e.target.value)}
                disabled={adminWallets.length <= 1}
              >
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
            )}
          </div>

          <button
            className="vote-signing-confirm"
            disabled={!canSubmit || building}
            onClick={handleBuildRequest}
          >
            {building ? '获取人口快照并构建签名…' : '生成签名请求'}
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
          <button className="cancel-button" onClick={() => setStep('form')} style={{ marginTop: 8 }}>取消</button>
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
        <div className="vote-signing-body">
          <p className="qr-instruction">正在验证签名并提交升级提案…</p>
        </div>
      )}

      {step === 'done' && (
        <div className="vote-signing-body">
          <div className="vote-success">
            <p>Runtime 升级提案已提交</p>
            {txHash && <code className="tx-hash">交易哈希: {txHash}</code>}
          </div>
          <p className="upgrade-done-note">提案已进入联合投票阶段，需各机构管理员投票通过后进入公民投票。</p>
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
