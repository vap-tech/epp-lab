import { createFileRoute } from "@tanstack/react-router"

export const Route = createFileRoute("/_layout/transactions")({
  component: Transactions,
  head: () => ({ meta: [{ title: "EPP Transactions - EPP Lab" }] }),
})

function Transactions() {
  return (
    <section className="flex flex-col gap-2">
      <h1 className="text-2xl font-bold tracking-tight">EPP Transactions</h1>
      <p className="text-muted-foreground">
        Transaction views are not available yet.
      </p>
    </section>
  )
}
