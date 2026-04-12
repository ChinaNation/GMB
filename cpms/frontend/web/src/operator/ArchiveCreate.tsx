// 新建公民档案。省份和城市从 QR1 初始化信息自动获取，不可修改。
// 镇和村/路从后端地址 API 加载，联动选择。具体地址文本输入（最长 100 字符）。

import { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import * as api from '../api';

export default function ArchiveCreate() {
  const navigate = useNavigate();
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
    full_name: '', birth_date: '',
    gender_code: 'M', height_cm: '', citizen_status: 'NORMAL', voting_eligible: true,
  });
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');

  // 从 install_status 获取省市信息
  useEffect(() => {
    api.installStatus().then(res => {
      if (res.data?.site_sfid) {
        const parts = res.data.site_sfid.split('-');
        if (parts.length === 5) {
          setProvinceCode(parts[1].slice(0, 2));
          setCityCode(parts[1].slice(2));
        }
      }
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

  const handleSubmit = async () => {
    if (!form.full_name.trim()) { setError('请输入姓名'); return; }
    if (!provinceCode || !cityCode) { setError('省市信息未加载'); return; }
    if (!selectedTown) { setError('请选择镇'); return; }
    if (!selectedVillage) { setError('请选择村/路'); return; }
    setError('');
    setLoading(true);
    try {
      const body = {
        province_code: provinceCode,
        city_code: cityCode,
        town_code: selectedTown,
        village_id: selectedVillage,
        address: addressText.trim() || undefined,
        ...form,
        height_cm: form.height_cm ? parseFloat(form.height_cm) : undefined,
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
        <div className="form-group"><label>姓名 *</label><input className="form-input" value={form.full_name} onChange={e => set('full_name', e.target.value)} /></div>
        <div className="form-group"><label>出生日期</label><input className="form-input" type="date" value={form.birth_date} onChange={e => set('birth_date', e.target.value)} /></div>
      </div>
      <div className="form-row">
        <div className="form-group">
          <label>性别</label>
          <select className="form-input" value={form.gender_code} onChange={e => set('gender_code', e.target.value)}>
            <option value="M">男</option>
            <option value="W">女</option>
          </select>
        </div>
        <div className="form-group"><label>身高 (cm)</label><input className="form-input" type="number" value={form.height_cm} onChange={e => set('height_cm', e.target.value)} /></div>
      </div>
      <div className="form-row">
        <div className="form-group">
          <label>公民状态 *</label>
          <select className="form-input" value={form.citizen_status} onChange={e => set('citizen_status', e.target.value)}>
            <option value="NORMAL">正常</option>
            <option value="ABNORMAL">异常</option>
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
