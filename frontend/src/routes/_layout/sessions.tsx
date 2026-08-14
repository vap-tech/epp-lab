import { createFileRoute } from "@tanstack/react-router"

import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { useEppSessions } from "@/hooks/useEpp"

export const Route = createFileRoute("/_layout/sessions")({
  component: Sessions,
  head: () => ({ meta: [{ title: "EPP Sessions - EPP Lab" }] }),
})

function Sessions() {
  const query = useEppSessions()
  return (
    <section className="flex flex-col gap-6">
      <div className="flex items-start justify-between">
        <div>
          <h1 className="text-2xl font-bold tracking-tight">EPP Sessions</h1>
          <p className="text-muted-foreground">
            Connection and authentication history.
          </p>
        </div>
        <Button variant="outline" onClick={() => query.refetch()}>
          Refresh
        </Button>
      </div>
      {query.isPending ? (
        <p className="text-muted-foreground">Loading sessions…</p>
      ) : query.isError ? (
        <p className="text-destructive">Failed to load EPP sessions.</p>
      ) : query.data?.items.length === 0 ? (
        <p className="text-muted-foreground">
          No EPP sessions have been recorded yet.
        </p>
      ) : (
        <div className="overflow-x-auto rounded-lg border">
          <table className="w-full text-sm">
            <thead className="bg-muted/50">
              <tr>
                {[
                  "State",
                  "Registrar",
                  "Remote",
                  "Connected",
                  "Commands",
                  "Disconnect",
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
                    <Badge
                      variant={
                        item.disconnected_at
                          ? "secondary"
                          : item.authenticated_at
                            ? "default"
                            : "outline"
                      }
                    >
                      {item.disconnected_at
                        ? "Closed"
                        : item.authenticated_at
                          ? "Authenticated"
                          : "Connected"}
                    </Badge>
                  </td>
                  <td className="px-4 py-3">{item.registrar?.handle ?? "—"}</td>
                  <td className="px-4 py-3 font-mono">{item.remote_addr}</td>
                  <td className="px-4 py-3">
                    {new Date(item.connected_at).toLocaleString()}
                  </td>
                  <td className="px-4 py-3">{item.transaction_count}</td>
                  <td className="px-4 py-3">{item.disconnect_reason ?? "—"}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </section>
  )
}
