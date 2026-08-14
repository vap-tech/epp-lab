import { createFileRoute } from "@tanstack/react-router"
import { Badge } from "@/components/ui/badge"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import { Switch } from "@/components/ui/switch"
import {
  useExtensionCatalog,
  useSetZoneExtension,
  useUpdateZone,
  useZone,
  useZoneExtensions,
} from "@/hooks/useZones"

const requirements = ["forbidden", "optional", "required"] as const
type Requirement = (typeof requirements)[number]

export const Route = createFileRoute("/_layout/zones/$zoneId")({
  component: ZoneDetail,
  head: () => ({ meta: [{ title: "Zone - EPP Lab" }] }),
})

function ZoneDetail() {
  const { zoneId } = Route.useParams()
  const query = useZone(zoneId)
  const mutations = useUpdateZone(zoneId)
  const catalog = useExtensionCatalog()
  const assignments = useZoneExtensions(zoneId)
  const setExtension = useSetZoneExtension(zoneId)

  if (query.isPending)
    return <p className="text-muted-foreground">Loading zone…</p>
  if (query.isError || !query.data)
    return <p className="text-destructive">Failed to load zone.</p>
  const zone = query.data

  return (
    <section className="flex flex-col gap-6">
      <div className="flex items-center justify-between gap-4">
        <div className="flex items-center gap-3">
          <h1 className="text-2xl font-bold tracking-tight">
            {zone.ascii_name}
          </h1>
          <Switch
            checked={zone.status === "active"}
            aria-label="Zone active"
            onCheckedChange={(checked) =>
              mutations.status.mutate(checked ? "active" : "disabled")
            }
          />
        </div>
        <Badge variant={zone.status === "active" ? "default" : "secondary"}>
          {zone.status}
        </Badge>
      </div>
      <div className="grid max-w-5xl gap-x-8 gap-y-3 text-sm sm:grid-cols-2 lg:grid-cols-4">
        <div>
          <span className="text-muted-foreground">Canonical name</span>
          <p className="font-mono">{zone.ascii_name}</p>
        </div>
        <div>
          <span className="text-muted-foreground">Unicode name</span>
          <p>{zone.unicode_name}</p>
        </div>
        <div>
          <span className="text-muted-foreground">Created</span>
          <p>{new Date(zone.created_at).toLocaleString()}</p>
        </div>
        <div>
          <span className="text-muted-foreground">Updated</span>
          <p>{new Date(zone.updated_at).toLocaleString()}</p>
        </div>
      </div>
      <section className="rounded-lg border p-6">
        <h2 className="font-semibold">Contact Usage</h2>
        <div className="mt-4 flex max-w-md flex-col gap-3">
          <PolicySelect
            label="Registrant"
            value={zone.contact_policy.registrant}
            onChange={(value) =>
              mutations.contactPolicy.mutate({
                ...zone.contact_policy,
                registrant: value,
              })
            }
          />
          <PolicySelect
            label="Admin"
            value={zone.contact_policy.admin}
            onChange={(value) =>
              mutations.contactPolicy.mutate({
                ...zone.contact_policy,
                admin: value,
              })
            }
          />
          <PolicySelect
            label="Tech"
            value={zone.contact_policy.tech}
            onChange={(value) =>
              mutations.contactPolicy.mutate({
                ...zone.contact_policy,
                tech: value,
              })
            }
          />
          <PolicySelect
            label="Billing"
            value={zone.contact_policy.billing}
            onChange={(value) =>
              mutations.contactPolicy.mutate({
                ...zone.contact_policy,
                billing: value,
              })
            }
          />
        </div>
        {zone.contactless ? (
          <p className="mt-4 text-sm text-muted-foreground">
            This is a contactless zone.
          </p>
        ) : null}
      </section>
      <section className="rounded-lg border p-6">
        <h2 className="font-semibold">Extensions</h2>
        {catalog.data?.length === 0 ? (
          <p className="mt-4 text-sm text-muted-foreground">
            No extensions are registered in this server.
          </p>
        ) : (
          <div className="mt-4 flex max-w-2xl flex-col gap-3">
            {catalog.data?.map((extension) => {
              const enabled =
                assignments.data?.find(
                  (item) => item.extension_key === extension.key,
                )?.enabled ?? false
              return (
                <div
                  key={extension.key}
                  className="flex items-center justify-between gap-4"
                >
                  <div>
                    <p>{extension.display_name}</p>
                    <p className="text-xs text-muted-foreground">
                      {extension.key}
                    </p>
                  </div>
                  <Switch
                    checked={enabled}
                    disabled={setExtension.isPending}
                    aria-label={`Enable ${extension.display_name}`}
                    onCheckedChange={(value) =>
                      setExtension.mutate({
                        key: extension.key,
                        enabled: value,
                      })
                    }
                  />
                </div>
              )
            })}
          </div>
        )}
      </section>
    </section>
  )
}

function PolicySelect({
  label,
  value,
  onChange,
}: {
  label: string
  value: Requirement
  onChange: (value: Requirement) => void
}) {
  return (
    <div className="grid grid-cols-[180px_160px] items-center gap-4">
      <span className="text-sm">{label}</span>
      <Select
        value={value}
        onValueChange={(next) => onChange(next as Requirement)}
      >
        <SelectTrigger className="w-32">
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          {requirements.map((requirement) => (
            <SelectItem key={requirement} value={requirement}>
              {requirement}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
    </div>
  )
}
