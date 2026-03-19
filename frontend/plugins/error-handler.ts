import { defineNuxtPlugin } from '#app'
import { useLogger } from '~/utils/logger'

export default defineNuxtPlugin((nuxtApp) => {
  const logger = useLogger('error-handler')

  nuxtApp.vueApp.config.errorHandler = (error, _instance, info) => {
    // Skip navigation errors — Nuxt handles them
    if (error instanceof Error && error.message.includes('navigation')) {
      logger.debug('Navigation error (handled by Nuxt)', { error })
    } else {
      logger.error('Unhandled Vue error', { error, info })
    }
  }

  nuxtApp.vueApp.config.warnHandler = (msg, instance, trace) => {
    logger.warn('Vue warning', { msg, trace })
  }

  // Handle unhandled promise rejections on the client
  if (import.meta.client) {
    window.addEventListener('unhandledrejection', (event) => {
      logger.error('Unhandled promise rejection', { reason: event.reason })
    })
  }
})
