import { createFileRoute, Outlet, useNavigate } from "@tanstack/react-router"
import { useEffect } from "react"

import { Footer } from "@/components/Common/Footer"
import AppSidebar from "@/components/Sidebar/AppSidebar"
import {
  SidebarInset,
  SidebarProvider,
  SidebarTrigger,
} from "@/components/ui/sidebar"
import { useAuth } from "@/hooks/useAuth"

export const Route = createFileRoute("/_layout")({
  component: Layout,
})

function Layout() {
  const navigate = useNavigate()
  const { session } = useAuth()
  useEffect(() => {
    const handleExpiredSession = () => navigate({ to: "/login" })
    window.addEventListener("epp-lab:auth-expired", handleExpiredSession)
    if (
      !session.isLoading &&
      (!session.data?.authenticated || session.isError)
    ) {
      navigate({ to: "/login" })
    }
    return () =>
      window.removeEventListener("epp-lab:auth-expired", handleExpiredSession)
  }, [
    navigate,
    session.data?.authenticated,
    session.isError,
    session.isLoading,
  ])
  if (session.isLoading || !session.data?.authenticated) return null

  return (
    <SidebarProvider>
      <AppSidebar />
      <SidebarInset>
        <header className="sticky top-0 z-10 flex h-16 shrink-0 items-center gap-2 border-b px-4">
          <SidebarTrigger className="-ml-1 text-muted-foreground" />
        </header>
        <main className="flex-1 p-6 md:p-8">
          <div className="mx-auto max-w-7xl">
            <Outlet />
          </div>
        </main>
        <Footer />
      </SidebarInset>
    </SidebarProvider>
  )
}
