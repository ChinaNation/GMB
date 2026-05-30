// 新建公民档案。省份和城市从 INSTALL 初始化信息自动获取，不可修改。
// 镇和村/路从后端地址 API 加载，联动选择。具体地址文本输入（最长 100 字符）。

import { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import * as api from '../api';

export default function ArchiveCreate() {
  const navigate = useNavigate();
  const today = new Date().toISOString().slice(0, 10);
  const [provinceCode, setProvinceCode] = useState('');
  const [cityCode, setCityCode] = useState('');
  const [provinceName, setProvinceName] = useState('');
  const [cityName, setCityName] = useState('');
  // 地址数据
  const [towns, setTowns] = useState<{ town_code: string; town_name: string }[]>([]);
  const [villages, setVillages] = useState<{ village_id: string; town_code: string; village_name: string }[]>([]);
  const [selectedTown, setSelectedTown] = useState('');
  const [selectedVillage, setSelectedVillage] = useState('');
  const [addressText, setAddressText] = useState('');

  const [form, setForm] = useState({
    last_name: '', first_name: '', birth_date: '',
    gender_code: 'M', height_cm: '', citizen_status: 'NORMAL', voting_eligible: true,
  });
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');

  // 中文注释：省市只来自 CPMS 安装授权，前端不允许手工指定归属。
  useEffect(() => {
    api.installStatus().then(res => {
      if (res.data?.province_code) setProvinceCode(res.data.province_code);
      if (res.data?.city_code) setCityCode(res.data.city_code);
      if (res.data?.province_name) setProvinceName(res.data.province_name);
      if (res.data?.city_name) setCityName(res.data.city_name);
    }).catch(() => {});
  }, []);

  // 加载镇列表
  useEffect(() => {
    api.listTowns().then(res => {
      if (res.data) setTowns(res.data);
    }).catch(() => {});
  }, []);

  // 选镇后联动加载村/路
  useEffect(() => {
    if (!selectedTown) { setVillages([]); setSelectedVillage(''); return; }
    api.listVillages(selectedTown).then(res => {
      if (res.data) setVillages(res.data);
    }).catch(() => {});
    setSelectedVillage('');
  }, [selectedTown]);

  const set = (k: string, v: string) => setForm(f => ({ ...f, [k]: v }));
  const isValidBirthDate = (value: string) => /^\d{4}-\d{2}-\d{2}$/.test(value) && value <= today;
  const isValidHeight = (value: string) => {
    const n = Number(value);
    return Number.isFinite(n) && n >= 30 && n <= 260;
  };

  const handleSubmit = async () => {
    if (!form.last_name.trim()) { setError('请输入姓氏'); return; }
    if (!form.first_name.trim()) { setError('请输入名字'); return; }
    if (!isValidBirthDate(form.birth_date)) { setError('请选择正确的出生日期'); return; }
    if (!form.gender_code) { setError('请选择性别'); return; }
    if (!isValidHeight(form.height_cm)) { setError('请输入正确的身高'); return; }
    if (!provinceCode || !cityCode) { setError('省市信息未加载'); return; }
    if (!selectedTown) { setError('请选择镇'); return; }
    if (!selectedVillage) { setError('请选择村/路'); return; }
    setError('');
    setLoading(true);
    try {
      const body = {
        town_code: selectedTown,
        village_id: selectedVillage,
        address: addressText.trim() || undefined,
        ...form,
        height_cm: parseFloat(form.height_cm),
      };
      const res = await api.createArchive(body);
      if (res.data) navigate(`/admin/archives/${res.data.archive_id}`);
    } catch (e) {
      setError(e instanceof Error ? e.message : '创建失败');
    }
    setLoading(false);
  };

  return (
    <div className="card">
      <div className="card__title">新建公民档案</div>
      {error && <div style={{ color: 'var(--color-danger)', fontSize: 13, marginBottom: 12 }}>{error}</div>}
      <div className="form-row">
        <div className="form-group">
          <label>省份</label>
          <input className="form-input" value={provinceName || provinceCode} disabled style={{ background: '#f3f4f6', cursor: 'not-allowed' }} />
        </div>
        <div className="form-group">
          <label>城市</label>
          <input className="form-input" value={cityName || cityCode} disabled style={{ background: '#f3f4f6', cursor: 'not-allowed' }} />
        </div>
      </div>
      <div className="form-row">
        <div className="form-group">
          <label>镇/街道 *</label>
          <select className="form-input" value={selectedTown} onChange={e => setSelectedTown(e.target.value)}>
            <option value="">请选择镇/街道</option>
            {towns.map(t => <option key={t.town_code} value={t.town_code}>{t.town_name}</option>)}
          </select>
        </div>
        <div className="form-group">
          <label>村/路 *</label>
          <select className="form-input" value={selectedVillage} onChange={e => setSelectedVillage(e.target.value)} disabled={!selectedTown}>
            <option value="">请选择村/路</option>
            {villages.map(v => <option key={v.village_id} value={v.village_id}>{v.village_name}</option>)}
          </select>
        </div>
      </div>
      <div className="form-group">
        <label>具体地址</label>
        <input className="form-input" placeholder="详细门牌号等（最长100字符）" maxLength={100} value={addressText} onChange={e => setAddressText(e.target.value)} />
      </div>
      <div className="form-row">
        <div className="form-group"><label>姓氏 *</label><input className="form-input" value={form.last_name} onChange={e => set('last_name', e.target.value)} /></div>
        <div className="form-group"><label>名字 *</label><input className="form-input" value={form.first_name} onChange={e => set('first_name', e.target.value)} /></div>
      </div>
      <div className="form-row">
        <div className="form-group"><label>出生日期 *</label><input className="form-input" type="date" max={today} value={form.birth_date} onChange={e => set('birth_date', e.target.value)} /></div>
        <div className="form-group">
          <label>性别 *</label>
          <select className="form-input" value={form.gender_code} onChange={e => set('gender_code', e.target.value)}>
            <option value="M">男</option>
            <option value="W">女</option>
          </select>
        </div>
        <div className="form-group"><label>身高 (cm) *</label><input className="form-input" type="number" min={30} max={260} step="0.1" value={form.height_cm} onChange={e => set('height_cm', e.target.value)} /></div>
      </div>
      <div className="form-row">
        <div className="form-group">
          <label>公民状态 *</label>
          <select className="form-input" value={form.citizen_status} onChange={e => set('citizen_status', e.target.value)}>
            <option value="NORMAL">正常</option>
            <option value="REVOKED">注销</option>
          </select>
        </div>
        <div className="form-group">
          <label>选举资格 *</label>
          <select className="form-input" value={String(form.voting_eligible)} onChange={e => setForm(f => ({ ...f, voting_eligible: e.target.value === 'true' }))}>
            <option value="true">有选举资格</option>
            <option value="false">无选举资格</option>
          </select>
        </div>
      </div>
      <div style={{ display: 'flex', gap: 8, marginTop: 20 }}>
        <button className="btn btn--primary" onClick={handleSubmit} disabled={loading}>{loading ? '提交中...' : '创建档案'}</button>
        <button className="btn btn--ghost" onClick={() => navigate('/admin')}>取消</button>
      </div>
    </div>
  );
}
