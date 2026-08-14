import { useQuery } from "@tanstack/react-query"

import { getZones } from "@/api/zones"

export function useZones() {
  return useQuery({ queryKey: ["zones"], queryFn: getZones })
}
