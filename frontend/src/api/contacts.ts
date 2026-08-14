import { z } from "zod"

import { api } from "./http"

const contactSchema = z.object({
  id: z.string(),
  contact_id: z.string(),
  roid: z.string(),
  registrar_id: z.string(),
  registrar_handle: z.string().nullable(),
  email: z.string(),
  statuses: z.array(z.string()),
  linked: z.boolean(),
  created_at: z.string(),
  updated_at: z.string(),
  name: z.string().optional(),
  organization: z.string().nullable().optional(),
  streets: z.array(z.string()).optional(),
  city: z.string().optional(),
  state_province: z.string().nullable().optional(),
  postal_code: z.string().nullable().optional(),
  country_code: z.string().optional(),
  voice: z.string().optional(),
  voice_extension: z.string().nullable().optional(),
  fax: z.string().nullable().optional(),
  fax_extension: z.string().nullable().optional(),
  disclose_flag: z.string().optional(),
  disclosure_fields: z.array(z.string()).optional(),
  localized_postal_info: z
    .object({
      name: z.string(),
      organization: z.string().nullable(),
      streets: z.array(z.string()),
      city: z.string(),
      state_province: z.string().nullable(),
      postal_code: z.string().nullable(),
      country_code: z.string(),
    })
    .nullable()
    .optional(),
})

export type Contact = z.infer<typeof contactSchema>

const contactPageSchema = z.object({
  items: z.array(contactSchema),
  page: z.number(),
  page_size: z.number(),
  total: z.number(),
  total_pages: z.number(),
})

export type ContactFilters = {
  registrar_id?: string
  status?: string
  search?: string
}

export async function getContacts(page = 1, filters: ContactFilters = {}) {
  const params = new URLSearchParams({ page: String(page), page_size: "50" })
  for (const [key, value] of Object.entries(filters)) {
    if (value) params.set(key, value)
  }
  return contactPageSchema.parse(
    await api.get(`/contacts?${params.toString()}`),
  )
}

export async function getContact(id: string) {
  return contactSchema.parse(await api.get<Contact>(`/contacts/${id}`))
}
