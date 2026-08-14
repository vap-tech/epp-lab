import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"

import {
  createZone,
  getZone,
  getZones,
  updateContactPolicy,
  updateZoneStatus,
} from "@/api/zones"

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

export function useZone(id: string) {
  return useQuery({ queryKey: ["zone", id], queryFn: () => getZone(id) })
}

export function useUpdateZone(id: string) {
  const queryClient = useQueryClient()
  const invalidate = () => {
    queryClient.invalidateQueries({ queryKey: ["zone", id] })
    queryClient.invalidateQueries({ queryKey: ["zones"] })
  }
  return {
    status: useMutation({
      mutationFn: (status: "active" | "disabled") =>
        updateZoneStatus(id, status),
      onSuccess: invalidate,
    }),
    contactPolicy: useMutation({
      mutationFn: (policy: Parameters<typeof updateContactPolicy>[1]) =>
        updateContactPolicy(id, policy),
      onSuccess: invalidate,
    }),
  }
}
