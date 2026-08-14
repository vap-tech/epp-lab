import {
  createFileRoute,
  Link,
  Outlet,
  useNavigate,
  useRouterState,
} from "@tanstack/react-router"
import { Badge } from "@/components/ui/badge"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"
import { useContacts } from "@/hooks/useContacts"

export const Route = createFileRoute("/_layout/contacts")({
  component: Contacts,
  head: () => ({ meta: [{ title: "Contacts - EPP Lab" }] }),
  validateSearch: (search) => ({
    page: Number(search.page) > 0 ? Number(search.page) : 1,
  }),
})

function Contacts() {
  const pathname = useRouterState({
    select: (state) => state.location.pathname,
  })
  const { page } = Route.useSearch()
  const navigate = useNavigate({ from: Route.fullPath })
  const contacts = useContacts(page)

  if (pathname !== "/contacts") return <Outlet />

  return (
    <section className="flex flex-col gap-6">
      <div>
        <h1 className="text-2xl font-bold tracking-tight">Contacts</h1>
        <p className="text-muted-foreground">
          EPP contact objects registered by clients.
        </p>
      </div>
      {contacts.isError ? (
        <p className="text-sm text-destructive">Unable to load contacts.</p>
      ) : null}
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>Contact ID</TableHead>
            <TableHead>ROID</TableHead>
            <TableHead>Email</TableHead>
            <TableHead>Registrar</TableHead>
            <TableHead>Status</TableHead>
            <TableHead>Linked</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {contacts.data?.items.map((contact) => (
            <TableRow key={contact.id}>
              <TableCell>
                <Link
                  className="font-medium text-primary hover:underline"
                  to="/contacts/$contactId"
                  params={{ contactId: contact.id }}
                  search={{ page }}
                >
                  {contact.contact_id}
                </Link>
              </TableCell>
              <TableCell>{contact.roid}</TableCell>
              <TableCell>{contact.email}</TableCell>
              <TableCell>
                {contact.registrar_handle ?? contact.registrar_id}
              </TableCell>
              <TableCell>
                <div className="flex flex-wrap gap-1">
                  {contact.statuses.map((status) => (
                    <Badge key={status} variant="secondary">
                      {status}
                    </Badge>
                  ))}
                </div>
              </TableCell>
              <TableCell>{contact.linked ? "Yes" : "No"}</TableCell>
            </TableRow>
          ))}
          {contacts.data?.items.length === 0 ? (
            <TableRow>
              <TableCell
                colSpan={6}
                className="h-24 text-center text-muted-foreground"
              >
                No contacts registered.
              </TableCell>
            </TableRow>
          ) : null}
        </TableBody>
      </Table>
      {contacts.data && contacts.data.total_pages > 1 ? (
        <div className="flex items-center justify-end gap-3 text-sm">
          <button
            className="text-muted-foreground disabled:opacity-50"
            disabled={page <= 1}
            onClick={() => navigate({ search: { page: page - 1 } })}
          >
            Previous
          </button>
          <span>
            Page {page} of {contacts.data.total_pages}
          </span>
          <button
            className="text-muted-foreground disabled:opacity-50"
            disabled={page >= contacts.data.total_pages}
            onClick={() => navigate({ search: { page: page + 1 } })}
          >
            Next
          </button>
        </div>
      ) : null}
    </section>
  )
}
