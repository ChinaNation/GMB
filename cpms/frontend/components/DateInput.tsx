import type { CSSProperties } from 'react';

function formatLocalYmd(date: Date): string {
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, '0');
  const day = String(date.getDate()).padStart(2, '0');
  return `${year}-${month}-${day}`;
}

function todayYmd(): string {
  return formatLocalYmd(new Date());
}

function yesterdayYmd(): string {
  return formatLocalYmd(new Date(Date.now() - 24 * 60 * 60 * 1000));
}

function parseYmd(value: string): Date | null {
  if (!/^\d{4}-\d{2}-\d{2}$/.test(value)) return null;
  const [year, month, day] = value.split('-').map(Number);
  const date = new Date(year, month - 1, day);
  if (
    date.getFullYear() !== year
    || date.getMonth() !== month - 1
    || date.getDate() !== day
  ) {
    return null;
  }
  return date;
}

export function isPastYmd(value: string): boolean {
  return parseYmd(value) !== null && value < todayYmd();
}

export function isAtLeastAgeYmd(value: string, years: number): boolean {
  const birth = parseYmd(value);
  if (!birth) return false;
  const today = new Date();
  let age = today.getFullYear() - birth.getFullYear();
  const passedBirthday = today.getMonth() > birth.getMonth()
    || (today.getMonth() === birth.getMonth() && today.getDate() >= birth.getDate());
  if (!passedBirthday) age -= 1;
  return age >= years;
}

interface DateInputProps {
  value: string;
  onChange: (value: string) => void;
  max?: string | null;
  min?: string;
  disabled?: boolean;
  required?: boolean;
  style?: CSSProperties;
  className?: string;
}

export default function DateInput({
  value,
  onChange,
  max,
  min,
  disabled,
  required,
  style,
  className = 'form-input',
}: DateInputProps) {
  const resolvedMax = max === null ? undefined : (max ?? yesterdayYmd());

  return (
    <input
      className={className}
      type="date"
      value={value}
      min={min}
      max={resolvedMax}
      disabled={disabled}
      required={required}
      style={style}
      onChange={event => onChange(event.target.value)}
    />
  );
}
