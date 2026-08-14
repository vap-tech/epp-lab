import { z } from "zod"

import { api } from "./http"

const healthSchema = z.object({
  status: z.string(),
  database: z.string(),
})

const infoSchema = z.object({
  name: z.string(),
  version: z.string(),
  epp_bind: z.string(),
  environment: z.string(),
})

const registrarSchema = z.object({
  id: z.string(),
  handle: z.string(),
  name: z.string(),
  client_id: z.string(),
  status: z.string(),
})

export type Health = z.infer<typeof healthSchema>
export type Info = z.infer<typeof infoSchema>
export type Registrar = z.infer<typeof registrarSchema>

export async function getHealth() {
  return healthSchema.parse(await api.get<Health>("/health"))
}

export async function getInfo() {
  return infoSchema.parse(await api.get<Info>("/info"))
}

export async function getRegistrars() {
  return z
    .array(registrarSchema)
    .parse(await api.get<Registrar[]>("/registrars"))
}
