import { z } from "zod"
import { api, setCsrfToken } from "./http"

const sessionSchema = z.object({
  authenticated: z.boolean(),
  user: z.object({ id: z.string(), username: z.string() }).nullable(),
  csrf_token: z.string().nullable(),
})

export type AuthSession = z.infer<typeof sessionSchema>

export async function getSession() {
  const session = sessionSchema.parse(
    await api.get<AuthSession>("/auth/session"),
  )
  setCsrfToken(session.csrf_token)
  return session
}

export async function login(username: string, password: string) {
  const session = sessionSchema.parse(
    await api.post<AuthSession>("/auth/login", { username, password }),
  )
  setCsrfToken(session.csrf_token)
  return session
}

export async function logout() {
  await api.post<void>("/auth/logout", undefined)
  setCsrfToken(null)
}
