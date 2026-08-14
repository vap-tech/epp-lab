import { createFileRoute } from "@tanstack/react-router"

export const Route = createFileRoute("/_layout/sessions")({
  component: Sessions,
  head: () => ({ meta: [{ title: "EPP Sessions - EPP Lab" }] }),
})

function Sessions() {
  return (
    <section className="flex flex-col gap-2">
      <h1 className="text-2xl font-bold tracking-tight">EPP Sessions</h1>
      <p className="text-muted-foreground">
        Session views are not available yet.
      </p>
    </section>
  )
}
