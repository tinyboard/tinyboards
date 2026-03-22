import { useAuthStore } from '~/stores/auth'

/**
 * Global middleware that checks auth state and populates the auth store.
 * Runs after site middleware (01) so site config is available.
 *
 * During SSR, reads pre-fetched user data from the Nitro server middleware
 * (event.context.auth) to avoid a second round-trip to the backend.
 * On client-side navigation, the store is already populated from SSR hydration.
 */
export default defineNuxtRouteMiddleware(async () => {
  const authStore = useAuthStore()

  // Store already populated — skip
  if (authStore.user !== null) { return }

  if (import.meta.server) {
    const event = useRequestEvent()
    const authContext = event?.context?.auth

    if (authContext?.isAuthenticated && authContext.user) {
      authStore.setUser(authContext.user)

      if (authContext.subscribedBoards) {
        authStore.setSubscribedBoards(authContext.subscribedBoards)
      }

      if (typeof authContext.unreadNotificationsCount === 'number') {
        authStore.setUnreadNotificationCount(authContext.unreadNotificationsCount)
      }
    }
  }
})
