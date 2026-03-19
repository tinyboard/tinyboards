import { useAuthStore } from '~/stores/auth'
import { useGraphQL } from '~/composables/useGraphQL'

const UNREAD_COUNT_QUERY = `
  query GetUnreadCount {
    getUnreadNotificationCount {
      total
    }
  }
`

const MARK_ALL_READ_MUTATION = `
  mutation MarkAllRead {
    markAllNotificationsAsRead {
      success
    }
  }
`

const POLL_INTERVAL_MS = 30_000

/**
 * Polls the unread notification count every 30 seconds while the user is
 * logged in and the tab is visible. Pauses when the tab is hidden and
 * resumes when it becomes visible again. Stops on logout.
 *
 * Must only be called client-side (use within <ClientOnly> or guard with
 * import.meta.client).
 */
export function useNotificationPolling () {
  const authStore = useAuthStore()
  let intervalId: ReturnType<typeof setInterval> | null = null

  async function fetchCount (): Promise<void> {
    if (!authStore.isLoggedIn) return

    const { execute } = useGraphQL<{ getUnreadNotificationCount: { total: number } }>()
    const result = await execute(UNREAD_COUNT_QUERY)
    if (result?.getUnreadNotificationCount) {
      authStore.setUnreadNotificationCount(result.getUnreadNotificationCount.total)
    }
  }

  async function markAllRead (): Promise<void> {
    const { execute } = useGraphQL()
    await execute(MARK_ALL_READ_MUTATION)
    authStore.setUnreadNotificationCount(0)
  }

  function startPolling (): void {
    if (intervalId) return
    fetchCount()
    intervalId = setInterval(fetchCount, POLL_INTERVAL_MS)
  }

  function stopPolling (): void {
    if (intervalId) {
      clearInterval(intervalId)
      intervalId = null
    }
  }

  function handleVisibility (): void {
    if (document.visibilityState === 'visible') {
      startPolling()
    } else {
      stopPolling()
    }
  }

  // Only run client-side
  if (import.meta.client) {
    // Start polling if user is logged in
    if (authStore.isLoggedIn) {
      startPolling()
      document.addEventListener('visibilitychange', handleVisibility)
    }

    // Watch for login/logout
    watch(() => authStore.isLoggedIn, (loggedIn) => {
      if (loggedIn) {
        startPolling()
        document.addEventListener('visibilitychange', handleVisibility)
      } else {
        stopPolling()
        document.removeEventListener('visibilitychange', handleVisibility)
      }
    })

    // Clean up on component unmount
    onUnmounted(() => {
      stopPolling()
      document.removeEventListener('visibilitychange', handleVisibility)
    })
  }

  return {
    fetchCount,
    markAllRead,
  }
}
