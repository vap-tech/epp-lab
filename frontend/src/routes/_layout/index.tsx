import { createFileRoute } from "@tanstack/react-router"

export const Route = createFileRoute("/_layout/")({
  component: Dashboard,
  head: () => ({
    meta: [
      {
        title: "Dashboard - EPP Lab",
      },
    ],
  }),
})

function Dashboard() {
  return (
    <div>
      <div>
        <h1 className="text-2xl truncate max-w-sm">EPP Lab</h1>
        <p className="text-muted-foreground">
          Registry administration foundation.
        </p>
      </div>
    </div>
  )
}
