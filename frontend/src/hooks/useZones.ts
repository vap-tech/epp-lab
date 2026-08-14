import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"

import { createZone, getZones } from "@/api/zones"

export function useZones() {
  return useQuery({ queryKey: ["zones"], queryFn: getZones })
}

export function useCreateZone() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: createZone,
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ["zones"] }),
  })
}
