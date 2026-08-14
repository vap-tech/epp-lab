import { createFileRoute, Link } from "@tanstack/react-router"
import { Badge } from "@/components/ui/badge"
import { useContact } from "@/hooks/useContacts"

export const Route = createFileRoute("/_layout/contacts/$contactId")({
  component: ContactDetail,
  head: () => ({ meta: [{ title: "Contact - EPP Lab" }] }),
})

function ContactDetail() {
  const { contactId } = Route.useParams()
  const contact = useContact(contactId)

  if (contact.isError)
    return <p className="text-sm text-destructive">Unable to load contact.</p>
  if (!contact.data)
    return <p className="text-sm text-muted-foreground">Loading contact…</p>

  const item = contact.data
  return (
    <section className="flex flex-col gap-6">
      <div>
        <Link
          to="/contacts"
          className="text-sm text-muted-foreground hover:underline"
        >
          ← Contacts
        </Link>
        <div className="mt-3 flex items-center justify-between gap-4">
          <div>
            <h1 className="text-2xl font-bold tracking-tight">
              {item.contact_id}
            </h1>
            <p className="text-muted-foreground">{item.roid}</p>
          </div>
          <Badge variant={item.linked ? "default" : "secondary"}>
            {item.linked ? "linked" : "unlinked"}
          </Badge>
        </div>
      </div>
      <div className="rounded-xl border p-6">
        <h2 className="font-semibold">Contact</h2>
        <dl className="mt-4 grid gap-4 sm:grid-cols-2">
          <div>
            <dt className="text-sm text-muted-foreground">Email</dt>
            <dd>{item.email}</dd>
          </div>
          <div>
            <dt className="text-sm text-muted-foreground">Registrar</dt>
            <dd className="break-all">{item.registrar_id}</dd>
          </div>
          <div>
            <dt className="text-sm text-muted-foreground">Created</dt>
            <dd>{new Date(item.created_at).toLocaleString()}</dd>
          </div>
          <div>
            <dt className="text-sm text-muted-foreground">Updated</dt>
            <dd>{new Date(item.updated_at).toLocaleString()}</dd>
          </div>
        </dl>
      </div>
      <div className="rounded-xl border p-6">
        <h2 className="font-semibold">Statuses</h2>
        <div className="mt-4 flex flex-wrap gap-2">
          {item.statuses.map((status) => (
            <Badge key={status} variant="secondary">
              {status}
            </Badge>
          ))}
        </div>
      </div>
      <p className="text-sm text-muted-foreground">
        Authentication information is never exposed in the admin interface.
      </p>
    </section>
  )
}
