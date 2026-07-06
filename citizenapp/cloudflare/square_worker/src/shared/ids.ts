export function createId(prefix: string): string {
  return `${prefix}_${crypto.randomUUID().replaceAll('-', '')}`;
}

export function assertOwnerAccount(value: unknown): string {
  if (typeof value !== 'string') {
    throw new Error('owner_account must be string');
  }

  const ownerAccount = value.trim();
  if (ownerAccount.length < 16 || ownerAccount.length > 128 || ownerAccount.includes('/')) {
    throw new Error('owner_account is invalid');
  }

  return ownerAccount;
}
