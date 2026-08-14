import { useQuery } from "@tanstack/react-query"

import { getContact, getContacts } from "@/api/contacts"

export function useContacts() {
  return useQuery({ queryKey: ["contacts"], queryFn: getContacts })
}

export function useContact(id: string) {
  return useQuery({ queryKey: ["contact", id], queryFn: () => getContact(id) })
}
