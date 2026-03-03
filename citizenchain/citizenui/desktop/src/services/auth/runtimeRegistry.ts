import { invoke, isTauri } from '@tauri-apps/api/core';
import type { OrganizationRegistryItem } from '../../constants/orgRegistry.types';

type RegistrySnapshot = {
  version: number;
  generated_at: number;
  records: OrganizationRegistryItem[];
};

export async function loadRuntimeOrgRegistry(): Promise<OrganizationRegistryItem[] | null> {
  if (!isTauri()) {
    return null;
  }
  try {
    const raw = await invoke<string | null>('read_org_registry_snapshot_json');
    if (!raw) {
      return null;
    }
    const parsed = JSON.parse(raw) as Partial<RegistrySnapshot>;
    if (!Array.isArray(parsed.records)) {
      return null;
    }
    return parsed.records.filter(
      (item): item is OrganizationRegistryItem =>
        typeof item === 'object' &&
        item !== null &&
        (item.role === 'nrc' || item.role === 'prc' || item.role === 'prb' || item.role === 'full') &&
        typeof item.organizationName === 'string' &&
        typeof item.adminAddress === 'string'
    );
  } catch {
    return null;
  }
}
