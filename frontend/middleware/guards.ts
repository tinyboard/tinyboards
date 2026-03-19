import { useAuthStore } from '~/stores/auth'

/**
 * Route middleware that redirects unauthenticated users to the login page.
 * Apply to individual pages via `definePageMeta({ middleware: 'guards' })`.
 */
export default defineNuxtRouteMiddleware((to) => {
  const authStore = useAuthStore()

  if (!authStore.isLoggedIn) {
    return navigateTo({
      path: '/login',
      query: { redirect: to.fullPath },
    })
  }
})
