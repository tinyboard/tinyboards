import type { RouterConfig } from '@nuxt/schema'

export default <RouterConfig>{
  scrollBehavior (to, from, savedPosition) {
    // Restore saved scroll position on back/forward navigation
    if (savedPosition) {
      return savedPosition
    }

    // Smooth scroll to top on route change
    return { top: 0, behavior: 'smooth' }
  },
}
