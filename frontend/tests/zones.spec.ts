import { expect, test } from "@playwright/test"

const zone = {
  id: "zone-id",
  ascii_name: "com",
  unicode_name: "com",
  status: "active",
  contact_policy: {
    registrant: "required",
    admin: "optional",
    tech: "optional",
    billing: "optional",
  },
  contactless: false,
  enabled_extensions_count: 0,
  created_at: "2026-08-14T18:00:00Z",
  updated_at: "2026-08-14T18:00:00Z",
}

test("lists zones and opens the zone detail view", async ({ page }) => {
  await page.route("**/api/auth/session", async (route) =>
    route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({
        authenticated: true,
        user: { id: "admin-id", username: "admin" },
        csrf_token: "csrf-token",
      }),
    }),
  )
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
  await page.route("**/api/zones", async (route) =>
    route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify([zone]),
    }),
  )
  await page.route("**/api/zones/zone-id", async (route) =>
    route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify(zone),
    }),
  )
  await page.route("**/api/extensions/catalog", async (route) =>
    route.fulfill({ status: 200, contentType: "application/json", body: "[]" }),
  )
  await page.route("**/api/zones/zone-id/extensions", async (route) =>
    route.fulfill({ status: 200, contentType: "application/json", body: "[]" }),
  )

  await page.goto("/zones")
  await expect(page.getByRole("heading", { name: "Zones" })).toBeVisible()
  await expect(page.getByRole("link", { name: "com" })).toBeVisible()

  await page.getByRole("link", { name: "com" }).click()
  await expect(page).toHaveURL(/\/zones\/zone-id$/)
  await expect(page.getByRole("heading", { name: "com" })).toBeVisible()
  await expect(
    page.getByRole("heading", { name: "Contact Usage" }),
  ).toBeVisible()
  await expect(
    page.getByText("No extensions are registered in this server."),
  ).toBeVisible()
})
