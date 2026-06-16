import { get } from '../common/http';
import type { City, Province, Town, Village } from './types';

export const listTowns = () => get<Town[]>('/api/v1/address/towns');

export const listVillages = (town_code: string) =>
  get<Village[]>(`/api/v1/address/villages?town_code=${encodeURIComponent(town_code)}`);

export const listBirthProvinces = () =>
  get<Province[]>('/api/v1/address/china/provinces');

export const listBirthCities = (province_code: string) =>
  get<City[]>(`/api/v1/address/china/cities?province_code=${encodeURIComponent(province_code)}`);

export const listBirthTowns = (province_code: string, city_code: string) =>
  get<Town[]>(
    `/api/v1/address/china/towns?province_code=${encodeURIComponent(province_code)}&city_code=${encodeURIComponent(city_code)}`
  );
