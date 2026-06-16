// 新建公民档案。居住省市从 INSTALL 初始化信息自动获取，不可修改。
// 居住镇村由安装城市接口加载；出生地省市镇由随包 SFID 行政区真源只读接口加载。

import { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { listBirthCities, listBirthProvinces, listBirthTowns, listTowns, listVillages } from '../address/api';
import { installStatus } from '../initialize/api';
import DateInput, { isAtLeastAgeYmd, isPastYmd } from '../components/DateInput';
import { createArchive } from './api';
import type { City, Province, Town, Village } from '../address/types';

export default function ArchiveCreate() {
  const navigate = useNavigate();
  const [provinceCode, setProvinceCode] = useState('');
  const [cityCode, setCityCode] = useState('');
  const [provinceName, setProvinceName] = useState('');
  const [cityName, setCityName] = useState('');
  // 地址数据
  const [towns, setTowns] = useState<Town[]>([]);
  const [villages, setVillages] = useState<Village[]>([]);
  const [selectedTown, setSelectedTown] = useState('');
  const [selectedVillage, setSelectedVillage] = useState('');
  const [addressText, setAddressText] = useState('');
  const [birthProvinces, setBirthProvinces] = useState<Province[]>([]);
  const [birthCities, setBirthCities] = useState<City[]>([]);
  const [birthTowns, setBirthTowns] = useState<Town[]>([]);
  const [birthProvince, setBirthProvince] = useState('');
  const [birthCity, setBirthCity] = useState('');
  const [birthTown, setBirthTown] = useState('');

  const [form, setForm] = useState({
    last_name: '', first_name: '', birth_date: '',
    gender_code: 'M', height_cm: '', citizen_status: 'NORMAL', voting_eligible: true,
  });
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');

  // 中文注释：省市只来自 CPMS 安装授权，前端不允许手工指定归属。
  useEffect(() => {
    installStatus().then(res => {
      if (res.data?.province_code) setProvinceCode(res.data.province_code);
      if (res.data?.city_code) setCityCode(res.data.city_code);
      if (res.data?.province_name) setProvinceName(res.data.province_name);
      if (res.data?.city_name) setCityName(res.data.city_name);
    }).catch(() => {});
  }, []);

  // 加载镇列表
  useEffect(() => {
    listTowns().then(res => {
      if (res.data) setTowns(res.data);
    }).catch(() => {});
    listBirthProvinces().then(res => {
      if (res.data) setBirthProvinces(res.data);
    }).catch(() => {});
  }, []);

  // 选镇后联动加载村/路
  useEffect(() => {
    if (!selectedTown) { setVillages([]); setSelectedVillage(''); return; }
    listVillages(selectedTown).then(res => {
      if (res.data) setVillages(res.data);
    }).catch(() => {});
    setSelectedVillage('');
  }, [selectedTown]);

  useEffect(() => {
    if (!birthProvince) { setBirthCities([]); setBirthCity(''); return; }
    listBirthCities(birthProvince).then(res => {
      if (res.data) setBirthCities(res.data);
    }).catch(() => {});
    setBirthCity('');
  }, [birthProvince]);

  useEffect(() => {
    if (!birthProvince || !birthCity) { setBirthTowns([]); setBirthTown(''); return; }
    listBirthTowns(birthProvince, birthCity).then(res => {
      if (res.data) setBirthTowns(res.data);
    }).catch(() => {});
    setBirthTown('');
  }, [birthProvince, birthCity]);

  const set = (k: string, v: string) => setForm(f => ({ ...f, [k]: v }));
  const setBirthDate = (value: string) => {
    setForm(f => ({
      ...f,
      birth_date: value,
      voting_eligible: isAtLeastAgeYmd(value, 16) ? f.voting_eligible : false,
    }));
  };
  const setCitizenStatus = (value: string) => {
    setForm(f => ({
      ...f,
      citizen_status: value,
      voting_eligible: value === 'REVOKED' || !isAtLeastAgeYmd(f.birth_date, 16) ? false : f.voting_eligible,
    }));
  };
  const canSetVotingEligible = form.citizen_status === 'NORMAL' && isAtLeastAgeYmd(form.birth_date, 16);
  const isValidHeight = (value: string) => {
    const n = Number(value);
    return Number.isFinite(n) && n >= 30 && n <= 260;
  };

  const handleSubmit = async () => {
    if (!form.last_name.trim()) { setError('请输入姓氏'); return; }
    if (!form.first_name.trim()) { setError('请输入名字'); return; }
    if (!isPastYmd(form.birth_date)) { setError('请选择正确的出生日期'); return; }
    if (form.voting_eligible && !isAtLeastAgeYmd(form.birth_date, 16)) { setError('未满16周岁的公民不能设置为有选举资格'); return; }
    if (!form.gender_code) { setError('请选择性别'); return; }
    if (!isValidHeight(form.height_cm)) { setError('请输入正确的身高'); return; }
    if (!provinceCode || !cityCode) { setError('居住省市信息未加载'); return; }
    if (!selectedTown) { setError('请选择居住镇'); return; }
    if (!selectedVillage) { setError('请选择居住村/路'); return; }
    if (!addressText.trim()) { setError('请输入居住地址'); return; }
    if (!birthProvince) { setError('请选择出生省份'); return; }
    if (!birthCity) { setError('请选择出生城市'); return; }
    if (!birthTown) { setError('请选择出生镇'); return; }
    setError('');
    setLoading(true);
    try {
      const body = {
        town_code: selectedTown,
        village_id: selectedVillage,
        address: addressText.trim() || undefined,
        birth_province_code: birthProvince,
        birth_city_code: birthCity,
        birth_town_code: birthTown,
        election_scope_level: 'PROVINCE' as const,
        ...form,
        height_cm: parseFloat(form.height_cm),
      };
      const res = await createArchive(body);
      if (res.data) navigate(`/admin/archives/${res.data.archive_id}`);
    } catch (e) {
      setError(e instanceof Error ? e.message : '创建失败');
    }
    setLoading(false);
  };

  return (
    <div className="card">
      <div className="card__title flex-between">
        <span>新建公民档案</span>
        <div style={{ display: 'flex', gap: 8 }}>
          <button className="btn btn--primary" onClick={handleSubmit} disabled={loading}>{loading ? '提交中...' : '创建档案'}</button>
          <button className="btn btn--ghost" onClick={() => navigate('/admin')}>取消</button>
        </div>
      </div>
      {error && <div style={{ color: 'var(--color-danger)', fontSize: 13, marginBottom: 12 }}>{error}</div>}
      <div className="form-row">
        <div className="form-group">
          <label>居住省份</label>
          <input className="form-input" value={provinceName || provinceCode} disabled style={{ background: '#f3f4f6', cursor: 'not-allowed' }} />
        </div>
        <div className="form-group">
          <label>居住城市</label>
          <input className="form-input" value={cityName || cityCode} disabled style={{ background: '#f3f4f6', cursor: 'not-allowed' }} />
        </div>
      </div>
      <div className="form-row">
        <div className="form-group">
          <label>居住镇 *</label>
          <select className="form-input" value={selectedTown} onChange={e => setSelectedTown(e.target.value)}>
            <option value="">请选择镇</option>
            {towns.map(t => <option key={t.town_code} value={t.town_code}>{t.town_name}</option>)}
          </select>
        </div>
        <div className="form-group">
          <label>居住村/路 *</label>
          <select className="form-input" value={selectedVillage} onChange={e => setSelectedVillage(e.target.value)} disabled={!selectedTown}>
            <option value="">请选择村/路</option>
            {villages.map(v => <option key={v.village_id} value={v.village_id}>{v.village_name}</option>)}
          </select>
        </div>
      </div>
      <div className="form-group">
        <label>居住地址 *</label>
        <input className="form-input" placeholder="详细门牌号等（最长100字符）" maxLength={100} value={addressText} onChange={e => setAddressText(e.target.value)} />
      </div>
      <div className="form-row">
        <div className="form-group"><label>姓氏 *</label><input className="form-input" value={form.last_name} onChange={e => set('last_name', e.target.value)} /></div>
        <div className="form-group"><label>名字 *</label><input className="form-input" value={form.first_name} onChange={e => set('first_name', e.target.value)} /></div>
      </div>
      <div className="form-row">
        <div className="form-group"><label>出生日期 *</label><DateInput value={form.birth_date} onChange={setBirthDate} required /></div>
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
          <label>出生省份 *</label>
          <select className="form-input" value={birthProvince} onChange={e => setBirthProvince(e.target.value)}>
            <option value="">请选择出生省份</option>
            {birthProvinces.map(p => <option key={p.province_code} value={p.province_code}>{p.province_name}</option>)}
          </select>
        </div>
        <div className="form-group">
          <label>出生城市 *</label>
          <select className="form-input" value={birthCity} onChange={e => setBirthCity(e.target.value)} disabled={!birthProvince}>
            <option value="">请选择出生城市</option>
            {birthCities.map(c => <option key={c.city_code} value={c.city_code}>{c.city_name}</option>)}
          </select>
        </div>
        <div className="form-group">
          <label>出生镇 *</label>
          <select className="form-input" value={birthTown} onChange={e => setBirthTown(e.target.value)} disabled={!birthCity}>
            <option value="">请选择出生镇</option>
            {birthTowns.map(t => <option key={t.town_code} value={t.town_code}>{t.town_name}</option>)}
          </select>
        </div>
      </div>
      <div className="form-row">
        <div className="form-group">
          <label>公民状态 *</label>
          <select className="form-input" value={form.citizen_status} onChange={e => setCitizenStatus(e.target.value)}>
            <option value="NORMAL">正常</option>
            <option value="REVOKED">注销</option>
          </select>
        </div>
        <div className="form-group">
          <label>选举资格 *</label>
          <select
            className="form-input"
            value={String(canSetVotingEligible ? form.voting_eligible : false)}
            onChange={e => setForm(f => ({ ...f, voting_eligible: e.target.value === 'true' }))}
            disabled={!canSetVotingEligible}
          >
            <option value="true">有选举资格</option>
            <option value="false">无选举资格</option>
          </select>
        </div>
      </div>
    </div>
  );
}
