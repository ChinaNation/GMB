import { get } from '../common/http';
import type { Town, Village } from './types';

export const listTowns = () => get<Town[]>('/api/v1/address/towns');

export const listVillages = (town_code: string) =>
  get<Village[]>(`/api/v1/address/villages?town_code=${encodeURIComponent(town_code)}`);
