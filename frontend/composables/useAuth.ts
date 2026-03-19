import { computed, ref } from 'vue'
import { useAuthStore } from '~/stores/auth'
import { useGraphQL } from '~/composables/useGraphQL'
import { useToast } from '~/composables/useToast'
import type { User } from '~/types/generated'
import type { LoginInput, RegisterInput, AuthRestResponse, RegisterRestResponse } from '~/types/api'

const ME_QUERY = `
  query Me {
    me {
      user {
        id
        name
        displayName
        avatar
        adminLevel
        postCount
        commentCount
        isBanned
        bio
        banner
      }
      unreadNotificationsCount
    }
  }
`

const SUBSCRIBED_BOARDS_QUERY = `
  query SubscribedBoards {
    listBoards(listingType: subscribed, limit: 50) {
      name
      title
      icon
      subscribers
    }
  }
`

interface MeResponse {
  me: { user: User; unreadNotificationsCount: number } | null
}

/**
 * Composable for authentication state and operations.
 * Login and register use dedicated REST endpoints via BFF proxy routes.
 * The me query and other data fetching still use GraphQL.
 * Reads auth state from the Pinia store only — never accesses cookies client-side (fixes BUG-006).
 */
export function useAuth () {
  const store = useAuthStore()
  const loading = ref(false)
  const error = ref<{ message: string } | null>(null)

  const user = computed(() => store.user)
  const isLoggedIn = computed(() => store.isLoggedIn)
  const isAdmin = computed(() => store.isAdmin)

  /**
   * Fetch the current user from the server using the httpOnly cookie.
   * Called during SSR/middleware to populate auth state.
   */
  async function fetchMe (): Promise<User | null> {
    const { execute: executeMe, error: meError } = useGraphQL<MeResponse>()
    const result = await executeMe(ME_QUERY)

    if (meError.value || !result?.me?.user) {
      store.clearUser()
      return null
    }

    store.setUser(result.me.user)

    // Fetch subscribed boards for the sidebar
    await fetchSubscribedBoards()

    return result.me.user
  }

  async function fetchSubscribedBoards (): Promise<void> {
    const { execute: exec } = useGraphQL<{ listBoards: Array<{ name: string; title: string; icon: string | null; subscribers: number }> }>()
    const result = await exec(SUBSCRIBED_BOARDS_QUERY)
    if (result?.listBoards) {
      store.setSubscribedBoards(result.listBoards)
    }
  }

  async function login (input: LoginInput): Promise<boolean> {
    loading.value = true
    error.value = null
    const toast = useToast()

    try {
      const data = await $fetch<AuthRestResponse>('/api/auth/login', {
        method: 'POST',
        body: {
          username_or_email: input.usernameOrEmail,
          password: input.password,
        },
      })

      if (!data.success) {
        error.value = { message: data.message ?? 'Login failed' }
        toast.error(data.message ?? 'Login failed')
        return false
      }

      // After login, fetch user profile via GraphQL (the cookie is now set)
      await fetchMe()
      toast.success('Logged in')
      return true
    } catch (err: unknown) {
      const fetchError = err as { data?: { error?: string }; statusMessage?: string }
      const msg = fetchError.data?.error ?? fetchError.statusMessage ?? 'Login failed'
      error.value = { message: msg }
      toast.error(msg)
      return false
    } finally {
      loading.value = false
    }
  }

  async function register (input: RegisterInput): Promise<boolean> {
    loading.value = true
    error.value = null
    const toast = useToast()

    try {
      const data = await $fetch<RegisterRestResponse>('/api/auth/register', {
        method: 'POST',
        body: {
          username: input.username,
          password: input.password,
          display_name: input.displayName,
          email: input.email,
          invite_code: input.inviteCode,
          application_answer: input.applicationAnswer,
        },
      })

      if (!data.success && !data.application_submitted) {
        error.value = { message: data.message ?? 'Registration failed' }
        toast.error(data.message ?? 'Registration failed')
        return false
      }

      if (data.application_submitted) {
        toast.info('Application submitted. You will be notified when approved.')
      } else if (data.user) {
        // Registration succeeded and account is immediately active — cookies are set
        await fetchMe()
        toast.success('Account created')
      }

      return true
    } catch (err: unknown) {
      const fetchError = err as { data?: { error?: string }; statusMessage?: string }
      const msg = fetchError.data?.error ?? fetchError.statusMessage ?? 'Registration failed'
      error.value = { message: msg }
      toast.error(msg)
      return false
    } finally {
      loading.value = false
    }
  }

  async function logout (): Promise<void> {
    const toast = useToast()
    // Call the BFF logout route which proxies to the backend and clears cookies
    try {
      await $fetch('/api/auth/logout', { method: 'POST' })
    } catch {
      // Even if the backend logout fails, clear local state
      // and fall back to the legacy cookie-clearing route
      await $fetch('/api/logout', { method: 'POST' })
    }

    store.clearUser()
    toast.success('Logged out')

    // Navigate to home after logout
    if (import.meta.client) {
      await navigateTo('/home')
    }
  }

  return {
    user,
    isLoggedIn,
    isAdmin,
    loading,
    error,
    fetchMe,
    login,
    register,
    logout,
  }
}
