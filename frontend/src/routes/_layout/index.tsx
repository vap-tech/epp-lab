import { createFileRoute } from "@tanstack/react-router"
import { Badge } from "@/components/ui/badge"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"
import { useHealth, useInfo } from "@/hooks/useAdmin"

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
  const health = useHealth()
  const info = useInfo()

  return (
    <div className="flex flex-col gap-6">
      <div>
        <h1 className="text-2xl font-bold tracking-tight">Dashboard</h1>
        <p className="text-muted-foreground">Registry service overview.</p>
      </div>
      <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
        <StatusCard
          title="Registry"
          value={
            health.data?.status ??
            (health.isPending ? "Loading" : "Unavailable")
          }
          detail="Public health endpoint"
          good={health.data?.status === "ok"}
        />
        <StatusCard
          title="Database"
          value={
            health.data?.database ??
            (health.isPending ? "Loading" : "Unavailable")
          }
          detail="PostgreSQL connectivity"
          good={health.data?.database === "ok"}
        />
        <StatusCard
          title="Environment"
          value={info.data?.environment ?? "—"}
          detail="Application mode"
        />
        <StatusCard
          title="EPP listener"
          value={info.data?.epp_bind ?? "—"}
          detail={info.data?.version ?? ""}
        />
      </div>
      {health.isError || info.isError ? (
        <p className="text-sm text-destructive">
          Some service details are unavailable. Check the backend connection.
        </p>
      ) : null}
    </div>
  )
}

function StatusCard({
  title,
  value,
  detail,
  good,
}: {
  title: string
  value: string
  detail: string
  good?: boolean
}) {
  return (
    <Card>
      <CardHeader className="pb-3">
        <CardDescription>{title}</CardDescription>
        <CardTitle className="flex items-center gap-2 text-xl">
          {value}
          {good !== undefined ? (
            <Badge variant={good ? "default" : "destructive"}>
              {good ? "Healthy" : "Issue"}
            </Badge>
          ) : null}
        </CardTitle>
      </CardHeader>
      <CardContent className="text-sm text-muted-foreground">
        {detail}
      </CardContent>
    </Card>
  )
}
