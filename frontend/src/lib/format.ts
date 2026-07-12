export function formatMoney(amountMinor: number, currency = 'EUR'): string {
  const value = amountMinor / 100;
  return new Intl.NumberFormat(undefined, { style: 'currency', currency }).format(value);
}

/** Parse an ISO date string as **local** midnight to avoid UTC-offset shifts. */
export function parseLocalDate(iso: string): Date {
  // "YYYY-MM-DD" → append T00:00:00 so the browser treats it as local time,
  // not as UTC midnight (which can display as the previous day in negative-offset zones).
  return new Date(iso.length === 10 ? iso + 'T00:00:00' : iso);
}

export function formatDate(iso: string | null | undefined): string {
  if (!iso) return '—';
  try {
    return parseLocalDate(iso).toLocaleDateString();
  } catch {
    return iso;
  }
}

export function formatLastPayment(iso: string | null | undefined, fallback = '—'): string {
  if (!iso) return fallback;
  return formatDate(iso);
}

const intlLocales: Record<string, string> = {
  en: 'en',
  de: 'de-DE',
  es: 'es',
  fr: 'fr-FR',
};

export function formatPercent(value: number, locale = 'en'): string {
  const tag = intlLocales[locale] ?? locale;
  return new Intl.NumberFormat(tag, {
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  }).format(value);
}

export function todayIso(): string {
  return new Date().toISOString().slice(0, 10);
}
