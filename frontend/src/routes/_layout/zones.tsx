import { createFileRoute } from "@tanstack/react-router"
import { Badge } from "@/components/ui/badge"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"
import { useZones } from "@/hooks/useZones"

export const Route = createFileRoute("/_layout/zones")({
  component: Zones,
  head: () => ({ meta: [{ title: "Zones - EPP Lab" }] }),
})

function Zones() {
  const zones = useZones()

  return (
    <section className="flex flex-col gap-6">
      <div>
        <h1 className="text-2xl font-bold tracking-tight">Zones</h1>
        <p className="text-muted-foreground">
          Registry zones and their contact usage policies.
        </p>
      </div>
      {zones.isError ? (
        <p className="text-sm text-destructive">Unable to load zones.</p>
      ) : null}
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>Zone</TableHead>
            <TableHead>Status</TableHead>
            <TableHead>Contacts</TableHead>
            <TableHead>Extensions</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {zones.data?.map((zone) => (
            <TableRow key={zone.id}>
              <TableCell>
                <div className="font-medium">{zone.ascii_name}</div>
                {zone.unicode_name !== zone.ascii_name ? (
                  <div className="text-sm text-muted-foreground">
                    {zone.unicode_name}
                  </div>
                ) : null}
              </TableCell>
              <TableCell>
                <Badge
                  variant={zone.status === "active" ? "default" : "secondary"}
                >
                  {zone.status}
                </Badge>
              </TableCell>
              <TableCell>
                {zone.contactless ? "Contactless" : "Configured"}
              </TableCell>
              <TableCell>{zone.enabled_extensions_count}</TableCell>
            </TableRow>
          ))}
          {zones.data?.length === 0 ? (
            <TableRow>
              <TableCell
                colSpan={4}
                className="h-24 text-center text-muted-foreground"
              >
                No zones configured.
              </TableCell>
            </TableRow>
          ) : null}
        </TableBody>
      </Table>
    </section>
  )
}
