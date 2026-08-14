import { z } from "zod"

import { api } from "./http"

const contactPolicySchema = z.object({
  registrant: z.enum(["forbidden", "optional", "required"]),
  admin: z.enum(["forbidden", "optional", "required"]),
  tech: z.enum(["forbidden", "optional", "required"]),
  billing: z.enum(["forbidden", "optional", "required"]),
})
const zoneSchema = z.object({
  id: z.string(),
  ascii_name: z.string(),
  unicode_name: z.string(),
  status: z.enum(["active", "disabled"]),
  contact_policy: contactPolicySchema,
  contactless: z.boolean(),
  enabled_extensions_count: z.number(),
  created_at: z.string(),
  updated_at: z.string(),
})

export type Zone = z.infer<typeof zoneSchema>

export async function getZones() {
  return z.array(zoneSchema).parse(await api.get<Zone[]>("/zones"))
}

export async function createZone(name: string) {
  return zoneSchema.parse(await api.post<Zone>("/zones", { name }))
}
