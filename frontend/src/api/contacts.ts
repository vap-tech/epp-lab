import { z } from "zod"

import { api } from "./http"

const contactSchema = z.object({
  id: z.string(),
  contact_id: z.string(),
  roid: z.string(),
  registrar_id: z.string(),
  email: z.string(),
  statuses: z.array(z.string()),
  linked: z.boolean(),
  created_at: z.string(),
  updated_at: z.string(),
})

export type Contact = z.infer<typeof contactSchema>

export async function getContacts() {
  return z.array(contactSchema).parse(await api.get<Contact[]>("/contacts"))
}

export async function getContact(id: string) {
  return contactSchema.parse(await api.get<Contact>(`/contacts/${id}`))
}
