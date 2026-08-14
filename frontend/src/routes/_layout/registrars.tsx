import { createFileRoute } from "@tanstack/react-router"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"
import { useRegistrars } from "@/hooks/useAdmin"

export const Route = createFileRoute("/_layout/registrars")({
  component: Registrars,
  head: () => ({ meta: [{ title: "Registrars - EPP Lab" }] }),
})

function Registrars() {
  const registrars = useRegistrars()

  return (
    <section className="flex flex-col gap-6">
      <div>
        <h1 className="text-2xl font-bold tracking-tight">Registrars</h1>
        <p className="text-muted-foreground">
          Registered EPP clients and their current status.
        </p>
      </div>
      {registrars.isError ? (
        <p className="text-sm text-destructive">Unable to load registrars.</p>
      ) : null}
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>Handle</TableHead>
            <TableHead>Name</TableHead>
            <TableHead>Client ID</TableHead>
            <TableHead>Status</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {registrars.data?.map((registrar) => (
            <TableRow key={registrar.id}>
              <TableCell className="font-medium">{registrar.handle}</TableCell>
              <TableCell>{registrar.name || "—"}</TableCell>
              <TableCell>{registrar.client_id}</TableCell>
              <TableCell>{registrar.status}</TableCell>
            </TableRow>
          ))}
          {registrars.data?.length === 0 ? (
            <TableRow>
              <TableCell
                colSpan={4}
                className="h-24 text-center text-muted-foreground"
              >
                No registrars configured.
              </TableCell>
            </TableRow>
          ) : null}
        </TableBody>
      </Table>
    </section>
  )
}
