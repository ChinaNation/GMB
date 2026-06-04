// 中文注释:SFID 前端轻量缓存。
// 只缓存省市代码、确定性机构展示列表和机构详情快照;普通公民/机构精确搜索结果不得放进这里。

import type { AdminAuth } from '../auth/types';
import type { InstitutionDetail, InstitutionListRow } from '../subjects/api';
import { getSfidMeta, listSfidCities, type SfidCityItem, type SfidMetaResult } from './api';

const SFID_META_CACHE_VERSION = 'sfid-meta-v1';
const SFID_CITY_CACHE_VERSION = 'sfid-cities-v1';
const PUBLIC_SECURITY_CACHE_VERSION = 'public-security-v1';
const OFFICIAL_INSTITUTION_CACHE_VERSION = 'official-institutions-v1';
const INSTITUTION_DETAIL_CACHE_VERSION = 'institution-detail-v1';

interface CachedPayload<T> {
  version: string;
  data: T;
}

interface PublicSecurityCachePayload {
  version: string;
  admin_pubkey: string;
  role: string;
  province: string;
  city: string;
  rows: InstitutionListRow[];
}

function readCache<T>(key: string, version: string): T | null {
  try {
    const raw = localStorage.getItem(key);
    if (!raw) return null;
    const parsed = JSON.parse(raw) as CachedPayload<T>;
    if (parsed.version !== version || typeof parsed.data === 'undefined') {
      localStorage.removeItem(key);
      return null;
    }
    return parsed.data;
  } catch {
    localStorage.removeItem(key);
    return null;
  }
}

function writeCache<T>(key: string, version: string, data: T) {
  try {
    localStorage.setItem(key, JSON.stringify({ version, data } satisfies CachedPayload<T>));
  } catch {
    // 中文注释:确定性元数据缓存写入失败不能阻断页面展示。
  }
}

function sfidMetaCacheKey(auth: AdminAuth): string {
  return ['sfid:meta', SFID_META_CACHE_VERSION, auth.admin_pubkey, auth.role].join(':');
}

function sfidCitiesCacheKey(province: string): string {
  return ['sfid:cities', SFID_CITY_CACHE_VERSION, province].join(':');
}

export async function loadCachedSfidMeta(auth: AdminAuth): Promise<SfidMetaResult> {
  const key = sfidMetaCacheKey(auth);
  const cached = readCache<SfidMetaResult>(key, SFID_META_CACHE_VERSION);
  if (cached) return cached;
  const next = await getSfidMeta(auth);
  writeCache(key, SFID_META_CACHE_VERSION, next);
  return next;
}

export async function loadCachedSfidCities(
  auth: AdminAuth,
  province: string,
): Promise<SfidCityItem[]> {
  const key = sfidCitiesCacheKey(province);
  const cached = readCache<SfidCityItem[]>(key, SFID_CITY_CACHE_VERSION);
  if (cached) return cached;
  const rows = await listSfidCities(auth, province);
  writeCache(key, SFID_CITY_CACHE_VERSION, rows);
  return rows;
}

export function readCachedSfidCities(province: string): SfidCityItem[] | null {
  return readCache<SfidCityItem[]>(sfidCitiesCacheKey(province), SFID_CITY_CACHE_VERSION);
}

export function publicSecurityCacheKey(auth: AdminAuth, province: string, city: string): string {
  const scopeCity = auth.admin_city || city || 'ALL';
  const scopeProvince = auth.admin_province || province;
  return [
    'sfid:public-security',
    PUBLIC_SECURITY_CACHE_VERSION,
    auth.admin_pubkey,
    auth.role,
    scopeProvince,
    scopeCity,
  ].join(':');
}

export function readCachedPublicSecurityRows(key: string): InstitutionListRow[] | null {
  try {
    const raw = localStorage.getItem(key);
    if (!raw) return null;
    const parsed = JSON.parse(raw) as PublicSecurityCachePayload;
    if (parsed.version !== PUBLIC_SECURITY_CACHE_VERSION || !Array.isArray(parsed.rows)) {
      localStorage.removeItem(key);
      return null;
    }
    return parsed.rows;
  } catch {
    localStorage.removeItem(key);
    return null;
  }
}

export function writeCachedPublicSecurityRows(
  key: string,
  auth: AdminAuth,
  province: string,
  city: string,
  rows: InstitutionListRow[],
) {
  try {
    localStorage.setItem(
      key,
      JSON.stringify({
        version: PUBLIC_SECURITY_CACHE_VERSION,
        admin_pubkey: auth.admin_pubkey,
        role: auth.role,
        province: auth.admin_province || province,
        city: auth.admin_city || city || 'ALL',
        rows,
      } satisfies PublicSecurityCachePayload),
    );
  } catch {
    // 中文注释:公安局列表缓存只是展示加速,写失败不影响后端权威结果。
  }
}

export function officialInstitutionCacheKey(auth: AdminAuth, province: string, city: string): string {
  const scopeCity = auth.admin_city || city || 'ALL';
  const scopeProvince = auth.admin_province || province;
  return [
    'sfid:official-institutions',
    OFFICIAL_INSTITUTION_CACHE_VERSION,
    auth.admin_pubkey,
    auth.role,
    scopeProvince,
    scopeCity,
  ].join(':');
}

export function readCachedOfficialInstitutionRows(key: string): InstitutionListRow[] | null {
  try {
    const raw = localStorage.getItem(key);
    if (!raw) return null;
    const parsed = JSON.parse(raw) as PublicSecurityCachePayload;
    if (parsed.version !== OFFICIAL_INSTITUTION_CACHE_VERSION || !Array.isArray(parsed.rows)) {
      localStorage.removeItem(key);
      return null;
    }
    return parsed.rows;
  } catch {
    localStorage.removeItem(key);
    return null;
  }
}

export function writeCachedOfficialInstitutionRows(
  key: string,
  auth: AdminAuth,
  province: string,
  city: string,
  rows: InstitutionListRow[],
) {
  try {
    localStorage.setItem(
      key,
      JSON.stringify({
        version: OFFICIAL_INSTITUTION_CACHE_VERSION,
        admin_pubkey: auth.admin_pubkey,
        role: auth.role,
        province: auth.admin_province || province,
        city: auth.admin_city || city || 'ALL',
        rows,
      } satisfies PublicSecurityCachePayload),
    );
  } catch {
    // 中文注释:公权机构确定性列表缓存只是展示加速,写失败不影响后端权威结果。
  }
}

export function institutionDetailCacheKey(auth: AdminAuth, sfidNumber: string): string {
  return [
    'sfid:institution-detail',
    INSTITUTION_DETAIL_CACHE_VERSION,
    auth.admin_pubkey,
    auth.role,
    sfidNumber,
  ].join(':');
}

export function readCachedInstitutionDetail(key: string): InstitutionDetail | null {
  return readCache<InstitutionDetail>(key, INSTITUTION_DETAIL_CACHE_VERSION);
}

export function writeCachedInstitutionDetail(key: string, detail: InstitutionDetail) {
  writeCache(key, INSTITUTION_DETAIL_CACHE_VERSION, detail);
}
