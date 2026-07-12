export type SonderzahlungPlanAction = 'schedule' | 'backdated';

/** Compare ISO calendar dates (YYYY-MM-DD). */
export function compareIsoDates(a: string, b: string): number {
  return a.localeCompare(b);
}

/** Route a plan-row submission: past/today → backdated immediate, future → schedule. */
export function resolvePlanAction(planDate: string, today: string): SonderzahlungPlanAction {
  return compareIsoDates(planDate, today) <= 0 ? 'backdated' : 'schedule';
}
