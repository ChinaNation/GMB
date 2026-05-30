// 系统设置页：展示 INSTALL 安装授权、超级管理员绑定和 ARCHIVE 签发状态。

import { useState, useEffect } from 'react';
import { listTowns, listVillages } from '../address/api';
import { installStatus } from '../initialize/api';
import type { Town, Village } from '../address/types';
import type { InstallStatus } from '../initialize/types';
import { exportStatusFile, getStatusExportState } from './api';
import type { CpmsStatusExportState } from './types';

export default function SystemSettings() {
  const [status, setStatus] = useState<InstallStatus | null>(null);
  const [exportState, setExportState] = useState<CpmsStatusExportState | null>(null);
  const [error, setError] = useState('');
  const [exporting, setExporting] = useState(false);

  const load = () => {
    installStatus()
      .then(res => { if (res.data) setStatus(res.data); })
      .catch(e => setError(e instanceof Error ? e.message : '状态加载失败'));
    getStatusExportState()
      .then(res => { if (res.data?.state) setExportState(res.data.state); })
      .catch(e => setError(e instanceof Error ? e.message : '年度导出状态加载失败'));
  };

  useEffect(() => { load(); }, []);

  const initialized = status?.initialized ?? false;
  const adminBound = (status?.super_admin_bound_count ?? 0) >= 1;
  const signingReady = status?.archive_signing_ready ?? false;
  const pendingExportYear = exportState?.pending_export_year ?? null;
  const canExport = signingReady && (exportState?.can_export ?? false);
  const exportDisabled = !canExport || exporting;
  const exportButtonText = exporting
    ? '导出中...'
    : pendingExportYear
      ? `导出 ${pendingExportYear} 年度报告`
      : '导出年度报告';

  const handleExport = async () => {
    setError('');
    if (exportDisabled) return;
    setExporting(true);
    try {
      const res = await exportStatusFile();
      if (res.data) {
        const text = JSON.stringify(res.data.export_file, null, 2);
        const blob = new Blob([text], { type: 'application/json;charset=utf-8' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = res.data.file_name;
        a.click();
        URL.revokeObjectURL(url);
        load();
        window.dispatchEvent(new Event('cpms-status-export-updated'));
      }
    } catch (e) {
      setError(e instanceof Error ? e.message : '导出失败');
    } finally {
      setExporting(false);
    }
  };

  return (
    <div className="card">
      <div className="card__title" style={{ fontSize: 18, marginBottom: 20 }}>系统设置</div>

      {error && <div style={{ color: 'var(--color-danger)', fontSize: 13, marginBottom: 12 }}>{error}</div>}

      <Section step={1} title="安装授权" done={initialized}>
        {initialized ? (
          <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 8 }}>
            <InfoRow label="机构 SFID" value={status?.sfid_number || '—'} />
            <InfoRow label="省份" value={status?.province_name || status?.province_code || '—'} />
            <InfoRow label="城市" value={status?.city_name || status?.city_code || '—'} />
          </div>
        ) : (
          <div style={{ color: 'var(--color-text-secondary)', fontSize: 13 }}>
            尚未完成初始化，请前往初始化页面扫描 SFID 安装授权二维码。
          </div>
        )}
      </Section>

      <Divider />

      <Section step={2} title="超级管理员" done={adminBound}>
        <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 8 }}>
          <InfoRow label="已绑定数量" value={String(status?.super_admin_bound_count ?? 0)} />
          <InfoRow label="状态" value={adminBound ? '已绑定' : '未绑定'} />
        </div>
      </Section>

      <Divider />

      <Section step={3} title="档案签发" done={signingReady}>
        <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 8 }}>
          <InfoRow label="签发状态" value={signingReady ? '已就绪' : '未就绪'} />
          <InfoRow label="CPMS 公钥" value={status?.cpms_pubkey || '—'} />
        </div>
      </Section>

      <Divider />

      <Section step={4} title="年度报告" done={signingReady}>
        <div style={{ display: 'flex', alignItems: 'center', gap: 12, flexWrap: 'wrap' }}>
          {exportState?.reminder_active && (
            <span style={{
              display: 'inline-flex', alignItems: 'center', height: 24, padding: '0 10px',
              borderRadius: 4, fontSize: 12, fontWeight: 600,
              background: exportState.operator_lock_active ? '#fee2e2' : '#fef3c7',
              color: exportState.operator_lock_active ? 'var(--color-danger)' : 'var(--color-warning)',
            }}>
              {exportState.operator_lock_active ? '逾期未导出' : '待导出'}
            </span>
          )}
          <button className="btn btn--primary" onClick={handleExport} disabled={exportDisabled}>
            {exportButtonText}
          </button>
        </div>
        <div style={{ color: 'var(--color-text-secondary)', fontSize: 13, marginTop: 8 }}>
          {exportStatusText(signingReady, exportState)}
        </div>
        {exportState?.operator_lock_active && (
          <div style={{ color: 'var(--color-danger)', fontSize: 13, marginTop: 6 }}>
            操作管理员登录已锁定，完成年度报告导出后自动恢复。
          </div>
        )}
      </Section>

      <Divider />

      <AddressScope />
    </div>
  );
}

