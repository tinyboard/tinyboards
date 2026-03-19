import { useUIStore } from '~/stores/ui'

/**
 * Composable for showing toast notifications.
 * Wraps the UI store's toast methods with sensible defaults per type.
 */
export function useToast () {
  const ui = useUIStore()
  return {
    success: (message: string) => ui.addToast({ type: 'success', message, duration: 4000 }),
    error: (message: string) => ui.addToast({ type: 'error', message, duration: 6000 }),
    info: (message: string) => ui.addToast({ type: 'info', message, duration: 4000 }),
    warning: (message: string) => ui.addToast({ type: 'warning', message, duration: 5000 }),
  }
}
