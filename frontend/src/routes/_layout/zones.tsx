import { createFileRoute, Link } from "@tanstack/react-router"
import { Plus } from "lucide-react"
import { useState } from "react"
import { ApiError } from "@/api/http"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog"
import { Input } from "@/components/ui/input"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"
import { useCreateZone, useZones } from "@/hooks/useZones"

export const Route = createFileRoute("/_layout/zones")({
  component: Zones,
  head: () => ({ meta: [{ title: "Zones - EPP Lab" }] }),
})

function Zones() {
  const zones = useZones()
  const createZone = useCreateZone()
  const [name, setName] = useState("")
  const [open, setOpen] = useState(false)
  const createError =
    createZone.error instanceof ApiError ? createZone.error.status : 0

  const submit = (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault()
    createZone.mutate(name.trim(), {
      onSuccess: () => {
        setName("")
        setOpen(false)
      },
    })
  }

  return (
    <section className="flex flex-col gap-6">
      <div>
        <div className="flex items-start justify-between gap-4">
          <div>
            <h1 className="text-2xl font-bold tracking-tight">Zones</h1>
            <p className="text-muted-foreground">
              Registry zones and their contact usage policies.
            </p>
          </div>
          <Dialog open={open} onOpenChange={setOpen}>
            <DialogTrigger asChild>
              <Button>
                <Plus /> Create zone
              </Button>
            </DialogTrigger>
            <DialogContent>
              <form onSubmit={submit}>
                <DialogHeader>
                  <DialogTitle>Create zone</DialogTitle>
                  <DialogDescription>
                    Enter a canonical DNS or Unicode IDN zone name.
                  </DialogDescription>
                </DialogHeader>
                <div className="py-4">
                  <Input
                    autoFocus
                    value={name}
                    onChange={(event) => setName(event.target.value)}
                    placeholder="com or рф"
                    aria-label="Zone name"
                  />
                  {createError === 400 ? (
                    <p className="mt-2 text-sm text-destructive">
                      Enter a valid zone name.
                    </p>
                  ) : null}
                  {createError === 409 ? (
                    <p className="mt-2 text-sm text-destructive">
                      This zone already exists.
                    </p>
                  ) : null}
                </div>
                <DialogFooter>
                  <Button
                    type="submit"
                    disabled={!name.trim() || createZone.isPending}
                  >
                    {createZone.isPending ? "Creating…" : "Create zone"}
                  </Button>
                </DialogFooter>
              </form>
            </DialogContent>
          </Dialog>
        </div>
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
                <Link
                  className="font-medium text-primary hover:underline"
                  to="/zones/$zoneId"
                  params={{ zoneId: zone.id }}
                >
                  {zone.ascii_name}
                </Link>
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
