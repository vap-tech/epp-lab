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
const extensionSchema = z.object({
  key: z.string(),
  display_name: z.string(),
  namespace_uri: z.string(),
})
const zoneExtensionSchema = z.object({
  zone_id: z.string(),
  extension_key: z.string(),
  enabled: z.boolean(),
})

export type Zone = z.infer<typeof zoneSchema>
export type Extension = z.infer<typeof extensionSchema>
export type ZoneExtension = z.infer<typeof zoneExtensionSchema>

export async function getZones() {
  return z.array(zoneSchema).parse(await api.get<Zone[]>("/zones"))
}

export async function createZone(name: string) {
  return zoneSchema.parse(await api.post<Zone>("/zones", { name }))
}

export async function getZone(id: string) {
  return zoneSchema.parse(await api.get<Zone>(`/zones/${id}`))
}

export async function updateZoneStatus(id: string, status: Zone["status"]) {
  return zoneSchema.parse(await api.patch<Zone>(`/zones/${id}`, { status }))
}

export async function updateContactPolicy(
  id: string,
  policy: Zone["contact_policy"],
) {
  return zoneSchema.parse(
    await api.patch<Zone>(`/zones/${id}/contact-policy`, policy),
  )
}

export async function getExtensionCatalog() {
  return z
    .array(extensionSchema)
    .parse(await api.get<Extension[]>("/extensions/catalog"))
}

export async function getZoneExtensions(zoneId: string) {
  return z
    .array(zoneExtensionSchema)
    .parse(await api.get<ZoneExtension[]>(`/zones/${zoneId}/extensions`))
}

export async function setZoneExtension(
  zoneId: string,
  key: string,
  enabled: boolean,
) {
  return zoneExtensionSchema.parse(
    await api.patch<ZoneExtension>(`/zones/${zoneId}/extensions/${key}`, {
      enabled,
    }),
  )
}
