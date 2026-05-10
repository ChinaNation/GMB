// 开发期 Runtime 直升页：国储会任意已激活管理员签名后直接提交 developer_direct_upgrade。
import { useState, useEffect, useRef, useCallback } from 'react';
import { open } from '@tauri-apps/plugin-dialog';
import { QRCodeSVG } from 'qrcode.react';
import { sanitizeError } from '../../core/tauri';
import { hexToSs58 } from '../../shared/ss58';
import { QrScanner } from '../../shared/qr/QrScanner';
import { runtimeUpgradeApi as api } from './api';
import type { AdminWalletMatch, VoteSignRequestResult } from '../types';

type FlowStep = 'form' | 'qr' | 'scan' | 'submit' | 'done' | 'error';

type Props = {
  adminWallets: AdminWalletMatch[];
  onBack: () => void;
  onSuccess: () => void;
};

export function DeveloperUpgradePage({ adminWallets, onBack, onSuccess }: Props) {
  const [wasmPath, setWasmPath] = useState('');
  const [wasmFileName, setWasmFileName] = useState('');
  const [selectedPubkey, setSelectedPubkey] = useState(
    adminWallets.length === 1 ? adminWallets[0].pubkeyHex : ''
  );
  const [step, setStep] = useState<FlowStep>('form');
  const [signRequest, setSignRequest] = useState<VoteSignRequestResult | null>(null);
  const [requestJson, setRequestJson] = useState('');
  const [countdown, setCountdown] = useState(90);
  const [error, setError] = useState<string | null>(null);
  const [txHash, setTxHash] = useState<string | null>(null);
  const [building, setBuilding] = useState(false);

  const signRequestRef = useRef(signRequest);
  const selectedPubkeyRef = useRef(selectedPubkey);
  const wasmPathRef = useRef(wasmPath);
  signRequestRef.current = signRequest;
  selectedPubkeyRef.current = selectedPubkey;
  wasmPathRef.current = wasmPath;

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
    if (!wasmPath.trim() || !selectedPubkey) return;
    setBuilding(true);
    setError(null);
    try {
      const result = await api.buildDeveloperUpgradeRequest(selectedPubkey, wasmPath.trim());
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
  }, [wasmPath, selectedPubkey]);

  const handleScanResult = useCallback(async (responseText: string) => {
    const req = signRequestRef.current;
    const pubkey = selectedPubkeyRef.current;
    const path = wasmPathRef.current;
    if (!req || !pubkey) { setError('签名请求数据丢失，请重试'); setStep('error'); return; }
    setStep('submit');
    try {
      const result = await api.submitDeveloperUpgrade(
        req.requestId, pubkey, req.expectedPayloadHash,
        path, req.signNonce, req.signBlockNumber, responseText,
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
      <button className="back-button" onClick={onBack}>&larr; 返回</button>
      <h2>开发升级</h2>
      <p className="upgrade-proposal-hint">
        开发期直接提交 Runtime WASM，不进入联合投票流程。
      </p>

      {step === 'form' && (
        <div className="create-proposal-form">
          {error && <div className="error">{error}</div>}

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
              <p className="upgrade-no-wallet">无已激活国储会管理员</p>
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
            disabled={!wasmPath.trim() || !selectedPubkey || building}
            onClick={handleBuildRequest}
          >
            {building ? '构建中…' : '生成签名请求'}
          </button>
        </div>
      )}

      {step === 'qr' && (
        <div className="vote-signing-body qr-step">
          <p className="qr-instruction">用 wumin 离线设备扫描此二维码完成签名</p>
          <div className="qr-container"><QRCodeSVG value={requestJson} size={280} level="L" /></div>
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
          <p className="qr-instruction">正在提交到链…</p>
        </div>
      )}

      {step === 'done' && (
        <div className="vote-signing-body">
          <div className="vote-success">
            <p>Runtime 开发升级已提交</p>
            {txHash && <code className="tx-hash">交易哈希: {txHash}</code>}
          </div>
          <button
            className="vote-signing-confirm"
            onClick={() => {
              setWasmPath('');
              setWasmFileName('');
              onSuccess();
            }}
          >
            完成
          </button>
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