function exportStatusText(signingReady: boolean, exportState: CpmsStatusExportState | null) {
  if (!signingReady) return '档案签发密钥未就绪，暂不能导出年度报告。';
  if (!exportState) return '正在读取年度导出状态。';
  if (exportState.pending_export_year) {
    return `${exportState.pending_export_year} 年度报告尚未导出。`;
  }
  if (exportState.exported) {
    const next = exportState.next_export_available_at
      ? formatUtcDate(exportState.next_export_available_at)
      : '下一年度 1 月 1 日';
    return `当前无待导出年度，${next} 后开放下一次导出。`;
  }
  return '当前无待导出年度。';
}

function formatUtcDate(ts: number) {
  const date = new Date(ts * 1000);
  const year = date.getUTCFullYear();
  const month = String(date.getUTCMonth() + 1).padStart(2, '0');
  const day = String(date.getUTCDate()).padStart(2, '0');
  return `${year}-${month}-${day} UTC`;
}

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
      <div style={{ flex: 1, minWidth: 0 }}>
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
    <div style={{ display: 'flex', gap: 8, fontSize: 13, minWidth: 0 }}>
      <span style={{ color: 'var(--color-text-secondary)', minWidth: 80 }}>{label}</span>
      <span style={{ fontWeight: 500, fontFamily: 'monospace', overflowWrap: 'anywhere' }}>{value}</span>
    </div>
  );
}

function Divider() {
  return <div style={{ height: 1, background: 'var(--color-border)', margin: '16px 0' }} />;
}

// ── 当前市行政区只读预览 ──

function AddressScope() {
  const [towns, setTowns] = useState<Town[]>([]);
  const [selectedTown, setSelectedTown] = useState('');
  const [villages, setVillages] = useState<Village[]>([]);

  const loadTowns = () => {
    listTowns().then(res => { if (res.data) setTowns(res.data); }).catch(() => {});
  };
  const loadVillages = (code: string) => {
    if (!code) { setVillages([]); return; }
    listVillages(code).then(res => { if (res.data) setVillages(res.data); }).catch(() => {});
  };

  useEffect(() => { loadTowns(); }, []);
  useEffect(() => { loadVillages(selectedTown); }, [selectedTown]);

  return (
    <Section step={5} title="行政区数据" done={towns.length > 0}>
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
          </div>
        ))}
      </div>

      {selectedTown && (
        <>
          <div style={{ fontSize: 13, fontWeight: 500, marginBottom: 6 }}>
            {towns.find(t => t.town_code === selectedTown)?.town_name} - 村/路
          </div>
          <div style={{ display: 'flex', gap: 6, flexWrap: 'wrap', marginBottom: 8 }}>
            {villages.map(v => (
              <div key={v.village_id} style={{
                display: 'flex', alignItems: 'center', gap: 4,
                padding: '2px 8px', borderRadius: 4, fontSize: 12, background: '#f3f4f6',
              }}>
                {v.village_name}
              </div>
            ))}
          </div>
        </>
      )}
    </Section>
  );
}
