import { createFileRoute, Link, Outlet, useRouterState } from "@tanstack/react-router"
import { useState } from "react"
import { Badge } from "@/components/ui/badge"
import { Input } from "@/components/ui/input"
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table"
import { useDomains } from "@/hooks/useDomains"

export const Route = createFileRoute("/_layout/domains")({ component: Domains, head: () => ({ meta: [{ title: "Domains - EPP Lab" }] }) })

function Domains() {
  const pathname = useRouterState({ select: (state) => state.location.pathname })
  const [search, setSearch] = useState("")
  const domains = useDomains(1, search)
  if (pathname !== "/domains") return <Outlet />
  return <section className="flex flex-col gap-6">
    <div><h1 className="text-2xl font-bold tracking-tight">Domains</h1><p className="text-muted-foreground">Registry domains and their lifecycle state.</p></div>
    <Input value={search} onChange={(event) => setSearch(event.target.value)} placeholder="Search domains" aria-label="Search domains" className="max-w-sm" />
    {domains.isError ? <p className="text-sm text-destructive">Unable to load domains.</p> : null}
    <Table><TableHeader><TableRow><TableHead>Domain</TableHead><TableHead>Zone</TableHead><TableHead>Registrar</TableHead><TableHead>Status</TableHead><TableHead>Expires</TableHead></TableRow></TableHeader><TableBody>
      {domains.data?.items.map((domain) => <TableRow key={domain.id}><TableCell><Link className="font-medium text-primary hover:underline" to="/domains/$domainId" params={{ domainId: domain.id }}>{domain.domain_name}</Link><div className="text-sm text-muted-foreground">{domain.roid}</div></TableCell><TableCell>{domain.zone.name}</TableCell><TableCell>{domain.registrar.handle}</TableCell><TableCell><div className="flex gap-1">{domain.statuses.map((status) => <Badge key={status} variant={status === "ok" ? "default" : "secondary"}>{status}</Badge>)}</div></TableCell><TableCell>{new Date(domain.expires_at).toLocaleDateString()}</TableCell></TableRow>)}
      {domains.data?.items.length === 0 ? <TableRow><TableCell colSpan={5} className="h-24 text-center text-muted-foreground">No domains found.</TableCell></TableRow> : null}
    </TableBody></Table>
  </section>
}
