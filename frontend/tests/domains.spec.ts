import { expect, test } from "@playwright/test"

const domain = {
  id: "domain-id", domain_name: "example.com", roid: "D123-EXAMPLE",
  zone: { id: "zone-id", name: "com" },
  registrar: { id: "registrar-id", handle: "demo" },
  statuses: ["ok"], expires_at: "2027-08-16T00:00:00Z",
  created_at: "2026-08-16T00:00:00Z", updated_at: null,
}

test("lists domains and opens the domain detail view", async ({ page }) => {
  await page.route("**/api/auth/session", async (route) => route.fulfill({ status: 200, contentType: "application/json", body: JSON.stringify({ authenticated: true, user: { id: "admin-id", username: "admin" }, csrf_token: "csrf-token" }) }))
  await page.route("**/api/domains?*", async (route) => route.fulfill({ status: 200, contentType: "application/json", body: JSON.stringify({ items: [domain], page: 1, page_size: 50, total: 1, total_pages: 1 }) }))
  await page.route("**/api/domains/domain-id", async (route) => route.fulfill({ status: 200, contentType: "application/json", body: JSON.stringify({ ...domain, nameservers: ["ns1.external.example"], contacts: [{ role: "registrant", contact_id: "contact-id", effective: true }] }) }))

  await page.goto("/domains")
  await expect(page.getByRole("heading", { name: "Domains" })).toBeVisible()
  await expect(page.getByRole("link", { name: "example.com" })).toBeVisible()
  await page.getByRole("link", { name: "example.com" }).click()
  await expect(page).toHaveURL(/\/domains\/domain-id$/)
  await expect(page.getByText("ns1.external.example")).toBeVisible()
  await expect(page.getByText("registrant")).toBeVisible()
})
