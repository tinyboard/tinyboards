interface LogContext {
  [key: string]: unknown
}

interface Logger {
  debug: (message: string, context?: LogContext) => void
  info: (message: string, context?: LogContext) => void
  warn: (message: string, context?: LogContext) => void
  error: (message: string, context?: LogContext) => void
}

/**
 * Logging utility that only outputs in development mode.
 * All console output is gated behind import.meta.dev.
 * Fixes BUG-026 and BUG-039.
 */
export function useLogger (scope: string): Logger {
  const prefix = `[${scope}]`

  return {
    debug (message: string, context?: LogContext) {
      if (import.meta.dev) {
        // eslint-disable-next-line no-console
        console.debug(prefix, message, context ?? '')
      }
    },
    info (message: string, context?: LogContext) {
      if (import.meta.dev) {
        // eslint-disable-next-line no-console
        console.info(prefix, message, context ?? '')
      }
    },
    warn (message: string, context?: LogContext) {
      if (import.meta.dev) {
        // eslint-disable-next-line no-console
        console.warn(prefix, message, context ?? '')
      }
    },
    error (message: string, context?: LogContext) {
      if (import.meta.dev) {
        // eslint-disable-next-line no-console
        console.error(prefix, message, context ?? '')
      }
    },
  }
}
