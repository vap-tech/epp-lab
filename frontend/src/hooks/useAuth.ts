import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import { getSession, login, logout } from "@/api/auth"

export const authSessionKey = ["auth", "session"]

export function useAuth() {
  const queryClient = useQueryClient()
  const session = useQuery({
    queryKey: authSessionKey,
    queryFn: getSession,
    retry: false,
  })
  const loginMutation = useMutation({
    mutationFn: ({
      username,
      password,
    }: {
      username: string
      password: string
    }) => login(username, password),
    onSuccess: (value) => queryClient.setQueryData(authSessionKey, value),
  })
  const logoutMutation = useMutation({
    mutationFn: logout,
    onSuccess: () =>
      queryClient.setQueryData(authSessionKey, {
        authenticated: false,
        user: null,
        csrf_token: null,
      }),
  })
  return { session, loginMutation, logoutMutation }
}
