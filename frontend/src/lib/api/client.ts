const API_BASE = import.meta.env.VITE_API_BASE ?? '/api/v1';

export interface Money {
  amount_minor: number;
  currency: string;
}

export interface LoanSummary {
  id: string;
  label: string;
  remaining_balance: Money;
  /** Original loan amount at origination; null if not recorded. */
  original_principal: Money | null;
  periodic_payment: Money;
  payment_frequency: string;
  last_payment_date: string | null;
  projected_payoff_date: string | null;
  progress_percent: number;
}

export interface PayoffTimelineSeries {
  id: string;
  label: string;
  balances_minor: number[];
  /** Total future interest still to be paid at each timeline point. */
  interest_remaining_minor: number[];
}

export interface PayoffTimeline {
  dates: string[];
  series: PayoffTimelineSeries[];
  as_of_index: number;
}

export interface DashboardResponse {
  household: {
    total_balance: Money;
    total_monthly_obligation: Money;
  };
  loans: LoanSummary[];
  payoff_timeline: PayoffTimeline;
}

export interface AmortizationRow {
  date: string;
  payment_minor: number;
  interest_minor: number;
  principal_minor: number;
  balance_minor: number;
}

export interface AmortizationSchedule {
  total_payments: number;
  rows: AmortizationRow[];
}

export interface AuthStatus {
  auth_enabled: boolean;
  needs_setup: boolean;
  authenticated: boolean;
}

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const res = await fetch(`${API_BASE}${path}`, {
    credentials: 'include',
    headers: { 'Content-Type': 'application/json', ...init?.headers },
    ...init,
  });
  if (!res.ok) {
    const err = await res.json().catch(() => ({ error: res.statusText }));
    throw new Error((err as { error?: string }).error ?? res.statusText);
  }
  if (res.status === 204) return undefined as T;
  return res.json() as Promise<T>;
}

export const api = {
  authStatus: () => request<AuthStatus>('/auth/status'),

  register: (username: string, password: string) =>
    request<{ ok: boolean }>('/auth/register', {
      method: 'POST',
      body: JSON.stringify({ username, password }),
    }),

  login: (username: string, password: string) =>
    request<{ ok: boolean }>('/auth/login', {
      method: 'POST',
      body: JSON.stringify({ username, password }),
    }),

  logout: () => request<void>('/auth/logout', { method: 'POST' }),

  dashboard: (includeArchived = false) =>
    request<DashboardResponse>(`/dashboard?include_archived=${includeArchived}`),

  createLoan: (body: unknown) =>
    request<Record<string, unknown>>('/loans', { method: 'POST', body: JSON.stringify(body) }),

  loanDetail: (id: string) => request<Record<string, unknown>>(`/loans/${id}`),

  updateLoan: (id: string, body: unknown) =>
    request<Record<string, unknown>>(`/loans/${id}`, {
      method: 'PATCH',
      body: JSON.stringify(body),
    }),

  listPayments: (id: string) =>
    request<unknown[]>(`/loans/${id}/payments`),

  immediateSonderzahlung: (id: string, body: unknown) =>
    request<unknown>(`/loans/${id}/sonderzahlungen/immediate`, {
      method: 'POST',
      body: JSON.stringify(body),
    }),

  scheduleSonderzahlung: (id: string, body: unknown) =>
    request<unknown>(`/loans/${id}/sonderzahlungen/scheduled`, {
      method: 'POST',
      body: JSON.stringify(body),
    }),

  cancelScheduled: (loanId: string, scheduleId: string) =>
    request<void>(`/loans/${loanId}/sonderzahlungen/scheduled/${scheduleId}`, {
      method: 'DELETE',
    }),

  recordPayment: (id: string, body: unknown) =>
    request<unknown>(`/loans/${id}/payments`, {
      method: 'POST',
      body: JSON.stringify(body),
    }),

  exportData: () => request<unknown>('/export'),

  importData: (body: unknown) =>
    request<unknown>('/import?confirm=true', {
      method: 'POST',
      body: JSON.stringify(body),
    }),

  deleteLoan: (id: string) =>
    request<void>(`/loans/${id}?confirm=true`, { method: 'DELETE' }),

  archiveLoan: (id: string) =>
    request<unknown>(`/loans/${id}/archive`, { method: 'POST' }),

  loanAmortization: (id: string) =>
    request<AmortizationSchedule>(`/loans/${id}/amortization`),

  combinedAmortization: () =>
    request<AmortizationSchedule>('/amortization'),
};
