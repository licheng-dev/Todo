const WEEK = ["周日", "周一", "周二", "周三", "周四", "周五", "周六"];

export function dateKey(ms: number): string {
  const d = new Date(ms);
  return keyOf(d.getFullYear(), d.getMonth(), d.getDate());
}

export function keyOf(year: number, month: number, day: number): string {
  const m = String(month + 1).padStart(2, "0");
  const d = String(day).padStart(2, "0");
  return `${year}-${m}-${d}`;
}

export function todayKey(): string {
  return dateKey(Date.now());
}

export function parseKey(key: string): { year: number; month: number; day: number } {
  const [year, month, day] = key.split("-").map(Number);
  return { year, month: month - 1, day };
}

export function formatDateLabel(key: string): string {
  const { year, month, day } = parseKey(key);
  const date = new Date(year, month, day);
  return `${month + 1}月${day}日 ${WEEK[date.getDay()]}`;
}
