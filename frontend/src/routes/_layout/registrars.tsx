import { createFileRoute } from "@tanstack/react-router"

export const Route = createFileRoute("/_layout/registrars")({
  component: Registrars,
  head: () => ({ meta: [{ title: "Registrars - EPP Lab" }] }),
})

function Registrars() {
  return (
    <section className="flex flex-col gap-2">
      <h1 className="text-2xl font-bold tracking-tight">Registrars</h1>
      <p className="text-muted-foreground">
        Registrar management will use the Admin API in the next iteration.
      </p>
    </section>
  )
}
