// 系统设置页：QR1 信息 → QR2 生成 → QR3 扫码，三步骤一体化布局。

import { useState, useEffect, useRef } from 'react';
import { QRCodeSVG } from 'qrcode.react';
import * as api from '../api';
import type { InstallStatus } from '../types';
import { startCameraScanner, scanImageQr } from '../utils/cameraScanner';
import { ScanIcon } from '../components/ScanIcon';

export default function SystemSettings() {
  const [status, setStatus] = useState<InstallStatus | null>(null);
  const [qr2Payload, setQr2Payload] = useState<string | null>(null);
  const [qr2Loading, setQr2Loading] = useState(false);
  const [qr3Done, setQr3Done] = useState(false);
  const [scannerActive, setScannerActive] = useState(false);
  const [scannerReady, setScannerReady] = useState(false);
  const [scanSubmitting, setScanSubmitting] = useState(false);
  const [error, setError] = useState('');
  const [msg, setMsg] = useState('');
  const [qr2ModalOpen, setQr2ModalOpen] = useState(false);
  const videoRef = useRef<HTMLVideoElement | null>(null);
  const scanCleanupRef = useRef<(() => void) | null>(null);
  const fileInputRef = useRef<HTMLInputElement | null>(null);

  const load = () => {
    api.installStatus().then(res => {
      if (res.data) {
        setStatus(res.data);
        setQr3Done(res.data.anon_cert_done);
        if (res.data.qr2_payload) {
          setQr2Payload(res.data.qr2_payload);
        }
      }
    }).catch(() => {});
  };

  useEffect(() => { load(); }, []);

  // ── QR2 生成 ──
  const handleGenerateQr2 = async () => {
    setError('');
    setMsg('');
    setQr2Loading(true);
    try {
      const res = await api.adminGenerateQr2();
      if (res.data) {
        setQr2Payload(res.data.qr2_payload);
      }
    } catch (e) {
      setError(e instanceof Error ? e.message : 'QR2 生成失败');
    }
    setQr2Loading(false);
  };

  // ── QR2 下载：用隐藏容器渲染 SVG → canvas → PNG ──
  const handleDownloadQr2 = () => {
    if (!qr2Payload) return;
    // 找页面上任意一个 QR2 SVG（缩略图或弹窗里的都行）
    const svg = document.querySelector('[data-qr2] svg');
    if (!svg) return;
    const svgData = new XMLSerializer().serializeToString(svg);
    const canvas = document.createElement('canvas');
    canvas.width = 520;
    canvas.height = 520;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;
    ctx.fillStyle = '#fff';
    ctx.fillRect(0, 0, 520, 520);
    const img = new Image();
    img.onload = () => {
      ctx.drawImage(img, 10, 10, 500, 500);
      const a = document.createElement('a');
      a.href = canvas.toDataURL('image/png');
      a.download = 'cpms-qr2.png';
      a.click();
    };
    img.src = 'data:image/svg+xml;base64,' + btoa(unescape(encodeURIComponent(svgData)));
  };

  // ── QR3 扫码 ──
  const stopScanner = () => {
    if (scanCleanupRef.current) {
      scanCleanupRef.current();
      scanCleanupRef.current = null;
    }
    setScannerReady(false);
  };

  useEffect(() => {
    if (!scannerActive || !videoRef.current) {
      stopScanner();
      return;
    }
    const video = videoRef.current;
    const cleanup = startCameraScanner(
      video,
      (raw) => { handleQr3Scanned(raw); },
      () => { setScannerReady(true); },
      (errMsg) => { setError(errMsg); setScannerActive(false); },
    );
    scanCleanupRef.current = cleanup;
    return () => stopScanner();
  }, [scannerActive]);

  const handleQr3Scanned = async (raw: string) => {
    setError('');
    setMsg('');
    setScanSubmitting(true);
    setScannerActive(false);
    stopScanner();
    try {
      await api.adminProcessAnonCert(raw);
      setMsg('QR3 注册完成，系统已具备档案签发能力');
      setQr3Done(true);
      load();
    } catch (e) {
      setError(e instanceof Error ? e.message : 'QR3 处理失败');
    }
    setScanSubmitting(false);
  };

  const onUploadQr3Image = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (fileInputRef.current) fileInputRef.current.value = '';
    if (!file) return;
    try {
      const raw = await scanImageQr(file);
      await handleQr3Scanned(raw);
    } catch {
      setError('未识别到二维码');
    }
  };

  const initialized = status?.initialized ?? false;
  const siteSfid = status?.site_sfid ?? '—';

  // 步骤状态
  const step1Done = initialized;
  const step2Done = !!qr2Payload || status?.qr2_ready === true;
  const step3Done = qr3Done;

  return (
    <div className="card">
      <div className="card__title" style={{ fontSize: 18, marginBottom: 20 }}>系统设置</div>

      {error && <div style={{ color: 'var(--color-danger)', fontSize: 13, marginBottom: 12 }}>{error}</div>}
      {msg && <div style={{ color: 'var(--color-success)', fontSize: 13, marginBottom: 12 }}>{msg}</div>}

      {/* ── 第 1 部分：QR1 信息（左右两栏） ── */}
      <Section
        step={1}
        title="安装授权（QR1）"
        done={step1Done}
      >
        {step1Done ? (() => {
          const parsed = parseSfid(siteSfid);
          return (
            <>
              <div style={{ marginBottom: 8 }}>
                <InfoRow label="站点 SFID" value={siteSfid} />
              </div>
              <div style={{ display: 'flex', gap: 32 }}>
                <div style={{ flex: 1, display: 'flex', flexDirection: 'column', gap: 6 }}>
                  <InfoRow label="机构名称" value={status?.institution_name || parsed.institutionCode} />
                  <InfoRow label="机构类型" value={`${parsed.a3Name}/${parsed.instName}`} />
                  <InfoRow label="管理员数" value={String(status?.super_admin_bound_count ?? 0)} />
                </div>
                <div style={{ flex: 1, display: 'flex', flexDirection: 'column', gap: 6 }}>
                  <InfoRow label="省份" value={status?.province_name || parsed.provinceCode} />
                  <InfoRow label="城市" value={status?.city_name || parsed.cityCode} />
                  <InfoRow label="成立日期" value={parsed.date} />
                </div>
              </div>
            </>
          );
        })() : (
          <div style={{ color: 'var(--color-text-secondary)', fontSize: 13 }}>
            尚未完成初始化，请前往初始化页面扫描 QR1。
          </div>
        )}
      </Section>

      <Divider />

      {/* ── 第 2 部分：QR2 生成 ── */}
      <Section
        step={2}
        title="注册二维码（QR2）"
        done={step2Done && step3Done}
      >
        {step3Done ? (
          <div style={{ color: 'var(--color-success)', fontSize: 13 }}>QR2 流程已完成。</div>
        ) : !qr2Payload ? (
          <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'flex-start', gap: 8 }}>
            <div style={{ color: 'var(--color-text-secondary)', fontSize: 13 }}>
              生成 QR2 后将二维码交给 SFID 管理员扫码注册。
            </div>
            <button className="btn btn--primary btn--sm" onClick={handleGenerateQr2} disabled={qr2Loading || !step1Done}>
              {qr2Loading ? '生成中...' : '生成 QR2'}
            </button>
          </div>
        ) : (
          <div style={{ display: 'flex', alignItems: 'center', gap: 16 }}>
            {/* 缩略图（点击放大） */}
            <div
              onClick={() => setQr2ModalOpen(true)}
              style={{
                cursor: 'pointer',
                padding: 4,
                background: '#fff',
                borderRadius: 6,
                border: '1px solid var(--color-border)',
                lineHeight: 0,
              }}
              title="点击查看完整二维码"
              data-qr2=""
            >
              <QRCodeSVG value={qr2Payload} size={64} fgColor="#134e4a" />
            </div>
            <div style={{ display: 'flex', flexDirection: 'column', gap: 6 }}>
              <div style={{ color: 'var(--color-text-secondary)', fontSize: 13 }}>
                QR2 已生成，请交给 SFID 管理员扫码。
              </div>
              <div style={{ display: 'flex', gap: 8 }}>
                <button className="btn btn--ghost btn--sm" onClick={() => setQr2ModalOpen(true)}>
                  查看二维码
                </button>
                <button className="btn btn--ghost btn--sm" onClick={handleDownloadQr2}>
                  下载
                </button>
                <button className="btn btn--ghost btn--sm" onClick={handleGenerateQr2} disabled={qr2Loading}>
                  {qr2Loading ? '重新生成中...' : '重新生成'}
                </button>
              </div>
            </div>
          </div>
        )}
      </Section>

      <Divider />

      {/* ── 第 3 部分：QR3 扫码 ── */}
      <Section
        step={3}
        title="匿名证书（QR3）"
        done={step3Done}
      >
        {step3Done ? (
          <div style={{ color: 'var(--color-success)', fontSize: 13 }}>
            匿名证书注册完成，系统已具备档案签发能力。
          </div>
        ) : (
          <div style={{ display: 'flex', alignItems: 'flex-start', gap: 16 }}>
            {/* 扫码窗口 */}
            <div style={{
              width: 140, height: 140, flexShrink: 0,
              background: 'linear-gradient(145deg, #0f172a, #1e293b)',
              borderRadius: 10, overflow: 'hidden',
              display: 'flex', alignItems: 'center', justifyContent: 'center',
              position: 'relative', border: '1px solid #334155',
            }}>
              <video ref={videoRef} style={{ width: '100%', height: '100%', objectFit: 'cover' }} muted playsInline />
              {!scannerReady && !scannerActive && (
                <div style={{
                  position: 'absolute', inset: 0,
                  display: 'flex', flexDirection: 'column',
                  alignItems: 'center', justifyContent: 'center', gap: 4,
                  cursor: 'pointer', userSelect: 'none',
                }} onClick={() => setScannerActive(true)}>
                  <ScanIcon size={24} color="rgba(255,255,255,0.25)" />
                  <div style={{ color: 'rgba(255,255,255,0.5)', fontSize: 10 }}>点击扫码</div>
                </div>
              )}
            </div>
            <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
              <div style={{ color: 'var(--color-text-secondary)', fontSize: 13 }}>
                SFID 管理员扫描 QR2 后会返回 QR3 二维码，用摄像头扫描或上传图片完成注册。
              </div>
              <div style={{ display: 'flex', gap: 8 }}>
                <button
                  className="btn btn--primary btn--sm"
                  onClick={() => setScannerActive(v => !v)}
                  disabled={scanSubmitting || !step2Done}
                >
                  {scanSubmitting ? '处理中...' : scannerActive ? '停止扫码' : '扫描 QR3'}
                </button>
                <input type="file" accept="image/*" ref={fileInputRef} style={{ display: 'none' }} onChange={onUploadQr3Image} />
                <button
                  className="btn btn--ghost btn--sm"
                  onClick={() => fileInputRef.current?.click()}
                  disabled={scanSubmitting || !step2Done}
                >
                  上传图片
                </button>
              </div>
            </div>
          </div>
        )}
      </Section>

      <Divider />

      {/* ── 第 4 部分：地址管理 ── */}
      <AddressManagement />

      {/* ── QR2 弹窗 ── */}
      {qr2ModalOpen && qr2Payload && (
        <div
          style={{
            position: 'fixed', inset: 0, zIndex: 1000,
            background: 'rgba(0,0,0,0.5)',
            display: 'flex', alignItems: 'center', justifyContent: 'center',
          }}
          onClick={() => setQr2ModalOpen(false)}
        >
          <div
            style={{
              background: '#fff', borderRadius: 16, padding: 24,
              display: 'flex', flexDirection: 'column', alignItems: 'center', gap: 12,
            }}
            onClick={e => e.stopPropagation()}
          >
            <div style={{ fontSize: 16, fontWeight: 600, color: '#134e4a' }}>注册二维码（QR2）</div>
            <div data-qr2="" style={{ lineHeight: 0 }}>
              <QRCodeSVG value={qr2Payload} size={320} fgColor="#134e4a" />
            </div>
            <div style={{ color: '#6b7280', fontSize: 12 }}>请将此二维码展示给 SFID 管理员扫码</div>
            <div style={{ display: 'flex', gap: 8 }}>
              <button className="btn btn--primary btn--sm" onClick={handleDownloadQr2}>下载</button>
              <button className="btn btn--ghost btn--sm" onClick={() => setQr2ModalOpen(false)}>关闭</button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

// ── 子组件 ──

function Section({ step, title, done, children }: {
  step: number;
  title: string;
  done: boolean;
  children: React.ReactNode;
}) {
  return (
    <div style={{ display: 'flex', gap: 12, alignItems: 'flex-start' }}>
      <div style={{
        width: 28, height: 28, borderRadius: '50%', flexShrink: 0,
        display: 'flex', alignItems: 'center', justifyContent: 'center',
        fontSize: 13, fontWeight: 600,
        background: done ? '#dcfce7' : 'var(--color-primary)',
        color: done ? 'var(--color-success)' : '#fff',
        marginTop: 2,
      }}>
        {done ? '✓' : step}
      </div>
      <div style={{ flex: 1 }}>
        <div style={{ fontSize: 14, fontWeight: 600, marginBottom: 8, color: 'var(--color-text)' }}>
          {title}
        </div>
        {children}
      </div>
    </div>
  );
}

function InfoRow({ label, value }: { label: string; value: string }) {
  return (
    <div style={{ display: 'flex', gap: 8, fontSize: 13 }}>
      <span style={{ color: 'var(--color-text-secondary)', minWidth: 80 }}>{label}</span>
      <span style={{ fontWeight: 500, fontFamily: 'monospace' }}>{value}</span>
    </div>
  );
}

function Divider() {
  return <div style={{ height: 1, background: 'var(--color-border)', margin: '16px 0' }} />;
}

// A3 类型名称（6 条固定枚举）
const A3_NAMES: Record<string, string> = {
  GMR: '公民人', ZRR: '自然人', ZNR: '智能人',
  GFR: '公法人', SFR: '私法人', FFR: '非法人',
};
// 机构代码名称（9 条固定枚举）
const INST_NAMES: Record<string, string> = {
  ZG: '中国', ZF: '政府', LF: '立法院', SF: '司法院',
  JC: '监察院', JY: '教育委员会', CB: '储备委员会', CH: '储备银行', TG: '他国',
};

// ── 地址管理组件（超管维护镇/村路） ──

function AddressManagement() {
  const [towns, setTowns] = useState<{ town_code: string; town_name: string }[]>([]);
  const [selectedTown, setSelectedTown] = useState('');
  const [villages, setVillages] = useState<{ village_id: string; town_code: string; village_name: string }[]>([]);
  const [newTownCode, setNewTownCode] = useState('');
  const [newTownName, setNewTownName] = useState('');
  const [newVillageName, setNewVillageName] = useState('');
  const [addrError, setAddrError] = useState('');

  const loadTowns = () => {
    api.listTowns().then(res => { if (res.data) setTowns(res.data); }).catch(() => {});
  };
  const loadVillages = (code: string) => {
    if (!code) { setVillages([]); return; }
    api.listVillages(code).then(res => { if (res.data) setVillages(res.data); }).catch(() => {});
  };

  useEffect(() => { loadTowns(); }, []);
  useEffect(() => { loadVillages(selectedTown); }, [selectedTown]);

  const handleAddTown = async () => {
    if (!newTownCode.trim() || !newTownName.trim()) { setAddrError('请输入镇代码和名称'); return; }
    setAddrError('');
    try {
      await api.createTown(newTownCode.trim(), newTownName.trim());
      setNewTownCode(''); setNewTownName('');
      loadTowns();
    } catch (e) { setAddrError(e instanceof Error ? e.message : '新增失败'); }
  };

  const handleDeleteTown = async (code: string) => {
    if (!confirm('删除镇会同时删除下属所有村/路，确认？')) return;
    try { await api.deleteTown(code); loadTowns(); setSelectedTown(''); } catch { /* ignore */ }
  };

  const handleAddVillage = async () => {
    if (!selectedTown || !newVillageName.trim()) { setAddrError('请选择镇并输入村/路名称'); return; }
    setAddrError('');
    try {
      await api.createVillage(selectedTown, newVillageName.trim());
      setNewVillageName('');
      loadVillages(selectedTown);
    } catch (e) { setAddrError(e instanceof Error ? e.message : '新增失败'); }
  };

  const handleDeleteVillage = async (id: string) => {
    try { await api.deleteVillage(id); loadVillages(selectedTown); } catch { /* ignore */ }
  };

  return (
    <Section step={4} title="地址管理" done={towns.length > 0}>
      {addrError && <div style={{ color: 'var(--color-danger)', fontSize: 12, marginBottom: 8 }}>{addrError}</div>}

      {/* 镇管理 */}
      <div style={{ fontSize: 13, fontWeight: 500, marginBottom: 6 }}>镇/街道</div>
      <div style={{ display: 'flex', gap: 6, flexWrap: 'wrap', marginBottom: 8 }}>
        {towns.map(t => (
          <div key={t.town_code} style={{
            display: 'flex', alignItems: 'center', gap: 4,
            padding: '2px 8px', borderRadius: 4, fontSize: 12,
            background: selectedTown === t.town_code ? 'var(--color-primary)' : '#f3f4f6',
            color: selectedTown === t.town_code ? '#fff' : 'var(--color-text)',
            cursor: 'pointer',
          }} onClick={() => setSelectedTown(t.town_code)}>
            {t.town_name}
            <span onClick={e => { e.stopPropagation(); handleDeleteTown(t.town_code); }}
              style={{ cursor: 'pointer', opacity: 0.5, marginLeft: 2 }}>×</span>
          </div>
        ))}
      </div>
      <div style={{ display: 'flex', gap: 6, marginBottom: 16 }}>
        <input className="form-input" style={{ width: 80 }} placeholder="代码" value={newTownCode} onChange={e => setNewTownCode(e.target.value)} />
        <input className="form-input" style={{ width: 140 }} placeholder="镇/街道名称" value={newTownName} onChange={e => setNewTownName(e.target.value)} />
        <button className="btn btn--primary btn--sm" onClick={handleAddTown}>新增镇</button>
      </div>

      {/* 村/路管理 */}
      {selectedTown && (
        <>
          <div style={{ fontSize: 13, fontWeight: 500, marginBottom: 6 }}>
            {towns.find(t => t.town_code === selectedTown)?.town_name} — 村/路
          </div>
          <div style={{ display: 'flex', gap: 6, flexWrap: 'wrap', marginBottom: 8 }}>
            {villages.map(v => (
              <div key={v.village_id} style={{
                display: 'flex', alignItems: 'center', gap: 4,
                padding: '2px 8px', borderRadius: 4, fontSize: 12, background: '#f3f4f6',
              }}>
                {v.village_name}
                <span onClick={() => handleDeleteVillage(v.village_id)}
                  style={{ cursor: 'pointer', opacity: 0.5, marginLeft: 2 }}>×</span>
              </div>
            ))}
          </div>
          <div style={{ display: 'flex', gap: 6 }}>
            <input className="form-input" style={{ width: 160 }} placeholder="村/路名称" value={newVillageName} onChange={e => setNewVillageName(e.target.value)} />
            <button className="btn btn--primary btn--sm" onClick={handleAddVillage}>新增村/路</button>
          </div>
        </>
      )}
    </Section>
  );
}

/// 从 SFID 号解析各字段。
/// 格式：A3-R5(省2+市3)-T2(2)P1(1)C1(1)-N9(9)-D(8)
function parseSfid(sfid: string) {
  const parts = sfid.split('-');
  if (parts.length !== 5) {
    return { a3: '—', a3Name: '—', provinceCode: '—', cityCode: '—', institutionCode: '—', instName: '—', date: '—' };
  }
  const [a3, r5, t2p1c1, , d] = parts;
  const provinceCode = r5.slice(0, 2);
  const cityCode = r5.slice(2);
  const institutionCode = t2p1c1.slice(0, 2);
  const date = d.length === 8
    ? `${d.slice(0, 4)}-${d.slice(4, 6)}-${d.slice(6, 8)}`
    : d;
  return {
    a3, a3Name: A3_NAMES[a3] ?? a3,
    provinceCode, cityCode,
    institutionCode, instName: INST_NAMES[institutionCode] ?? institutionCode,
    date,
  };
}
