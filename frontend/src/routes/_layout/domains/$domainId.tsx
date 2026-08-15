import { createFileRoute, Link } from "@tanstack/react-router"
import { Badge } from "@/components/ui/badge"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { useDomain } from "@/hooks/useDomains"

export const Route = createFileRoute("/_layout/domains/$domainId")({ component: DomainDetail, head: () => ({ meta: [{ title: "Domain - EPP Lab" }] }) })

function DomainDetail() {
  const { domainId } = Route.useParams()
  const domain = useDomain(domainId)
  if (domain.isLoading) return <p className="text-muted-foreground">Loading domain…</p>
  if (domain.isError || !domain.data) return <p className="text-sm text-destructive">Unable to load domain.</p>
  const value = domain.data
  return <section className="flex flex-col gap-6"><div><Link to="/domains" className="text-sm text-muted-foreground hover:underline">← Domains</Link><h1 className="mt-3 text-2xl font-bold tracking-tight">{value.domain_name}</h1><p className="text-muted-foreground">{value.roid}</p></div>
    <Card><CardHeader><CardTitle>General</CardTitle></CardHeader><CardContent className="grid gap-4 sm:grid-cols-4"><div><p className="text-sm text-muted-foreground">Zone</p><p>{value.zone.name}</p></div><div><p className="text-sm text-muted-foreground">Registrar</p><p>{value.registrar.handle}</p></div><div><p className="text-sm text-muted-foreground">Created</p><p>{new Date(value.created_at).toLocaleString()}</p></div><div><p className="text-sm text-muted-foreground">Expires</p><p>{new Date(value.expires_at).toLocaleString()}</p></div></CardContent></Card>
    <Card><CardHeader><CardTitle>Status</CardTitle></CardHeader><CardContent className="flex gap-2">{value.statuses.map((status) => <Badge key={status}>{status}</Badge>)}</CardContent></Card>
    <Card><CardHeader><CardTitle>Nameservers</CardTitle></CardHeader><CardContent>{value.nameservers.length ? <ul className="space-y-2">{value.nameservers.map((nameserver) => <li key={nameserver}>{nameserver}</li>)}</ul> : <p className="text-muted-foreground">No nameservers.</p>}</CardContent></Card>
    <Card><CardHeader><CardTitle>Contacts</CardTitle></CardHeader><CardContent>{value.contacts.length ? <ul className="space-y-2">{value.contacts.map((contact) => <li key={`${contact.role}-${contact.contact_id}`} className="flex justify-between"><Link className="text-primary hover:underline" to="/contacts/$contactId" search={{ page: 1 }} params={{ contactId: contact.contact_id }}>{contact.contact_id}</Link><span className="text-muted-foreground">{contact.role}{contact.effective ? "" : " · dormant"}</span></li>)}</ul> : <p className="text-muted-foreground">No contacts.</p>}</CardContent></Card>
  </section>
}
