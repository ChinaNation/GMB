import { useState, useEffect, useCallback } from 'react';
import { useNavigate } from 'react-router-dom';
import * as api from '../api';
import type { Archive } from '../types';

const PAGE_SIZE = 20;

function calcAge(birthDate: string): string {
  if (!/^\d{4}-\d{2}-\d{2}$/.test(birthDate)) return '-';
  const birth = new Date(`${birthDate}T00:00:00`);
  if (Number.isNaN(birth.getTime())) return '-';
  const today = new Date();
  let age = today.getFullYear() - birth.getFullYear();
  const passedBirthday = today.getMonth() > birth.getMonth()
    || (today.getMonth() === birth.getMonth() && today.getDate() >= birth.getDate());
  if (!passedBirthday) age -= 1;
  return age >= 0 ? `${age}岁` : '-';
}

export default function ArchiveList() {
  const navigate = useNavigate();
  const [archives, setArchives] = useState<Archive[]>([]);
  const [total, setTotal] = useState(0);
  const [page, setPage] = useState(1);
  const [search, setSearch] = useState('');
  const [loading, setLoading] = useState(false);

  const load = useCallback(async () => {
    setLoading(true);
    try {
      const res = await api.listArchives({ q: search.trim() || undefined, page, page_size: PAGE_SIZE });
      if (res.data) {
        setArchives(res.data.items || []);
        setTotal(res.data.total || 0);
      }
    } catch { /* ignore */ }
    setLoading(false);
  }, [search, page]);

  useEffect(() => { load(); }, [load]);

  const totalPages = Math.max(1, Math.ceil(total / PAGE_SIZE));

  return (
    <div className="card">
      <div className="card__title" style={{ textAlign: 'center', borderLeft: 'none', paddingLeft: 0 }}>
        公民信息列表
      </div>
      <div style={{ display: 'flex', gap: 8, marginBottom: 16, alignItems: 'center', justifyContent: 'space-between' }}>
        <div style={{ display: 'flex', gap: 8, alignItems: 'center' }}>
          <input className="form-input" style={{ width: 360 }} placeholder="按姓名、档案号搜索" value={search} onChange={e => setSearch(e.target.value)} />
          <button className="btn btn--blue" onClick={() => { setPage(1); load(); }}>搜索</button>
        </div>
        <button className="btn btn--primary" onClick={() => navigate('/admin/create')}>+ 新建档案</button>
      </div>

      <table className="table">
        <thead>
          <tr>
            <th>档案号</th>
            <th>姓名</th>
            <th>性别</th>
            <th>年龄</th>
            <th>公民状态</th>
            <th>创建时间</th>
            <th>操作</th>
          </tr>
        </thead>
        <tbody>
          {loading ? (
            <tr><td colSpan={7} className="text-center">加载中...</td></tr>
          ) : archives.length === 0 ? (
            <tr><td colSpan={7} className="text-center" style={{ color: 'var(--color-text-secondary)' }}>暂无数据</td></tr>
          ) : archives.map(a => (
            <tr key={a.archive_id}>
              <td style={{ fontFamily: 'monospace', whiteSpace: 'nowrap' }}>{a.archive_no}</td>
              <td>{a.last_name}{a.first_name}</td>
              <td>{a.gender_code === 'M' ? '男' : '女'}</td>
              <td>{calcAge(a.birth_date)}</td>
              <td>
                <span className={`tag ${a.citizen_status === 'NORMAL' ? 'tag--success' : 'tag--danger'}`}>
                  {a.citizen_status === 'NORMAL' ? '正常' : '异常'}
                </span>
              </td>
              <td>{new Date(a.created_at * 1000).toLocaleDateString()}</td>
              <td><button className="btn btn--ghost btn--sm" onClick={() => navigate(`/admin/archives/${a.archive_id}`)}>详情</button></td>
            </tr>
          ))}
        </tbody>
      </table>

      <div className="pagination">
        <span>共 {total} 条</span>
        <button disabled={page <= 1} onClick={() => setPage(p => p - 1)}>上一页</button>
        <span>{page} / {totalPages}</span>
        <button disabled={page >= totalPages} onClick={() => setPage(p => p + 1)}>下一页</button>
      </div>
    </div>
  );
}
