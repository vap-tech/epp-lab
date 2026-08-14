import { useNavigate } from "@tanstack/react-router"
import {
  ArrowRightLeft,
  ClipboardList,
  Gauge,
  LogOut,
  Users,
} from "lucide-react"

import { SidebarAppearance } from "@/components/Common/Appearance"
import { Logo } from "@/components/Common/Logo"
import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarHeader,
  SidebarMenuButton,
  SidebarMenuItem,
} from "@/components/ui/sidebar"
import { useAuth } from "@/hooks/useAuth"
import { type Item, Main } from "./Main"

const baseItems: Item[] = [
  { icon: Gauge, title: "Dashboard", path: "/" },
  { icon: Users, title: "Registrars", path: "/registrars" },
  { icon: ArrowRightLeft, title: "EPP Sessions", path: "/sessions" },
  { icon: ClipboardList, title: "EPP Transactions", path: "/transactions" },
]

export function AppSidebar() {
  const navigate = useNavigate()
  const { logoutMutation } = useAuth()

  return (
    <Sidebar collapsible="icon">
      <SidebarHeader className="px-4 py-6 group-data-[collapsible=icon]:px-0 group-data-[collapsible=icon]:items-center">
        <Logo variant="responsive" />
      </SidebarHeader>
      <SidebarContent>
        <Main items={baseItems} />
      </SidebarContent>
      <SidebarFooter>
        <SidebarMenuItem>
          <SidebarMenuButton
            onClick={() =>
              logoutMutation.mutate(undefined, {
                onSuccess: () => navigate({ to: "/login" }),
              })
            }
            disabled={logoutMutation.isPending}
            tooltip="Sign out"
          >
            <LogOut className="size-4 text-muted-foreground" />
            <span>Sign out</span>
          </SidebarMenuButton>
        </SidebarMenuItem>
        <SidebarAppearance />
      </SidebarFooter>
    </Sidebar>
  )
}

export default AppSidebar
