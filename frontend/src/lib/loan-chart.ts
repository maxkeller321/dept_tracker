export const CHART_COLORS = [
  '#6366f1',
  '#eab308',
  '#0d9488',
  '#ec4899',
  '#0f4c5c',
  '#f97316',
  '#8b5cf6',
  '#14b8a6',
] as const;

export interface LoanChartInput {
  id: string;
  label: string;
  amountMinor: number;
}

export interface LoanChartSlice extends LoanChartInput {
  color: string;
  percent: number;
  startAngle: number;
  endAngle: number;
}

function polar(cx: number, cy: number, radius: number, angleDeg: number): [number, number] {
  const rad = (angleDeg * Math.PI) / 180;
  return [cx + radius * Math.cos(rad), cy + radius * Math.sin(rad)];
}

/** SVG path for one donut segment (angles in degrees, 0° = east, sweep clockwise). */
export function describeDonutSegment(
  cx: number,
  cy: number,
  outerR: number,
  innerR: number,
  startAngle: number,
  endAngle: number,
): string {
  const sweep = endAngle - startAngle;
  if (sweep >= 359.999) {
    const mid = startAngle + 180;
    return (
      describeDonutSegment(cx, cy, outerR, innerR, startAngle, mid) +
      describeDonutSegment(cx, cy, outerR, innerR, mid, endAngle)
    );
  }
  if (sweep <= 0) return '';

  const largeArc = sweep > 180 ? 1 : 0;
  const [x1, y1] = polar(cx, cy, outerR, startAngle);
  const [x2, y2] = polar(cx, cy, outerR, endAngle);
  const [x3, y3] = polar(cx, cy, innerR, endAngle);
  const [x4, y4] = polar(cx, cy, innerR, startAngle);

  return [
    `M ${x1.toFixed(3)} ${y1.toFixed(3)}`,
    `A ${outerR} ${outerR} 0 ${largeArc} 1 ${x2.toFixed(3)} ${y2.toFixed(3)}`,
    `L ${x3.toFixed(3)} ${y3.toFixed(3)}`,
    `A ${innerR} ${innerR} 0 ${largeArc} 0 ${x4.toFixed(3)} ${y4.toFixed(3)}`,
    'Z',
  ].join(' ');
}

export function buildLoanChartSlices(loans: LoanChartInput[]): LoanChartSlice[] {
  const positive = loans.filter((l) => l.amountMinor > 0);
  const total = positive.reduce((sum, l) => sum + l.amountMinor, 0);
  if (total <= 0 || positive.length === 0) return [];

  let angle = -90;
  return positive.map((loan, index) => {
    const sweep = (loan.amountMinor / total) * 360;
    const startAngle = angle;
    const endAngle = angle + sweep;
    angle = endAngle;
    return {
      ...loan,
      color: CHART_COLORS[index % CHART_COLORS.length],
      percent: (loan.amountMinor / total) * 100,
      startAngle,
      endAngle,
    };
  });
}
