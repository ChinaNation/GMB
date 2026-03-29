import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import * as api from '../api';

export default function ArchiveCreate() {
  const navigate = useNavigate();
  const [form, setForm] = useState({
    province_code: '', city_code: '', full_name: '', birth_date: '',
    gender_code: 'M', height_cm: '', passport_no: '', citizen_status: 'NORMAL',
  });
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');

  const set = (k: string, v: string) => setForm(f => ({ ...f, [k]: v }));

  const handleSubmit = async () => {
    if (!form.full_name.trim()) { setError('请输入姓名'); return; }
    setError('');
    setLoading(true);
    try {
      const body = {
        ...form,
        height_cm: form.height_cm ? parseFloat(form.height_cm) : undefined,
      };
      const res = await api.createArchive(body);
      if (res.data) navigate(`/operator/archives/${res.data.archive_id}`);
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
        <div className="form-group"><label>省份代码</label><input className="form-input" placeholder="如 GD" value={form.province_code} onChange={e => set('province_code', e.target.value)} /></div>
        <div className="form-group"><label>城市代码</label><input className="form-input" placeholder="如 001" value={form.city_code} onChange={e => set('city_code', e.target.value)} /></div>
      </div>
      <div className="form-row">
        <div className="form-group"><label>姓名</label><input className="form-input" value={form.full_name} onChange={e => set('full_name', e.target.value)} /></div>
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
      <div className="form-group"><label>护照号</label><input className="form-input" value={form.passport_no} onChange={e => set('passport_no', e.target.value)} /></div>
      <div style={{ display: 'flex', gap: 8, marginTop: 20 }}>
        <button className="btn btn--primary" onClick={handleSubmit} disabled={loading}>{loading ? '提交中...' : '创建档案'}</button>
        <button className="btn btn--ghost" onClick={() => navigate('/operator')}>取消</button>
      </div>
    </div>
  );
}
