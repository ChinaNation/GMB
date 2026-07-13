// 开发期直升页：国家储委会任意已激活管理员签名后直接提交 developer_direct_upgrade。
import { useState, useEffect, useRef, useCallback } from 'react';
import { open } from '@tauri-apps/plugin-dialog';
import { sanitizeError } from '../../tauri';
import { hexToSs58 } from '../../shared/ss58';
import { CitizenSignaturePanel } from '../../shared/qr/CitizenSignaturePanel';
import { runtimeUpgradeApi as api } from './api';
import type { PowDifficultyParams } from './api';
import type { AdminWalletMatch, VoteSignRequestResult } from '../types';

type FlowStep = 'form' | 'qr' | 'submit' | 'done' | 'error';

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
  const [powParams, setPowParams] = useState<PowDifficultyParams | null>(null);

  const signRequestRef = useRef(signRequest);
  const selectedPubkeyRef = useRef(selectedPubkey);
  const wasmPathRef = useRef(wasmPath);
  const powParamsRef = useRef(powParams);
  signRequestRef.current = signRequest;
  selectedPubkeyRef.current = selectedPubkey;
  wasmPathRef.current = wasmPath;
  powParamsRef.current = powParams;

  useEffect(() => {
    api.getPowDifficultyParams()
      .then(setPowParams)
      .catch((e) => setError(sanitizeError(e)));
  }, []);

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
    if (!wasmPath.trim() || !selectedPubkey || !powParams) return;
    setBuilding(true);
    setError(null);
    try {
      const result = await api.buildDeveloperUpgradeRequest(
        selectedPubkey,
        wasmPath.trim(),
        powParams,
      );
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
  }, [wasmPath, selectedPubkey, powParams]);

  const handleScanResult = useCallback(async (responseText: string) => {
    const req = signRequestRef.current;
    const pubkey = selectedPubkeyRef.current;
    const path = wasmPathRef.current;
    const params = powParamsRef.current;
    if (!req || !pubkey || !params) { setError('签名请求数据丢失，请重试'); setStep('error'); return; }
    setStep('submit');
    try {
      const result = await api.submitDeveloperUpgrade(
        req.requestId, pubkey, req.expectedPayloadHash,
        path, params, req.signNonce, req.signBlockNumber, responseText,
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

          {powParams && (
            <div className="wallet-form-field">
              <label>PoW 参数（与本次 runtime 升级原子绑定）</label>
              {([
                ['paramsVersion', '参数版本'],
                ['algorithmVersion', '算法版本'],
                ['targetBlockTimeMs', '平均目标时间（毫秒）'],
                ['adjustmentInterval', '调整窗口（块）'],
                ['maxAdjustUpFactor', '最大上调倍率'],
                ['maxAdjustDownDivisor', '最大下调分母'],
              ] as const).map(([field, label]) => (
                <label key={field}>{label}
                  <input
                    type="number"
                    min={1}
                    value={powParams[field]}
                    onChange={(e) => setPowParams({
                      ...powParams,
                      [field]: Number(e.target.value),
                    })}
                    disabled={building}
                  />
                </label>
              ))}
            </div>
          )}

          <div className="wallet-form-field">
            <label>发起管理员</label>
            {adminWallets.length === 0 ? (
              <p className="upgrade-no-wallet">无已激活国家储委会管理员</p>
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
            disabled={!wasmPath.trim() || !selectedPubkey || !powParams || building}
            onClick={handleBuildRequest}
          >
            {building ? '构建中…' : '生成签名请求'}
          </button>
        </div>
      )}

      {step === 'qr' && (
        <div className="vote-signing-body">
          <CitizenSignaturePanel
            qrValue={requestJson}
            countdownSeconds={countdown}
            onScan={handleScanResult}
            onScanError={(e) => { setError(e); setStep('error'); }}
          />
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
            <p>开发升级已提交</p>
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
