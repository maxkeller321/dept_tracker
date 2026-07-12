import { describe, it, expect } from 'vitest';
import { compareIsoDates, resolvePlanAction } from './sonderzahlung';

describe('compareIsoDates', () => {
  it('orders dates chronologically', () => {
    expect(compareIsoDates('2025-01-01', '2025-06-01')).toBeLessThan(0);
    expect(compareIsoDates('2025-06-01', '2025-01-01')).toBeGreaterThan(0);
    expect(compareIsoDates('2025-06-01', '2025-06-01')).toBe(0);
  });
});

describe('resolvePlanAction', () => {
  const today = '2025-06-15';

  it('schedules future dates', () => {
    expect(resolvePlanAction('2025-07-01', today)).toBe('schedule');
  });

  it('backdates today', () => {
    expect(resolvePlanAction('2025-06-15', today)).toBe('backdated');
  });

  it('backdates past dates', () => {
    expect(resolvePlanAction('2025-01-01', today)).toBe('backdated');
  });
});
