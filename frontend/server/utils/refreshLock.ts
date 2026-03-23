/**
 * Prevents concurrent token refresh attempts (thundering herd problem).
 *
 * When multiple requests hit the BFF with an expired access token at the same
 * time, each would independently attempt to refresh. Because the backend
 * rotates the refresh token hash on every refresh, only the first succeeds —
 * subsequent attempts use an already-invalidated token and fail, logging the
 * user out.
 *
 * This module serializes refresh attempts: the first caller does the actual
 * refresh, and all concurrent callers wait for that result.
 */
let activeRefresh: Promise<string | null> | null = null

/**
 * Execute a refresh callback, deduplicating concurrent calls.
 * Returns the new access token string on success, or null on failure.
 */
export async function withRefreshLock (
  refreshFn: () => Promise<string | null>,
): Promise<string | null> {
  if (activeRefresh) {
    // Another request is already refreshing — wait for it
    return activeRefresh
  }

  activeRefresh = refreshFn().finally(() => {
    activeRefresh = null
  })

  return activeRefresh
}
