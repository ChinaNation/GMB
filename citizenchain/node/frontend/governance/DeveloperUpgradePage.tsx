// 开发升级页：选择 WASM 文件 → 选冷钱包 → QR 签名 → 提交 developer_direct_upgrade。
import { useState, useEffect, useRef, useCallback } from 'react';
import { open } from '@tauri-apps/plugin-dialog';
import { QRCodeSVG } from 'qrcode.react';
import { api, sanitizeError } from '../api';
import { QrScanner } from './QrScanner';
import type { VoteSignRequestResult } from './governance-types';

type FlowStep = 'form' | 'qr' | 'scan' | 'submit' | 'done' | 'error';

export function DeveloperUpgradePage() {
  const [wasmPath, setWasmPath] = useState('');
  const [wasmFileName, setWasmFileName] = useState('');
  const [selectedPubkey, setSelectedPubkey] = useState('');
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

  // QR 倒计时
  useEffect(() => {
    if (step !== 'qr') return;
    if (countdown <= 0) { setError('签名请求已过期，请重新操作'); setStep('error'); return; }
    const timer = setTimeout(() => setCountdown((c) => c - 1), 1000);
    return () => clearTimeout(timer);
  }, [step, countdown]);

  // 系统文件选择器
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
        // 显示文件名（取路径最后一段）
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
    <div className="developer-upgrade-page">
      <h2>开发期 Runtime 升级</h2>
      <p className="dev-upgrade-hint">
        NRC 管理员直接 set_code，不走联合投票。仅在开发期（DeveloperUpgradeEnabled = true）可用。
      </p>

      {step === 'form' && (
        <div className="dev-upgrade-form">
          <div className="dev-upgrade-field">
            <label>Runtime WASM 文件</label>
            <div className="dev-upgrade-file-row">
              <button className="dev-upgrade-pick-file" onClick={handlePickFile}>选择文件</button>
              <span className="dev-upgrade-file-name">
                {wasmFileName || '未选择文件'}
              </span>
            </div>
          </div>
          <div className="dev-upgrade-field">
            <label>NRC 管理员公钥（64 位十六进制）</label>
            <input
              type="text"
              value={selectedPubkey}
              onChange={(e) => setSelectedPubkey(e.target.value.trim())}
              placeholder="输入管理员公钥 hex"
              className="dev-upgrade-pubkey-input"
            />
          </div>
          {error && <div className="error">{error}</div>}
          <button
            className="dev-upgrade-submit"
            disabled={!wasmPath.trim() || !selectedPubkey || building}
            onClick={handleBuildRequest}
          >
            {building ? '构建签名请求中…' : '生成签名请求'}
          </button>
        </div>
      )}

      {step === 'qr' && (
        <div className="dev-upgrade-qr">
          <p className="qr-instruction">用 wumin 离线设备扫描此二维码完成签名</p>
          <div className="qr-container"><QRCodeSVG value={requestJson} size={280} level="L" /></div>
          <p className="qr-countdown">剩余 <strong>{countdown}</strong> 秒</p>
          <button className="dev-upgrade-submit" onClick={() => setStep('scan')}>已签名，扫描回执</button>
          <button className="cancel-button" onClick={() => setStep('form')}>取消</button>
        </div>
      )}

      {step === 'scan' && (
        <div className="dev-upgrade-scan">
          <p className="qr-instruction">将签名回执二维码对准摄像头</p>
          <QrScanner onScan={handleScanResult} onError={(e) => { setError(e); setStep('error'); }} />
          <button className="cancel-button" onClick={() => setStep('qr')}>返回</button>
        </div>
      )}

      {step === 'submit' && (
        <div className="dev-upgrade-status">
          <p className="qr-instruction">正在验证签名并提交 runtime 升级…</p>
        </div>
      )}

      {step === 'done' && (
        <div className="dev-upgrade-done">
          <div className="vote-success">
            <p>Runtime 升级已提交</p>
            {txHash && <code className="tx-hash">交易哈希: {txHash}</code>}
          </div>
          <p>新 runtime 将在下一个区块生效。</p>
          <button className="dev-upgrade-submit" onClick={() => { setStep('form'); setTxHash(null); setWasmPath(''); setWasmFileName(''); }}>完成</button>
        </div>
      )}

      {step === 'error' && (
        <div className="dev-upgrade-error">
          <div className="error">{error}</div>
          <button className="dev-upgrade-submit" onClick={() => { setError(null); setStep('form'); }}>重试</button>
        </div>
      )}
    </div>
  );
}
