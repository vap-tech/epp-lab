import { createFileRoute, Link } from "@tanstack/react-router"
import { useState } from "react"
import { XmlViewer } from "@/components/protocol/xml-viewer"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { useEppTransaction } from "@/hooks/useEpp"

export const Route = createFileRoute("/_layout/transactions/$transactionId")({
  component: TransactionDetail,
})

function TransactionDetail() {
  const { transactionId } = Route.useParams()
  const query = useEppTransaction(transactionId)
  const [raw, setRaw] = useState(false)
  const [wrap, setWrap] = useState(false)
  if (query.isPending)
    return <p className="text-muted-foreground">Loading transaction…</p>
  if (query.isError || !query.data)
    return <p className="text-destructive">Failed to load EPP transaction.</p>
  const item = query.data
  return (
    <section className="flex flex-col gap-6">
      <div>
        <h1 className="text-2xl font-bold tracking-tight">EPP Transaction</h1>
        <p className="font-mono text-sm text-muted-foreground">{item.id}</p>
      </div>
      <div className="grid gap-4 rounded-lg border p-6 sm:grid-cols-2">
        <Field label="Command">{item.command}</Field>
        <Field label="Registrar">{item.registrar?.handle ?? "—"}</Field>
        <Field label="Session">
          <Link
            className="text-primary hover:underline"
            to="/sessions/$sessionId"
            search={{ page: 1 }}
            params={{ sessionId: item.session_id }}
          >
            {item.session_id}
          </Link>
        </Field>
        <Field label="Protocol result">
          {item.response_code ?? "Greeting / no result code"}
        </Field>
        <Field label="Delivery">
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
        </Field>
        <Field label="clTRID">{item.cl_trid ?? "—"}</Field>
        <Field label="svTRID">{item.sv_trid}</Field>
        <Field label="Duration">
          {item.duration_ms === null ? "—" : `${item.duration_ms} ms`}
        </Field>
      </div>
      <section className="flex flex-col gap-4">
        <div className="flex justify-end gap-1">
          <Button
            variant={raw ? "default" : "secondary"}
            size="sm"
            onClick={() => setRaw((value) => !value)}
          >
            Raw
          </Button>
          <Button
            variant={wrap ? "default" : "secondary"}
            size="sm"
            onClick={() => setWrap((value) => !value)}
          >
            Wrap
          </Button>
        </div>
        <XmlViewer
          title="Request XML"
          xml={item.request_xml}
          raw={raw}
          wrap={wrap}
        />
        <XmlViewer
          title="Response XML"
          xml={item.response_xml}
          raw={raw}
          wrap={wrap}
        />
      </section>
      {item.delivery_error ? (
        <Field label="Delivery error">{item.delivery_error}</Field>
      ) : null}
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
