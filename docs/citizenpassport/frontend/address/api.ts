import { get } from '../common/http';
import type { AddressUnit, City, Province, Town } from './types';

export const listTowns = () => get<Town[]>('/api/v1/address/towns');

export const listAddressUnits = (town_code: string) =>
  get<AddressUnit[]>(`/api/v1/address/units?town_code=${encodeURIComponent(town_code)}`);

export const listBirthProvinces = () =>
  get<Province[]>('/api/v1/address/china/provinces');

export const listBirthCities = (province_code: string) =>
  get<City[]>(`/api/v1/address/china/cities?province_code=${encodeURIComponent(province_code)}`);

export const listBirthTowns = (province_code: string, city_code: string) =>
  get<Town[]>(
    `/api/v1/address/china/towns?province_code=${encodeURIComponent(province_code)}&city_code=${encodeURIComponent(city_code)}`
  );
