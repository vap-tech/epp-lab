import { useQuery } from "@tanstack/react-query"
import { getDomain, getDomains } from "@/api/domains"

export function useDomains(page = 1, search = "") {
  return useQuery({ queryKey: ["domains", page, search], queryFn: () => getDomains(page, search) })
}

export function useDomain(id: string) {
  return useQuery({ queryKey: ["domain", id], queryFn: () => getDomain(id) })
}
