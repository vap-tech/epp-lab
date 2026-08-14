import { createFileRoute, useNavigate } from "@tanstack/react-router"
import { useState } from "react"

import { AuthLayout } from "@/components/Common/AuthLayout"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { useAuth } from "@/hooks/useAuth"

export const Route = createFileRoute("/login")({
  component: Login,
  head: () => ({ meta: [{ title: "Sign in - EPP Lab" }] }),
})

function Login() {
  const navigate = useNavigate()
  const { loginMutation } = useAuth()
  const [username, setUsername] = useState("")
  const [password, setPassword] = useState("")
  return (
    <AuthLayout>
      <div className="flex flex-col gap-6">
        <div className="flex flex-col items-center gap-2 text-center">
          <h1 className="text-2xl font-bold">Sign in to EPP Lab</h1>
          <p className="text-muted-foreground text-sm">Admin access</p>
        </div>
        <form
          className="grid gap-4"
          onSubmit={(event) => {
            event.preventDefault()
            loginMutation.mutate(
              { username, password },
              { onSuccess: () => navigate({ to: "/" }) },
            )
          }}
        >
          <div className="grid gap-2">
            <Label htmlFor="username">Username</Label>
            <Input
              id="username"
              name="username"
              autoComplete="username"
              value={username}
              onChange={(event) => setUsername(event.target.value)}
            />
          </div>
          <div className="grid gap-2">
            <Label htmlFor="password">Password</Label>
            <Input
              id="password"
              name="password"
              type="password"
              autoComplete="current-password"
              value={password}
              onChange={(event) => setPassword(event.target.value)}
            />
          </div>
          <Button type="submit" disabled={loginMutation.isPending}>
            Sign in
          </Button>
          {loginMutation.isError && (
            <p className="text-destructive text-sm">
              Invalid username or password
            </p>
          )}
        </form>
      </div>
    </AuthLayout>
  )
}
