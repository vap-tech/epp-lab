import { createFileRoute } from "@tanstack/react-router"

import { AuthLayout } from "@/components/Common/AuthLayout"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"

export const Route = createFileRoute("/login")({
  component: Login,
  head: () => ({ meta: [{ title: "Sign in - EPP Lab" }] }),
})

function Login() {
  return (
    <AuthLayout>
      <div className="flex flex-col gap-6">
        <div className="flex flex-col items-center gap-2 text-center">
          <h1 className="text-2xl font-bold">Sign in to EPP Lab</h1>
          <p className="text-muted-foreground text-sm">
            Admin authentication will be enabled in the next iteration.
          </p>
        </div>
        <form
          className="grid gap-4"
          onSubmit={(event) => event.preventDefault()}
        >
          <div className="grid gap-2">
            <Label htmlFor="username">Username</Label>
            <Input id="username" name="username" autoComplete="username" />
          </div>
          <div className="grid gap-2">
            <Label htmlFor="password">Password</Label>
            <Input
              id="password"
              name="password"
              type="password"
              autoComplete="current-password"
            />
          </div>
          <Button type="submit">Sign in</Button>
        </form>
      </div>
    </AuthLayout>
  )
}
