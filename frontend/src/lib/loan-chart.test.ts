import { describe, it, expect } from 'vitest';
import { buildLoanChartSlices, describeDonutSegment } from './loan-chart';

describe('buildLoanChartSlices', () => {
  it('builds proportional slices', () => {
    const slices = buildLoanChartSlices([
      { id: '1', label: 'ING', amountMinor: 60_000_00 },
      { id: '2', label: 'Sparkasse', amountMinor: 40_000_00 },
    ]);
    expect(slices).toHaveLength(2);
    expect(slices[0].percent).toBeCloseTo(60, 5);
    expect(slices[1].percent).toBeCloseTo(40, 5);
    expect(slices[0].endAngle - slices[0].startAngle).toBeCloseTo(216, 5);
  });

  it('skips zero-balance loans', () => {
    const slices = buildLoanChartSlices([
      { id: '1', label: 'Paid off', amountMinor: 0 },
      { id: '2', label: 'Active', amountMinor: 10_000_00 },
    ]);
    expect(slices).toHaveLength(1);
    expect(slices[0].label).toBe('Active');
  });

  it('returns empty when all balances are zero', () => {
    expect(buildLoanChartSlices([{ id: '1', label: 'X', amountMinor: 0 }])).toEqual([]);
  });
});

describe('describeDonutSegment', () => {
  it('returns a closed path', () => {
    const path = describeDonutSegment(100, 100, 80, 50, 0, 90);
    expect(path.startsWith('M ')).toBe(true);
    expect(path.endsWith('Z')).toBe(true);
  });

  it('handles a full ring', () => {
    const path = describeDonutSegment(100, 100, 80, 50, -90, 270);
    expect(path.length).toBeGreaterThan(20);
  });
});
