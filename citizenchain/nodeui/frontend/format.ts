/**
 * 金额千分位格式化工具。
 *
 * @example
 * formatAmount("1234567.89") // "1,234,567.89"
 * formatAmount("100")        // "100"
 * formatAmount(null)         // null
 */
export function formatAmount(value: string | null | undefined): string | null {
  if (value == null) return null;
  const trimmed = value.trim();
  if (!trimmed) return null;

  const match = trimmed.match(/^(-?[\d.]+)(.*)$/);
  if (!match) return trimmed;

  const [, numPart, suffix] = match;
  const [intPart, decimal] = numPart.split('.');
  const formatted = intPart.replace(/\B(?=(\d{3})+(?!\d))/g, ',');
  const decimalStr = decimal != null ? `.${decimal}` : '';
  return `${formatted}${decimalStr}${suffix}`;
}
