import { createFileRoute, Link } from "@tanstack/react-router"
import { Badge } from "@/components/ui/badge"
import { useEppSession } from "@/hooks/useEpp"

export const Route = createFileRoute("/_layout/sessions/$sessionId")({
  component: SessionDetail,
})

function SessionDetail() {
  const { sessionId } = Route.useParams()
  const query = useEppSession(sessionId)
  if (query.isPending)
    return <p className="text-muted-foreground">Loading session…</p>
  if (query.isError || !query.data)
    return <p className="text-destructive">Failed to load EPP session.</p>
  const item = query.data
  const state = item.disconnected_at
    ? "Closed"
    : item.authenticated_at
      ? "Authenticated"
      : "Connected"
  return (
    <section className="flex flex-col gap-6">
      <div>
        <h1 className="text-2xl font-bold tracking-tight">EPP Session</h1>
        <p className="font-mono text-sm text-muted-foreground">{item.id}</p>
      </div>
      <div className="grid gap-4 rounded-lg border p-6 sm:grid-cols-2">
        <Field label="State">
          <Badge>{state}</Badge>
        </Field>
        <Field label="Registrar">{item.registrar?.handle ?? "—"}</Field>
        <Field label="Certificate fingerprint">
          <span className="break-all font-mono">
            {item.certificate?.fingerprint_sha256 ?? "—"}
          </span>
        </Field>
        <Field label="Remote address">
          <span className="font-mono">{item.remote_addr}</span>
        </Field>
        <Field label="Connected">
          {new Date(item.connected_at).toLocaleString()}
        </Field>
        <Field label="Authenticated">
          {item.authenticated_at
            ? new Date(item.authenticated_at).toLocaleString()
            : "—"}
        </Field>
        <Field label="Disconnected">
          {item.disconnected_at
            ? new Date(item.disconnected_at).toLocaleString()
            : "—"}
        </Field>
        <Field label="Disconnect reason">{item.disconnect_reason ?? "—"}</Field>
      </div>
      <Link
        className="text-primary hover:underline"
        to="/transactions"
        search={{ session_id: item.id }}
      >
        View session transactions
      </Link>
    </section>
  )
}
function Field({
  label,
  children,
}: {
  label: string
  children: React.ReactNode
}) {
  return (
    <div>
      <dt className="text-sm text-muted-foreground">{label}</dt>
      <dd className="mt-1">{children}</dd>
    </div>
  )
}
