import { useQuery } from "@tanstack/react-query"

import {
  getSession,
  getSessions,
  getTransaction,
  getTransactions,
} from "@/api/epp"

export const eppKeys = {
  sessions: (page: number) => ["epp", "sessions", page],
  transactions: (page: number) => ["epp", "transactions", page],
}
export function useEppSessions(
  page = 1,
  filters: { state?: string; remote_addr?: string } = {},
) {
  return useQuery({
    queryKey: [...eppKeys.sessions(page), filters],
    queryFn: () => getSessions(page, filters),
  })
}
export function useEppTransactions(
  page = 1,
  filters: { command?: string; delivery_status?: string; trid?: string } = {},
) {
  return useQuery({
    queryKey: [...eppKeys.transactions(page), filters],
    queryFn: () => getTransactions(page, filters),
  })
}
export function useEppSession(id: string) {
  return useQuery({
    queryKey: ["epp", "session", id],
    queryFn: () => getSession(id),
  })
}
export function useEppTransaction(id: string) {
  return useQuery({
    queryKey: ["epp", "transaction", id],
    queryFn: () => getTransaction(id),
  })
}
