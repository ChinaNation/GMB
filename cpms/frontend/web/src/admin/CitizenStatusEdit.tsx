import { useState } from 'react';
import * as api from '../api';

export default function CitizenStatusEdit() {
  const [archiveId, setArchiveId] = useState('');
  const [status, setStatus] = useState('NORMAL');
  const [result, setResult] = useState('');
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);

  const handleSubmit = async () => {
    if (!archiveId.trim()) { setError('请输入档案 ID'); return; }
    setError(''); setResult('');
    setLoading(true);
    try {
      const res = await api.updateCitizenStatus(archiveId.trim(), status);
      if (res.data) setResult(`档案 ${res.data.archive_id} 公民状态已更新为 ${res.data.citizen_status}`);
    } catch (e) {
      setError(e instanceof Error ? e.message : '更新失败');
    }
    setLoading(false);
  };

  return (
    <div className="card">
      <div className="card__title">公民状态变更</div>
      {error && <div style={{ color: 'var(--color-danger)', fontSize: 13, marginBottom: 12 }}>{error}</div>}
      {result && <div style={{ color: 'var(--color-success)', fontSize: 13, marginBottom: 12 }}>{result}</div>}
      <div className="form-group"><label>档案 ID</label><input className="form-input" placeholder="ar_xxxx" value={archiveId} onChange={e => setArchiveId(e.target.value)} /></div>
      <div className="form-group">
        <label>目标状态</label>
        <select className="form-input" value={status} onChange={e => setStatus(e.target.value)}>
          <option value="NORMAL">正常 (NORMAL)</option>
          <option value="ABNORMAL">异常 (ABNORMAL)</option>
        </select>
      </div>
      <button className="btn btn--primary" onClick={handleSubmit} disabled={loading}>
        {loading ? '提交中...' : '更新公民状态'}
      </button>
    </div>
  );
}
