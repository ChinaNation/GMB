// 开发期 Runtime 直升页：国储会/省储会管理员可直接 set_code（不走联合投票）。
// 仅在 DeveloperUpgradeEnabled = true 的开发期可用。
//
// 2026-04-24 重构：从 governance/DeveloperUpgradePage.tsx 迁入本目录（随"治理"tab 下线，
// 功能归属改为设置页"开发升级"子块）。form 阶段改为横排布局，复用 .bootnode-inline 3 列网格：
// 左侧 = "开发升级" 标题 + 选择文件按钮 + 文件名；中间 = 管理员下拉；右侧 = 生成签名请求按钮。
// 外层 section 承载背景框（与"节点身份密钥"/"确定性投票节点"视觉一致）。
//
// 流程保持不变：form → qr → scan → submit → done/error；仅 form 阶段参与 inline 排版，
// 其他阶段保留原有的 QR/扫描/进度/完成/错误面板。
import { useState, useEffect, useRef, useCallback } from 'react';
import { open } from '@tauri-apps/plugin-dialog';
import { QRCodeSVG } from 'qrcode.react';
import { sanitizeError } from '../../core/tauri';
import { governanceApi as api } from '../../governance/api';
import { hexToSs58 } from '../../shared/ss58';
import { QrScanner } from '../../shared/qr/QrScanner';
import type { ActivatedAdmin, InstitutionListItem, VoteSignRequestResult } from '../../governance/types';

type FlowStep = 'form' | 'qr' | 'scan' | 'submit' | 'done' | 'error';
type JointProposerAdmin = ActivatedAdmin & { institutionName: string };

export function DeveloperUpgradePage() {
  const [admins, setAdmins] = useState<JointProposerAdmin[]>([]);
  const [loadingAdmins, setLoadingAdmins] = useState(true);
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

  // 加载国储会 + 43 个省储会已激活管理员，供开发期直升入口复用联合提案发起权限。
  useEffect(() => {
    let cancelled = false;
    async function loadJointProposerAdmins() {
      try {
        const overview = await api.getGovernanceOverview();
        const institutions: InstitutionListItem[] = [
          ...overview.nationalCouncils,
          ...overview.provincialCouncils,
        ];
        const adminGroups = await Promise.all(
          institutions.map(async (institution) => {
            const list = await api.getActivatedAdmins(institution.shenfenId).catch(() => [] as ActivatedAdmin[]);
            return list.map((admin) => ({
              ...admin,
              institutionName: institution.name,
            }));
          }),
        );
        const deduped = Array.from(
          new Map(
            adminGroups
              .flat()
              .map((admin) => [admin.pubkeyHex, admin] as const),
          ).values(),
        );
        if (cancelled) return;
        setAdmins(deduped);
        setSelectedPubkey((current) => {
          if (deduped.some((admin) => admin.pubkeyHex === current)) return current;
          return deduped.length === 1 ? deduped[0].pubkeyHex : '';
        });
      } catch {
        if (!cancelled) setAdmins([]);
      } finally {
        if (!cancelled) setLoadingAdmins(false);
      }
    }
    loadJointProposerAdmins();
    return () => {
      cancelled = true;
    };
  }, []);

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

  if (loadingAdmins) return <div className="governance-loading">加载中…</div>;

  return (
    <section className="section settings-devup-section">
      {step === 'form' && (
        <>
          <div className="bootnode-inline devup-inline">
            <div className="devup-label-group">
              <h2>开发升级</h2>
              <button onClick={handlePickFile}>选择文件</button>
              <span className="dev-upgrade-file-name">
                {wasmFileName || '未选择文件'}
              </span>
            </div>
            {admins.length === 0 ? (
              <p className="upgrade-no-wallet">无已激活的国储会或省储会管理员</p>
            ) : (
              <select
                value={selectedPubkey}
                onChange={(e) => setSelectedPubkey(e.target.value)}
                disabled={admins.length <= 1}
              >
                {admins.length === 1 ? (
                  <option value={admins[0].pubkeyHex}>
                    {admins[0].institutionName} · {hexToSs58(admins[0].pubkeyHex)}
                  </option>
                ) : (
                  <>
                    <option value="">请选择管理员…</option>
                    {admins.map((a) => (
                      <option key={a.pubkeyHex} value={a.pubkeyHex}>
                        {a.institutionName} · {hexToSs58(a.pubkeyHex)}
                      </option>
                    ))}
                  </>
                )}
              </select>
            )}
            <button
              disabled={!wasmPath.trim() || !selectedPubkey || building}
              onClick={handleBuildRequest}
            >
              {building ? '构建中…' : '生成签名请求'}
            </button>
          </div>
          {error ? <p className="section-inline-error">{error}</p> : null}
        </>
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
        <div className="dev-upgrade-submit-progress">
          <p className="qr-instruction">正在提交到链…</p>
        </div>
      )}

      {step === 'done' && (
        <div className="dev-upgrade-done">
          <div className="vote-success">
            <p>Runtime 升级已提交</p>
            {txHash && <code className="tx-hash">交易哈希: {txHash}</code>}
          </div>
          <button className="dev-upgrade-submit" onClick={() => { setStep('form'); setWasmPath(''); setWasmFileName(''); }}>
            完成
          </button>
        </div>
      )}

      {step === 'error' && (
        <div className="dev-upgrade-error">
          <div className="error">{error}</div>
          <button className="cancel-button" onClick={() => { setStep('form'); setError(null); }}>返回</button>
        </div>
      )}
    </section>
  );
}
