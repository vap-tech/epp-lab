export class ApiError extends Error {
  constructor(
    message: string,
    readonly status: number,
  ) {
    super(message)
  }
}

let csrfToken: string | null = null

export function setCsrfToken(token: string | null) {
  csrfToken = token
}

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const headers = new Headers(init?.headers)
  headers.set("Content-Type", "application/json")
  if (init?.method && init.method !== "GET" && csrfToken) {
    headers.set("X-CSRF-Token", csrfToken)
  }
  const response = await fetch(`/api${path}`, {
    ...init,
    credentials: "same-origin",
    headers,
  })
  if (!response.ok) {
    throw new ApiError("Request failed", response.status)
  }
  return (await response.json()) as T
}

export const api = {
  get: <T>(path: string) => request<T>(path),
  post: <T>(path: string, body: unknown) =>
    request<T>(path, { method: "POST", body: JSON.stringify(body) }),
}
