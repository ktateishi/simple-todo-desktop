export const DAY_MS = 86_400_000;

/** その日の 0:00（ローカルタイム）のタイムスタンプを返す */
export function startOfDay(ts: number): number {
  const d = new Date(ts);
  return new Date(d.getFullYear(), d.getMonth(), d.getDate()).getTime();
}

/** ts から n 日後の 0:00 を返す（DST 跨ぎも Date 経由で吸収） */
export function addDays(ts: number, n: number): number {
  const d = new Date(ts);
  return new Date(d.getFullYear(), d.getMonth(), d.getDate() + n).getTime();
}

/** a（0:00）から b（0:00）までの日数差 */
export function daysBetween(a: number, b: number): number {
  return Math.round((startOfDay(b) - startOfDay(a)) / DAY_MS);
}

export function formatMonth(ts: number): string {
  const d = new Date(ts);
  return `${d.getFullYear()}年${d.getMonth() + 1}月`;
}

export function formatDate(ts: number): string {
  return new Date(ts).toLocaleDateString('ja-JP', { month: 'numeric', day: 'numeric' });
}

export function dayOfMonth(ts: number): number {
  return new Date(ts).getDate();
}

/** 0=Sun ... 6=Sat */
export function weekday(ts: number): number {
  return new Date(ts).getDay();
}
