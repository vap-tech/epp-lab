import { useQuery } from "@tanstack/react-query"

import { getContact, getContacts, type ContactFilters } from "@/api/contacts"

export const contactKeys = {
  all: ["contacts"] as const,
  list: (page: number, filters: ContactFilters) =>
    [...contactKeys.all, "list", page, filters] as const,
  detail: (id: string) => [...contactKeys.all, "detail", id] as const,
}

export function useContacts(page = 1, filters: ContactFilters = {}) {
  return useQuery({
    queryKey: contactKeys.list(page, filters),
    queryFn: () => getContacts(page, filters),
  })
}

export function useContact(id: string) {
  return useQuery({
    queryKey: contactKeys.detail(id),
    queryFn: () => getContact(id),
  })
}
