import { describe, it, expect } from 'vitest';
import {
  buildChartLayout,
  buildExtendedLayers,
  buildStackedLayers,
  closestDataIndex,
  downsampleTimeline,
  formatAxisMoney,
  formatMonthLabel,
  splitAreaPaths,
  splitExtendedPaths,
  type PayoffTimelineData,
} from './payoff-timeline-chart';

const sample: PayoffTimelineData = {
  dates: ['2025-05-01', '2025-06-01', '2025-07-01', '2025-08-01'],
  as_of_index: 1,
  series: [
    {
      id: '1',
      label: 'ING',
      balances_minor: [60_000_00, 59_000_00, 58_000_00, 57_000_00],
      interest_remaining_minor: [10_000_00, 9_500_00, 9_000_00, 0],
    },
    {
      id: '2',
      label: 'Sparkasse',
      balances_minor: [40_000_00, 39_500_00, 39_000_00, 0],
      interest_remaining_minor: [5_000_00, 4_750_00, 4_500_00, 0],
    },
  ],
};

describe('buildStackedLayers', () => {
  it('stacks bottom layer from zero', () => {
    const layers = buildStackedLayers(sample);
    expect(layers[0].lower[0]).toBe(0);
    expect(layers[0].upper[0]).toBe(60_000_00);
  });

  it('stacks second layer above first', () => {
    const layers = buildStackedLayers(sample);
    expect(layers[1].lower[0]).toBe(60_000_00);
    expect(layers[1].upper[0]).toBe(100_000_00);
  });
});

describe('buildExtendedLayers', () => {
  it('first loan principalLower starts at zero', () => {
    const ext = buildExtendedLayers(sample);
    expect(ext[0].principalLower[0]).toBe(0);
    expect(ext[0].principalUpper[0]).toBe(60_000_00);
    expect(ext[0].interestUpper[0]).toBe(70_000_00);
  });

  it('second loan sits above first loan principal + interest', () => {
    const ext = buildExtendedLayers(sample);
    // Below = principal_0 + interest_0 = 60k + 10k = 70k
    expect(ext[1].principalLower[0]).toBe(70_000_00);
    expect(ext[1].principalUpper[0]).toBe(110_000_00);
    expect(ext[1].interestUpper[0]).toBe(115_000_00);
  });

  it('last point has zero interest', () => {
    const ext = buildExtendedLayers(sample);
    const lastIdx = sample.dates.length - 1;
    expect(ext[0].interestUpper[lastIdx]).toBe(ext[0].principalUpper[lastIdx]);
  });
});

describe('splitAreaPaths', () => {
  it('produces non-empty past and future paths', () => {
    const layers = buildStackedLayers(sample);
    const layout = buildChartLayout(sample);
    const { past, future } = splitAreaPaths(layers[0], layout, sample.as_of_index);
    expect(past.startsWith('M ')).toBe(true);
    expect(past.endsWith('Z')).toBe(true);
    expect(future.startsWith('M ')).toBe(true);
    expect(future.endsWith('Z')).toBe(true);
  });

  it('returns empty string for zero-length past segment', () => {
    const layers = buildStackedLayers(sample);
    const layout = buildChartLayout(sample);
    const { past } = splitAreaPaths(layers[0], layout, 0);
    expect(past).toBe('');
  });
});

describe('splitExtendedPaths', () => {
  it('produces four non-empty path strings when asOfIndex > 0 and < n-1', () => {
    const ext = buildExtendedLayers(sample);
    const layout = buildChartLayout(sample, true);
    const paths = splitExtendedPaths(ext[0], layout, 1);
    expect(paths.principalPast.endsWith('Z')).toBe(true);
    expect(paths.principalFuture.endsWith('Z')).toBe(true);
    expect(paths.interestPast.endsWith('Z')).toBe(true);
    expect(paths.interestFuture.endsWith('Z')).toBe(true);
  });
});

describe('buildChartLayout with interest', () => {
  it('yMax is at least as large with interest as without', () => {
    const withoutInterest = buildChartLayout(sample, false);
    const withInterest = buildChartLayout(sample, true);
    // Interest can only increase or maintain yMax, never reduce it
    expect(withInterest.yMax).toBeGreaterThanOrEqual(withoutInterest.yMax);
  });

  it('yMax grows when interest is large enough to cross a tick boundary', () => {
    // 100k principal + 60k interest = 160k, which forces a new tick beyond 100k
    const bigInterest: PayoffTimelineData = {
      ...sample,
      series: [
        { ...sample.series[0], interest_remaining_minor: [60_000_00, 50_000_00, 40_000_00, 0] },
        sample.series[1],
      ],
    };
    const withoutInterest = buildChartLayout(bigInterest, false);
    const withInterest = buildChartLayout(bigInterest, true);
    expect(withInterest.yMax).toBeGreaterThan(withoutInterest.yMax);
  });
});

describe('closestDataIndex', () => {
  it('returns 0 for x before first point', () => {
    const layout = buildChartLayout(sample);
    expect(closestDataIndex(0, layout)).toBe(0);
  });

  it('returns last index for x after last point', () => {
    const layout = buildChartLayout(sample);
    expect(closestDataIndex(10_000, layout)).toBe(sample.dates.length - 1);
  });
});

describe('downsampleTimeline', () => {
  it('preserves interest_remaining_minor when downsampling', () => {
    const long: PayoffTimelineData = {
      dates: Array.from({ length: 200 }, (_, i) => `2025-${String((i % 12) + 1).padStart(2, '0')}-01`),
      as_of_index: 5,
      series: [{
        id: '1',
        label: 'A',
        balances_minor: Array.from({ length: 200 }, (_, i) => (200 - i) * 1000),
        interest_remaining_minor: Array.from({ length: 200 }, (_, i) => (200 - i) * 500),
      }],
    };
    const out = downsampleTimeline(long, 72);
    expect(out.dates.length).toBeLessThanOrEqual(72);
    expect(out.series[0].interest_remaining_minor.length).toBe(out.dates.length);
    // Last point preserved
    expect(out.dates[out.dates.length - 1]).toBe(long.dates[199]);
  });
});

describe('formatAxisMoney', () => {
  it('uses compact notation above 10k', () => {
    const result = formatAxisMoney(10_000_00, 'EUR', 'de');
    expect(result.length).toBeLessThan(12);
  });

  it('returns zero label for 0', () => {
    expect(formatAxisMoney(0, 'EUR', 'de')).toBe('0');
  });
});

describe('formatMonthLabel', () => {
  it('formats german date correctly', () => {
    const result = formatMonthLabel('2026-05-01', 'de');
    expect(result).toContain('26');
  });
});
