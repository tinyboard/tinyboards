import { ref } from 'vue'
import type { Ref } from 'vue'
import type { ApiResponse, ApiError } from '~/types/api'
import { useAuthStore } from '~/stores/auth'

interface GraphQLRequestOptions {
  variables?: Record<string, unknown>
  operationName?: string
}

interface UseGraphQLReturn<T> {
  data: Ref<T | null>
  error: Ref<ApiError | null>
  loading: Ref<boolean>
  execute: (query: string, options?: GraphQLRequestOptions) => Promise<T | null>
}

/**
 * Composable for making GraphQL requests through the BFF proxy.
 * All requests go to `/api/graphql` (the Nuxt server route), never directly to the backend.
 *
 * Includes the required X-Requested-With header for CSRF protection.
 */
export function useGraphQL<T = unknown> (): UseGraphQLReturn<T> {
  const data: Ref<T | null> = ref(null)
  const error: Ref<ApiError | null> = ref(null)
  const loading = ref(false)

  async function execute (query: string, options?: GraphQLRequestOptions): Promise<T | null> {
    loading.value = true
    error.value = null

    try {
      const headers: Record<string, string> = {
        'Content-Type': 'application/json',
        'X-Requested-With': 'XMLHttpRequest',
      }

      // During SSR, forward the browser's cookies to the internal BFF route
      // so that auth-dependent queries work server-side
      if (import.meta.server) {
        const requestHeaders = useRequestHeaders(['cookie'])
        if (requestHeaders.cookie) {
          headers.cookie = requestHeaders.cookie
        }
      }

      const response = await $fetch<ApiResponse<T>>('/api/graphql', {
        method: 'POST',
        headers,
        body: {
          query,
          variables: options?.variables,
          operationName: options?.operationName,
        },
      })

      if (response.errors && response.errors.length > 0) {
        error.value = response.errors[0]
        data.value = null

        // If the BFF returned an auth error, the refresh already failed server-side
        // and cookies were cleared. Sync the client-side store so the UI reflects
        // the logged-out state instead of showing stale auth.
        if (import.meta.client && isAuthError(response.errors)) {
          const authStore = useAuthStore()
          if (authStore.isLoggedIn) {
            authStore.clearUser()
          }
        }

        return null
      }

      data.value = response.data
      return response.data
    } catch (err: unknown) {
      const message = err instanceof Error ? err.message : 'An unexpected error occurred'
      error.value = { message }
      data.value = null
      return null
    } finally {
      loading.value = false
    }
  }

  return { data, error, loading, execute }
}

/**
 * Convenience wrapper for GraphQL mutations.
 * Identical to useGraphQL but named for semantic clarity.
 */
export function useGraphQLMutation<T = unknown> (): UseGraphQLReturn<T> {
  return useGraphQL<T>()
}

function isAuthError (errors: ApiError[]): boolean {
  return errors.some(
    err =>
      err.message?.toLowerCase().includes('unauthorized') ||
      err.message?.toLowerCase().includes('not authenticated') ||
      err.extensions?.code === 'UNAUTHENTICATED',
  )
}
