import { useQuery } from "@tanstack/react-query"

import { getSessions, getTransactions } from "@/api/epp"

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
