import { describe, it, expect } from 'vitest';
import { formatMoney, formatDate, formatPercent, parseLocalDate } from './format';

describe('formatMoney', () => {
  it('formats EUR minor units', () => {
    const s = formatMoney(12345, 'EUR');
    expect(s).toContain('123');
  });
});

describe('formatDate', () => {
  it('returns placeholder for null', () => {
    expect(formatDate(null)).toBe('—');
  });

  it('formats iso date', () => {
    expect(formatDate('2025-06-01')).not.toBe('No payments recorded yet');
  });
});

describe('formatPercent', () => {
  it('shows two decimal places in German locale', () => {
    expect(formatPercent(0, 'de')).toBe('0,00');
    expect(formatPercent(12.345, 'de')).toBe('12,35');
  });

  it('shows two decimal places in English locale', () => {
    expect(formatPercent(0, 'en')).toBe('0.00');
  });
});

// Regression: Bug 3 – parseLocalDate must keep the correct calendar day
// regardless of the local timezone offset.
describe('parseLocalDate', () => {
  it('returns the correct year/month/day for a YYYY-MM-DD string', () => {
    const d = parseLocalDate('2030-05-01');
    expect(d.getFullYear()).toBe(2030);
    expect(d.getMonth()).toBe(4); // 0-based: 4 = May
    expect(d.getDate()).toBe(1);
  });

  it('does not shift the date into the previous day in negative UTC offsets', () => {
    // new Date("2030-05-01") (without T00:00:00) is UTC midnight which in UTC-12
    // would render as April 30. parseLocalDate avoids this.
    const d = parseLocalDate('2030-05-01');
    expect(d.getDate()).toBe(1); // must be the 1st, not the 30th
  });

  it('passes through datetime strings unchanged', () => {
    const iso = '2030-05-01T14:30:00';
    const d = parseLocalDate(iso);
    expect(d.getFullYear()).toBe(2030);
    expect(d.getMonth()).toBe(4);
    expect(d.getDate()).toBe(1);
  });
});
