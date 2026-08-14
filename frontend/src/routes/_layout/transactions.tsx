import { createFileRoute } from "@tanstack/react-router"

import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { useEppTransactions } from "@/hooks/useEpp"

export const Route = createFileRoute("/_layout/transactions")({
  component: Transactions,
  head: () => ({ meta: [{ title: "EPP Transactions - EPP Lab" }] }),
})

function Transactions() {
  const query = useEppTransactions()
  return (
    <section className="flex flex-col gap-6">
      <div className="flex items-start justify-between">
        <div>
          <h1 className="text-2xl font-bold tracking-tight">
            EPP Transactions
          </h1>
          <p className="text-muted-foreground">
            Protocol exchanges and delivery outcomes.
          </p>
        </div>
        <Button variant="outline" onClick={() => query.refetch()}>
          Refresh
        </Button>
      </div>
      {query.isPending ? (
        <p className="text-muted-foreground">Loading transactions…</p>
      ) : query.isError ? (
        <p className="text-destructive">Failed to load EPP transactions.</p>
      ) : query.data?.items.length === 0 ? (
        <p className="text-muted-foreground">
          No EPP transactions have been recorded yet.
        </p>
      ) : (
        <div className="overflow-x-auto rounded-lg border">
          <table className="w-full text-sm">
            <thead className="bg-muted/50">
              <tr>
                {[
                  "Time",
                  "Registrar",
                  "Command",
                  "Result",
                  "Delivery",
                  "Duration",
                  "clTRID",
                ].map((title) => (
                  <th
                    className="px-4 py-3 text-left text-xs uppercase text-muted-foreground"
                    key={title}
                  >
                    {title}
                  </th>
                ))}
              </tr>
            </thead>
            <tbody>
              {query.data?.items.map((item) => (
                <tr className="border-t" key={item.id}>
                  <td className="px-4 py-3">
                    {new Date(item.started_at).toLocaleString()}
                  </td>
                  <td className="px-4 py-3">{item.registrar?.handle ?? "—"}</td>
                  <td className="px-4 py-3 font-mono">{item.command}</td>
                  <td className="px-4 py-3">{item.response_code ?? "—"}</td>
                  <td className="px-4 py-3">
                    <Badge
                      variant={
                        item.delivery_status === "delivered"
                          ? "default"
                          : item.delivery_status === "failed"
                            ? "destructive"
                            : "secondary"
                      }
                    >
                      {item.delivery_status}
                    </Badge>
                  </td>
                  <td className="px-4 py-3">
                    {item.duration_ms === null ? "—" : `${item.duration_ms} ms`}
                  </td>
                  <td className="px-4 py-3 font-mono">{item.cl_trid ?? "—"}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </section>
  )
}
