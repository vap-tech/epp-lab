import { createFileRoute } from "@tanstack/react-router"

import { Badge } from "@/components/ui/badge"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"

export const Route = createFileRoute("/_layout/sessions")({
  component: Sessions,
  head: () => ({ meta: [{ title: "EPP Sessions - EPP Lab" }] }),
})

function Sessions() {
  return (
    <section className="flex flex-col gap-6">
      <h1 className="text-2xl font-bold tracking-tight">EPP Sessions</h1>
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            Session history
            <Badge variant="secondary">Coming soon</Badge>
          </CardTitle>
        </CardHeader>
        <CardContent className="text-muted-foreground">
          The backend does not expose a session listing endpoint yet. No demo
          data is shown here.
        </CardContent>
      </Card>
    </section>
  )
}
