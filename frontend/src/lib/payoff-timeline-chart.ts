import { CHART_COLORS } from './loan-chart';

export interface PayoffTimelineData {
  dates: string[];
  series: {
    id: string;
    label: string;
    balances_minor: number[];
    interest_remaining_minor: number[];
  }[];
  as_of_index: number;
}

export interface StackedLayer {
  id: string;
  label: string;
  color: string;
  lower: number[];
  upper: number[];
}

/** One loan rendered as two stacked sub-layers: principal (solid) + interest cap (striped). */
export interface ExtendedStackedLayer {
  id: string;
  label: string;
  color: string;
  /** Bottom of the principal fill for this loan. */
  principalLower: number[];
  /** Top of the principal fill = bottom of interest cap. */
  principalUpper: number[];
  /** Top of the interest cap fill. */
  interestUpper: number[];
}

export interface ChartLayout {
  width: number;
  height: number;
  padLeft: number;
  padRight: number;
  padTop: number;
  padBottom: number;
  plotWidth: number;
  plotHeight: number;
  xs: number[];
  yMax: number;
  yTicks: number[];
  xLabelIndices: number[];
}

const DEFAULT_LAYOUT = {
  width: 660,
  height: 260,
  padLeft: 76,
  padRight: 12,
  padTop: 24,   // extra room so top tick label never overlaps chart fills
  padBottom: 36,
} as const;

export function downsampleTimeline(timeline: PayoffTimelineData, maxPoints = 72): PayoffTimelineData {
  if (timeline.dates.length <= maxPoints) return timeline;
  const step = Math.ceil(timeline.dates.length / maxPoints);
  const pick = (idx: number) => idx % step === 0 || idx === timeline.dates.length - 1;
  const indices = timeline.dates.map((_, i) => i).filter(pick);
  const ai = timeline.as_of_index;
  const best = indices.reduce(
    (b, idx, pos) => (Math.abs(idx - ai) < Math.abs(indices[b] - ai) ? pos : b),
    0,
  );
  return {
    dates: indices.map((i) => timeline.dates[i]),
    as_of_index: best,
    series: timeline.series.map((s) => ({
      ...s,
      balances_minor: indices.map((i) => s.balances_minor[i]),
      interest_remaining_minor: indices.map((i) => (s.interest_remaining_minor?.[i] ?? 0)),
    })),
  };
}

/** Principal-only stacked layers (used when interest toggle is off). */
export function buildStackedLayers(timeline: PayoffTimelineData): StackedLayer[] {
  const { series } = timeline;
  return series.map((s, i) => {
    const lower: number[] = [];
    const upper: number[] = [];
    for (let t = 0; t < timeline.dates.length; t++) {
      let below = 0;
      for (let j = 0; j < i; j++) below += series[j].balances_minor[t] ?? 0;
      const value = s.balances_minor[t] ?? 0;
      lower.push(below);
      upper.push(below + value);
    }
    return { id: s.id, label: s.label, color: CHART_COLORS[i % CHART_COLORS.length], lower, upper };
  });
}

/** Principal + interest stacked layers (used when interest toggle is on). */
export function buildExtendedLayers(timeline: PayoffTimelineData): ExtendedStackedLayer[] {
  const { series } = timeline;
  return series.map((s, i) => {
    const principalLower: number[] = [];
    const principalUpper: number[] = [];
    const interestUpper: number[] = [];
    for (let t = 0; t < timeline.dates.length; t++) {
      // The offset below this loan is the sum of (principal + interest) of all previous loans
      let below = 0;
      for (let j = 0; j < i; j++) {
        below += (series[j].balances_minor[t] ?? 0) + (series[j].interest_remaining_minor?.[t] ?? 0);
      }
      const principal = s.balances_minor[t] ?? 0;
      const interest = s.interest_remaining_minor?.[t] ?? 0;
      principalLower.push(below);
      principalUpper.push(below + principal);
      interestUpper.push(below + principal + interest);
    }
    return {
      id: s.id,
      label: s.label,
      color: CHART_COLORS[i % CHART_COLORS.length],
      principalLower,
      principalUpper,
      interestUpper,
    };
  });
}

/** Compute yMax across all time points, optionally including interest. */
function computeYMax(timeline: PayoffTimelineData, includeInterest: boolean): number {
  let yMax = 0;
  for (let t = 0; t < timeline.dates.length; t++) {
    let total = 0;
    for (const s of timeline.series) {
      total += s.balances_minor[t] ?? 0;
      if (includeInterest) total += s.interest_remaining_minor?.[t] ?? 0;
    }
    yMax = Math.max(yMax, total);
  }
  return yMax > 0 ? yMax : 1;
}

