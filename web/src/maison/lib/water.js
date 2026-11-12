/** Dates stay in the provider's calendar; missing days never become zero consumption. */
export function waterMonth(days, month) {
  const [year, number] = month.split('-').map(Number);
  if (!/^\d{4}-\d{2}$/.test(month) || number < 1 || number > 12) return [];
  return Array.from({length:new Date(year, number, 0).getDate()}, (_, i) => {
    const date = `${month}-${String(i+1).padStart(2,'0')}`;
    const value = days?.[date];
    return {date, value:typeof value === 'number' && Number.isFinite(value) && value >= 0 ? value : null};
  });
}
export function waterTotal(rows) {
  const measured = rows.filter(row => row.value != null);
  return measured.length ? measured.reduce((sum,row)=>sum+row.value,0) : null;
}
export function localMonth(now = new Date()) {
  return `${now.getFullYear()}-${String(now.getMonth()+1).padStart(2,'0')}`;
}

export function localDate(now = new Date()) {
  return `${localMonth(now)}-${String(now.getDate()).padStart(2, '0')}`;
}
