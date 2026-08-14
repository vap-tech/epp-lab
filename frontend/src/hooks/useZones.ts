import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"

import {
  createZone,
  getExtensionCatalog,
  getZone,
  getZoneExtensions,
  getZones,
  setZoneExtension,
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

export function useExtensionCatalog() {
  return useQuery({
    queryKey: ["extensions", "catalog"],
    queryFn: getExtensionCatalog,
  })
}

export function useZoneExtensions(id: string) {
  return useQuery({
    queryKey: ["zone", id, "extensions"],
    queryFn: () => getZoneExtensions(id),
  })
}

export function useSetZoneExtension(id: string) {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: ({ key, enabled }: { key: string; enabled: boolean }) =>
      setZoneExtension(id, key, enabled),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["zone", id, "extensions"] })
      queryClient.invalidateQueries({ queryKey: ["zone", id] })
      queryClient.invalidateQueries({ queryKey: ["zones"] })
    },
  })
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
