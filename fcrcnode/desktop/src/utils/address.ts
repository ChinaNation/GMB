export function isValidAddress(value: string): boolean {
  const address = value.trim();
  if (!address) return false;
  if (address.startsWith('0x')) return address.length === 66;
  return /^5[1-9A-HJ-NP-Za-km-z]{46,48}$/.test(address);
}