export function buildChartLayout(
  timeline: PayoffTimelineData,
  includeInterest = false,
): ChartLayout {
  const n = timeline.dates.length;
  const plotWidth = DEFAULT_LAYOUT.width - DEFAULT_LAYOUT.padLeft - DEFAULT_LAYOUT.padRight;
  const plotHeight = DEFAULT_LAYOUT.height - DEFAULT_LAYOUT.padTop - DEFAULT_LAYOUT.padBottom;

  const yDataMax = computeYMax(timeline, includeInterest);
  const tickStep = niceStep(yDataMax / 4);
  // Always keep at least one empty tick above the data so fills never touch the top border
  const niceTop = Math.ceil(yDataMax / tickStep) * tickStep;
  const yMaxRounded = niceTop > yDataMax ? niceTop : niceTop + tickStep;
  const yTicks: number[] = [];
  for (let v = 0; v <= yMaxRounded; v += tickStep) yTicks.push(v);

  const xs =
    n <= 1
      ? [DEFAULT_LAYOUT.padLeft + plotWidth / 2]
      : Array.from({ length: n }, (_, i) => DEFAULT_LAYOUT.padLeft + (i / (n - 1)) * plotWidth);

  const xLabelCount = Math.min(6, n);
  const xLabelIndices =
    n <= 1
      ? [0]
      : Array.from({ length: xLabelCount }, (_, i) => Math.round((i / (xLabelCount - 1)) * (n - 1)));

  return {
    ...DEFAULT_LAYOUT,
    plotWidth,
    plotHeight,
    xs,
    yMax: yMaxRounded,
    yTicks,
    xLabelIndices,
  };
}

function niceStep(raw: number): number {
  if (raw <= 0) return 1;
  const mag = 10 ** Math.floor(Math.log10(raw));
  const norm = raw / mag;
  return (norm <= 1 ? 1 : norm <= 2 ? 2 : norm <= 5 ? 5 : 10) * mag;
}

export function yScale(value: number, layout: ChartLayout): number {
  return layout.padTop + layout.plotHeight - (value / layout.yMax) * layout.plotHeight;
}

function areaSegment(
  lower: number[],
  upper: number[],
  layout: ChartLayout,
  fromIdx: number,
  toIdx: number,
): string {
  const { xs } = layout;
  if (fromIdx >= toIdx) return '';
  const pts = Array.from({ length: toIdx - fromIdx + 1 }, (_, k) => fromIdx + k);
  const top = pts
    .map((i, k) => `${k === 0 ? 'M' : 'L'} ${xs[i].toFixed(2)} ${yScale(upper[i], layout).toFixed(2)}`)
    .join(' ');
  const bottom = pts
    .slice()
    .reverse()
    .map((i) => `L ${xs[i].toFixed(2)} ${yScale(lower[i], layout).toFixed(2)}`)
    .join(' ');
  return `${top} ${bottom} Z`;
}

/** Returns {past, future} SVG path strings for a principal-only layer, split at asOfIndex. */
export function splitAreaPaths(
  layer: StackedLayer,
  layout: ChartLayout,
  asOfIndex: number,
): { past: string; future: string } {
  const n = layout.xs.length;
  const ai = Math.max(0, Math.min(asOfIndex, n - 1));
  return {
    past: areaSegment(layer.lower, layer.upper, layout, 0, ai),
    future: areaSegment(layer.lower, layer.upper, layout, ai, n - 1),
  };
}

/** Returns {principalPast, principalFuture, interestPast, interestFuture} paths for extended layers. */
export function splitExtendedPaths(
  layer: ExtendedStackedLayer,
  layout: ChartLayout,
  asOfIndex: number,
): { principalPast: string; principalFuture: string; interestPast: string; interestFuture: string } {
  const n = layout.xs.length;
  const ai = Math.max(0, Math.min(asOfIndex, n - 1));
  return {
    principalPast: areaSegment(layer.principalLower, layer.principalUpper, layout, 0, ai),
    principalFuture: areaSegment(layer.principalLower, layer.principalUpper, layout, ai, n - 1),
    interestPast: areaSegment(layer.principalUpper, layer.interestUpper, layout, 0, ai),
    interestFuture: areaSegment(layer.principalUpper, layer.interestUpper, layout, ai, n - 1),
  };
}

/** Returns the data index closest to the given SVG x coordinate. */
export function closestDataIndex(svgX: number, layout: ChartLayout): number {
  const { xs } = layout;
  if (xs.length === 0) return 0;
  let best = 0;
  let bestDist = Math.abs(svgX - xs[0]);
  for (let i = 1; i < xs.length; i++) {
    const d = Math.abs(svgX - xs[i]);
    if (d < bestDist) { best = i; bestDist = d; }
  }
  return best;
}

export function formatMonthLabel(isoDate: string, locale: string): string {
  const [y, m] = isoDate.split('-').map(Number);
  if (!y || !m) return isoDate;
  return new Intl.DateTimeFormat(locale === 'de' ? 'de-DE' : locale, {
    month: 'short',
    year: '2-digit',
  }).format(new Date(y, m - 1, 1));
}

export function formatAxisMoney(amountMinor: number, currency: string, locale: string): string {
  if (amountMinor === 0) return '0';
  const value = amountMinor / 100;
  const tag = locale === 'de' ? 'de-DE' : locale;
  // Get the currency symbol in a locale-aware way
  const sym =
    new Intl.NumberFormat(tag, { style: 'currency', currency, maximumFractionDigits: 0 })
      .formatToParts(1)
      .find((p) => p.type === 'currency')?.value ?? currency;
  // Manual K/M abbreviation — reliably short regardless of browser compact-notation support
  if (value >= 1_000_000) {
    const n = new Intl.NumberFormat(tag, { maximumFractionDigits: 1 }).format(value / 1_000_000);
    return `${n} M ${sym}`;
  }
  if (value >= 1_000) {
    const n = new Intl.NumberFormat(tag, { maximumFractionDigits: 0 }).format(value / 1_000);
    return `${n} K ${sym}`;
  }
  return new Intl.NumberFormat(tag, { maximumFractionDigits: 0 }).format(value) + '\u202f' + sym;
}
