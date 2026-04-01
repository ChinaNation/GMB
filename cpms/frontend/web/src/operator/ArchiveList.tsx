import { useState, useEffect, useCallback } from 'react';
import { useNavigate } from 'react-router-dom';
import * as api from '../api';
import type { Archive } from '../types';

const PAGE_SIZE = 20;

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
      const res = await api.listArchives({ full_name: search || undefined, page, page_size: PAGE_SIZE });
      if (res.data) {
        setArchives(res.data.archives || []);
        setTotal(res.data.total || 0);
      }
    } catch { /* ignore */ }
    setLoading(false);
  }, [search, page]);

  useEffect(() => { load(); }, [load]);

  const totalPages = Math.max(1, Math.ceil(total / PAGE_SIZE));

  return (
    <div className="card">
      <div className="card__title flex-between">
        公民信息
        <button className="btn btn--primary" onClick={() => navigate('/admin/create')}>+ 新建档案</button>
      </div>

      <div style={{ display: 'flex', gap: 8, marginBottom: 16 }}>
        <input className="form-input" style={{ maxWidth: 260 }} placeholder="按姓名搜索" value={search} onChange={e => setSearch(e.target.value)} />
        <button className="btn btn--blue" onClick={() => { setPage(1); load(); }}>搜索</button>
      </div>

      <table className="table">
        <thead>
          <tr>
            <th>档案号</th>
            <th>姓名</th>
            <th>性别</th>
            <th>省份</th>
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
              <td><span className="text-ellipsis">{a.archive_no}</span></td>
              <td>{a.full_name}</td>
              <td>{a.gender_code === 'M' ? '男' : '女'}</td>
              <td>{a.province_code}</td>
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
