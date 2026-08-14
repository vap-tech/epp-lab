import { useQuery } from "@tanstack/react-query"

import { getHealth, getInfo, getRegistrars } from "@/api/admin"

export function useHealth() {
  return useQuery({ queryKey: ["admin", "health"], queryFn: getHealth })
}

export function useInfo() {
  return useQuery({ queryKey: ["admin", "info"], queryFn: getInfo })
}

export function useRegistrars() {
  return useQuery({ queryKey: ["admin", "registrars"], queryFn: getRegistrars })
}
