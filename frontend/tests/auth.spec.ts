import { expect, test } from "@playwright/test"

test("redirects unauthenticated users to login", async ({ page }) => {
  await page.route("**/api/auth/session", async (route) =>
    route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({
        authenticated: false,
        user: null,
        csrf_token: null,
      }),
    }),
  )

  await page.goto("/")

  await expect(page).toHaveURL(/\/login$/)
  await expect(
    page.getByRole("heading", { name: "Sign in to EPP Lab" }),
  ).toBeVisible()
})

test("logs in and logs out from the protected shell", async ({ page }) => {
  let authenticated = false
  await page.route("**/api/auth/session", async (route) =>
    route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({
        authenticated,
        user: authenticated ? { id: "admin-id", username: "admin" } : null,
        csrf_token: authenticated ? "csrf-token" : null,
      }),
    }),
  )
  await page.route("**/api/auth/login", async (route) => {
    authenticated = true
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({
        authenticated: true,
        user: { id: "admin-id", username: "admin" },
        csrf_token: "csrf-token",
      }),
    })
  })
  await page.route("**/api/auth/logout", async (route) => {
    authenticated = false
    await route.fulfill({ status: 204 })
  })
  await page.route("**/api/health", async (route) =>
    route.fulfill({
      status: 200,
      contentType: "application/json",
      body: '{"status":"ok","database":"ok"}',
    }),
  )
  await page.route("**/api/info", async (route) =>
    route.fulfill({
      status: 200,
      contentType: "application/json",
      body: '{"name":"EPP Lab","version":"test","epp_bind":"700","environment":"test"}',
    }),
  )

  await page.goto("/login")
  await page.getByLabel("Username").fill("admin")
  await page.getByLabel("Password").fill("password")
  await page.getByRole("button", { name: "Sign in" }).click()

  await expect(page).toHaveURL(/\/$/)
  await expect(page.getByRole("heading", { name: "Dashboard" })).toBeVisible()
  await page.getByRole("button", { name: "Sign out" }).click()
  await expect(page).toHaveURL(/\/login$/)
})
