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
export function useEppSessions(page = 1) {
  return useQuery({
    queryKey: eppKeys.sessions(page),
    queryFn: () => getSessions(page),
  })
}
export function useEppTransactions(page = 1) {
  return useQuery({
    queryKey: eppKeys.transactions(page),
    queryFn: () => getTransactions(page),
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
