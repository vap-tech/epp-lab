import {
  createFileRoute,
  Link,
  Outlet,
  useRouterState,
} from "@tanstack/react-router"
import { z } from "zod"

import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { useEppTransactions } from "@/hooks/useEpp"

export const Route = createFileRoute("/_layout/transactions")({
  component: Transactions,
  validateSearch: z.object({
    page: z.coerce.number().int().min(1).catch(1),
    command: z.string().optional(),
    delivery_status: z.string().optional(),
    trid: z.string().optional(),
  }),
  head: () => ({ meta: [{ title: "EPP Transactions - EPP Lab" }] }),
})

function Transactions() {
  const { page, command, delivery_status, trid } = Route.useSearch()
  const navigate = Route.useNavigate()
  const pathname = useRouterState({
    select: (state) => state.location.pathname,
  })
  const query = useEppTransactions(page, { command, delivery_status, trid })
  if (pathname !== "/transactions") return <Outlet />
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
      <div className="flex flex-wrap gap-2">
        <input
          className="h-9 rounded-md border bg-background px-3 text-sm"
          placeholder="Search clTRID or svTRID"
          defaultValue={trid}
          onKeyDown={(event) => {
            if (event.key === "Enter")
              navigate({
                search: {
                  page: 1,
                  command,
                  delivery_status,
                  trid: event.currentTarget.value || undefined,
                },
              })
          }}
        />
        <select
          className="h-9 rounded-md border bg-background px-3 text-sm"
          value={command ?? ""}
          onChange={(event) =>
            navigate({
              search: {
                page: 1,
                command: event.target.value || undefined,
                delivery_status,
                trid,
              },
            })
          }
        >
          <option value="">All commands</option>
          <option value="hello">hello</option>
          <option value="login">login</option>
          <option value="logout">logout</option>
        </select>
        <select
          className="h-9 rounded-md border bg-background px-3 text-sm"
          value={delivery_status ?? ""}
          onChange={(event) =>
            navigate({
              search: {
                page: 1,
                command,
                delivery_status: event.target.value || undefined,
                trid,
              },
            })
          }
        >
          <option value="">All delivery</option>
          <option value="delivered">delivered</option>
          <option value="failed">failed</option>
          <option value="unknown">unknown</option>
        </select>
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
                  <td className="px-4 py-3 font-mono">
                    <Link
                      className="text-primary hover:underline"
                      to="/transactions/$transactionId"
                      search={{ page: 1 }}
                      params={{ transactionId: item.id }}
                    >
                      {item.command}
                    </Link>
                  </td>
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
          <div className="flex items-center justify-between border-t p-3 text-sm text-muted-foreground">
            <span>{query.data?.total ?? 0} transactions</span>
            <div className="flex gap-2">
              <Button
                variant="outline"
                size="sm"
                disabled={page <= 1}
                onClick={() => navigate({ search: { page: page - 1 } })}
              >
                Previous
              </Button>
              <span className="px-2 py-1">
                Page {page} of {query.data?.total_pages ?? 0}
              </span>
              <Button
                variant="outline"
                size="sm"
                disabled={page >= (query.data?.total_pages ?? 0)}
                onClick={() => navigate({ search: { page: page + 1 } })}
              >
                Next
              </Button>
            </div>
          </div>
        </div>
      )}
      <Outlet />
    </section>
  )
}
