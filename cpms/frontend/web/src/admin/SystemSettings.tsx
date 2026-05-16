// 系统设置页：展示 INSTALL 安装授权、超级管理员绑定和 ARCHIVE 签发状态。

import { useState, useEffect } from 'react';
import * as api from '../api';
import type { InstallStatus } from '../types';

export default function SystemSettings() {
  const [status, setStatus] = useState<InstallStatus | null>(null);
  const [error, setError] = useState('');

  const load = () => {
    api.installStatus()
      .then(res => { if (res.data) setStatus(res.data); })
      .catch(e => setError(e instanceof Error ? e.message : '状态加载失败'));
  };

  useEffect(() => { load(); }, []);

  const initialized = status?.initialized ?? false;
  const adminBound = (status?.super_admin_bound_count ?? 0) >= 1;
  const signingReady = status?.archive_signing_ready ?? false;

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

      <AddressManagement />
    </div>
  );
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
