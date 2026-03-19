import { useAuthStore } from '~/stores/auth'
import { useUIStore } from '~/stores/ui'

/**
 * Global middleware that blocks non-admin users from /admin/* routes.
 * Redirects to home page if user is not an admin.
 */
export default defineNuxtRouteMiddleware((to) => {
  // Only apply to admin routes
  if (!to.path.startsWith('/admin')) { return }

  const authStore = useAuthStore()

  if (!authStore.isLoggedIn) {
    return navigateTo({
      path: '/login',
      query: { redirect: to.fullPath },
    })
  }

  if (!authStore.isAdmin) {
    if (import.meta.client) {
      const ui = useUIStore()
      ui.addToast({ type: 'warning', message: 'Admin access required', duration: 5000 })
    }
    return navigateTo('/home')
  }
})
