import { useState, useEffect, useCallback } from 'react';
import { useNavigate } from 'react-router-dom';
import { listArchives, type ArchiveListParams } from './api';
import { listTowns } from '../address/api';
import { installStatus } from '../initialize/api';
import type { Town } from '../address/types';
import type { Archive } from './types';

const PAGE_SIZE_OPTIONS = [20, 50, 100] as const;
const CENTER_CELL_STYLE = { textAlign: 'center' as const };

interface ArchiveFilters {
  value: string;
  birthDate: string;
}

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
  const [totalActive, setTotalActive] = useState(0);
  const [limit, setLimit] = useState<number>(50);
  const [cursor, setCursor] = useState<string | null>(null);
  const [cursorStack, setCursorStack] = useState<Array<string | null>>([]);
  const [nextCursor, setNextCursor] = useState<string | null>(null);
  const [hasNext, setHasNext] = useState(false);
  const [draftValue, setDraftValue] = useState('');
  const [draftBirthDate, setDraftBirthDate] = useState('');
  const [filters, setFilters] = useState<ArchiveFilters>({ value: '', birthDate: '' });
  const [loading, setLoading] = useState(false);
  const [scopeProvince, setScopeProvince] = useState('');
  const [scopeCity, setScopeCity] = useState('');
  const [townNames, setTownNames] = useState<Record<string, string>>({});

  const load = useCallback(async () => {
    setLoading(true);
    try {
      const params: ArchiveListParams = { limit };
      if (cursor) params.cursor = cursor;
      const value = filters.value.trim();
      if (value) params.search = value;
      if (filters.birthDate.trim()) params.birth_date = filters.birthDate.trim();
      const res = await listArchives(params);
      if (res.data) {
        setArchives(res.data.items || []);
        setTotalActive(res.data.total_active || 0);
        setNextCursor(res.data.next_cursor || null);
        setHasNext(Boolean(res.data.has_next));
      }
    } catch { /* ignore */ }
    setLoading(false);
  }, [cursor, filters, limit]);

  useEffect(() => { load(); }, [load]);

  useEffect(() => {
    installStatus()
      .then(res => {
        const province = res.data?.province_name || res.data?.province_code || '';
        const city = res.data?.city_name || res.data?.city_code || '';
        setScopeProvince(province);
        setScopeCity(city);
      })
      .catch(() => {
        setScopeProvince('');
        setScopeCity('');
      });
  }, []);

  useEffect(() => {
    listTowns()
      .then(res => {
        const towns = res.data || [];
        // 中文注释：列表只保存 town_code，市镇列用当前市地址字典映射成镇/街道名称。
        setTownNames(Object.fromEntries(towns.map((town: Town) => [town.town_code, town.town_name])));
      })
      .catch(() => setTownNames({}));
  }, []);

  const hasScope = Boolean(scopeProvince || scopeCity);
  const townLabel = (archive: Archive) => townNames[archive.town_code] || archive.town_code || '-';
  const resetToFirstPage = () => {
    setCursor(null);
    setCursorStack([]);
  };
  const applySearch = () => {
    setFilters({
      value: draftValue.trim(),
      birthDate: draftBirthDate.trim(),
    });
    resetToFirstPage();
  };
  const goNext = () => {
    if (!hasNext || !nextCursor) return;
    setCursorStack(stack => [...stack, cursor]);
    setCursor(nextCursor);
  };
  const goPrev = () => {
    setCursorStack(stack => {
      const next = [...stack];
      const previous = next.pop() ?? null;
      setCursor(previous);
      return next;
    });
  };

  return (
    <div className="card">
      <div className="card__title" style={{ borderLeft: 'none', paddingLeft: 0 }}>
        <div style={{ display: 'grid', gridTemplateColumns: '1fr auto 1fr', alignItems: 'center', gap: 12 }}>
          <span style={{ color: 'var(--color-text-secondary)', fontSize: 13, fontWeight: 500, display: 'inline-flex', alignItems: 'center', gap: 6 }}>
            {hasScope ? (
              <>
                {scopeProvince && <span>{scopeProvince}</span>}
                {scopeProvince && scopeCity && (
                  <span style={{ display: 'inline-flex', alignItems: 'center', justifyContent: 'center', width: 8, lineHeight: 1 }}>·</span>
                )}
                {scopeCity && <span>{scopeCity}</span>}
              </>
            ) : '—'}
          </span>
          <span>公民档案列表</span>
          <span />
        </div>
      </div>
      <div style={{ display: 'flex', gap: 8, marginBottom: 16, alignItems: 'center', justifyContent: 'space-between' }}>
        <div style={{ display: 'flex', gap: 8, alignItems: 'center' }}>
          <input className="form-input" style={{ width: 408 }} placeholder="请输入姓名、护照号、档案号检索" value={draftValue} onChange={e => setDraftValue(e.target.value)} />
          <input className="form-input" style={{ width: 150 }} type="date" value={draftBirthDate} onChange={e => setDraftBirthDate(e.target.value)} />
          <button className="btn btn--blue" onClick={applySearch}>搜索</button>
          <button className="btn btn--ghost" onClick={() => { setDraftValue(''); setDraftBirthDate(''); setFilters({ value: '', birthDate: '' }); resetToFirstPage(); }}>清空</button>
        </div>
        <button className="btn btn--primary" onClick={() => navigate('/admin/create')}>+ 新建档案</button>
      </div>

      <table className="table">
        <thead>
          <tr>
            <th style={CENTER_CELL_STYLE}>档案号</th>
            <th style={CENTER_CELL_STYLE}>姓名</th>
            <th style={CENTER_CELL_STYLE}>性别</th>
            <th style={CENTER_CELL_STYLE}>年龄</th>
            <th style={CENTER_CELL_STYLE}>市镇</th>
            <th style={CENTER_CELL_STYLE}>公民状态</th>
            <th style={CENTER_CELL_STYLE}>创建时间</th>
            <th style={CENTER_CELL_STYLE}>操作</th>
          </tr>
        </thead>
        <tbody>
          {loading ? (
            <tr><td colSpan={8} className="text-center">加载中...</td></tr>
          ) : archives.length === 0 ? (
            <tr><td colSpan={8} className="text-center" style={{ color: 'var(--color-text-secondary)' }}>暂无数据</td></tr>
          ) : archives.map(a => (
            <tr key={a.archive_id}>
              <td style={{ ...CENTER_CELL_STYLE, fontFamily: 'monospace', whiteSpace: 'nowrap' }}>{a.archive_no}</td>
              <td style={CENTER_CELL_STYLE}>{a.last_name}{a.first_name}</td>
              <td style={CENTER_CELL_STYLE}>{a.gender_code === 'M' ? '男' : '女'}</td>
              <td style={CENTER_CELL_STYLE}>{calcAge(a.birth_date)}</td>
              <td style={{ ...CENTER_CELL_STYLE, whiteSpace: 'nowrap' }}>{townLabel(a)}</td>
              <td style={CENTER_CELL_STYLE}>
                <span className={`tag ${a.citizen_status === 'NORMAL' ? 'tag--success' : 'tag--danger'}`}>
                  {a.citizen_status === 'NORMAL' ? '正常' : '注销'}
                </span>
              </td>
              <td style={CENTER_CELL_STYLE}>{new Date(a.created_at * 1000).toLocaleDateString()}</td>
              <td style={CENTER_CELL_STYLE}><button className="btn btn--ghost btn--sm" onClick={() => navigate(`/admin/archives/${a.archive_id}`)}>详情</button></td>
            </tr>
          ))}
        </tbody>
      </table>

      <div className="pagination">
        <span>共 {totalActive} 条</span>
        <span>本页 {archives.length} 条</span>
        <select className="form-input" style={{ width: 92 }} value={limit} onChange={e => { setLimit(Number(e.target.value)); resetToFirstPage(); }}>
          {PAGE_SIZE_OPTIONS.map(size => <option key={size} value={size}>{size} 条</option>)}
        </select>
        <button disabled={cursorStack.length === 0} onClick={goPrev}>上一页</button>
        <button disabled={!hasNext} onClick={goNext}>下一页</button>
      </div>
    </div>
  );
}
