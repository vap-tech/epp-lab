import { z } from "zod"
import { api } from "./http"

const summarySchema = z.object({
  id: z.string(), domain_name: z.string(), roid: z.string(),
  zone: z.object({ id: z.string(), name: z.string() }),
  registrar: z.object({ id: z.string(), handle: z.string() }),
  statuses: z.array(z.string()), expires_at: z.string(), created_at: z.string(), updated_at: z.string().nullable(),
})
const detailSchema = summarySchema.extend({
  nameservers: z.array(z.string()),
  contacts: z.array(z.object({ role: z.string(), contact_id: z.string(), effective: z.boolean() })),
})
const pageSchema = z.object({ items: z.array(summarySchema), page: z.number(), page_size: z.number(), total: z.number(), total_pages: z.number() })
export type DomainSummary = z.infer<typeof summarySchema>
export type DomainDetail = z.infer<typeof detailSchema>

export async function getDomains(page = 1, search?: string) {
  const params = new URLSearchParams({ page: String(page), page_size: "50" })
  if (search) params.set("search", search)
  return pageSchema.parse(await api.get(`/domains?${params.toString()}`))
}

export async function getDomain(id: string) {
  return detailSchema.parse(await api.get(`/domains/${id}`))
}
