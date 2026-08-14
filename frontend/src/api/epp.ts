import { z } from "zod"

import { api } from "./http"

const registrarSchema = z.object({
  id: z.string(),
  handle: z.string(),
  name: z.string(),
})
const certificateSchema = z.object({
  id: z.string(),
  fingerprint_sha256: z.string(),
})
const sessionSchema = z.object({
  id: z.string(),
  registrar: registrarSchema.nullable(),
  certificate: certificateSchema.nullable(),
  remote_addr: z.string(),
  connected_at: z.string(),
  authenticated_at: z.string().nullable(),
  disconnected_at: z.string().nullable(),
  disconnect_reason: z.string().nullable(),
  transaction_count: z.number(),
})
const transactionSchema = z.object({
  id: z.string(),
  session_id: z.string(),
  registrar: registrarSchema.nullable(),
  command: z.string(),
  object_name: z.string().nullable(),
  cl_trid: z.string().nullable(),
  sv_trid: z.string(),
  request_xml: z.string().nullable(),
  response_xml: z.string().nullable(),
  response_code: z.number().nullable(),
  delivery_status: z.enum(["delivered", "failed", "unknown"]),
  delivery_error: z.string().nullable(),
  started_at: z.string(),
  finished_at: z.string().nullable(),
  duration_ms: z.number().nullable(),
})
const page = <T extends z.ZodType>(item: T) =>
  z.object({
    items: z.array(item),
    page: z.number(),
    page_size: z.number(),
    total: z.number(),
    total_pages: z.number(),
  })
export type EppSession = z.infer<typeof sessionSchema>
export type EppTransaction = z.infer<typeof transactionSchema>
export type Page<T> = {
  items: T[]
  page: number
  page_size: number
  total: number
  total_pages: number
}

export async function getSessions(pageNumber = 1) {
  return page(sessionSchema).parse(
    await api.get<Page<EppSession>>(
      `/epp/sessions?page=${pageNumber}&page_size=50`,
    ),
  )
}
export async function getTransactions(pageNumber = 1) {
  return page(transactionSchema).parse(
    await api.get<Page<EppTransaction>>(
      `/epp/transactions?page=${pageNumber}&page_size=50`,
    ),
  )
}
export async function getSession(id: string) {
  return sessionSchema.parse(await api.get<EppSession>(`/epp/sessions/${id}`))
}
export async function getTransaction(id: string) {
  return transactionSchema.parse(
    await api.get<EppTransaction>(`/epp/transactions/${id}`),
  )
}
