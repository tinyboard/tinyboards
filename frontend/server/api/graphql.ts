import { defineEventHandler, readBody, getCookie, setCookie, createError, getHeader } from 'h3'
import { withRefreshLock } from '~/server/utils/refreshLock'

interface GraphQLRequest {
  query: string
  variables?: Record<string, unknown>
  operationName?: string
}

interface GraphQLResponse {
  data?: unknown
  errors?: Array<{ message: string; extensions?: Record<string, unknown> }>
}

/**
 * BFF proxy for all GraphQL traffic.
 * - Reads httpOnly auth cookie server-side (tb_access)
 * - Forwards requests to the backend GraphQL endpoint
 * - Handles 401 → refresh → retry flow (with deduplication)
 * - Validates X-Requested-With header (CSRF protection)
 *
 * Auth mutations (login/register) are no longer handled here.
 * They go through the dedicated BFF routes in server/api/auth/.
 */
export default defineEventHandler(async (event) => {
  const config = useRuntimeConfig()
  const gqlEndpoint = config.internalGqlHost

  // CSRF check: all client requests must include this header
  const requestedWith = getHeader(event, 'x-requested-with')
  if (requestedWith !== 'XMLHttpRequest') {
    throw createError({
      statusCode: 403,
      statusMessage: 'Missing or invalid X-Requested-With header',
    })
  }

  const body = await readBody<GraphQLRequest>(event)
  if (!body?.query) {
    throw createError({
      statusCode: 400,
      statusMessage: 'Missing GraphQL query',
    })
  }

  const accessToken = getCookie(event, 'tb_access')

  // Attempt the request
  const result = await forwardGraphQLRequest(gqlEndpoint, body, accessToken)

  // Check for authentication errors (HTTP 401 or GraphQL-level auth errors)
  if (result.httpStatus === 401 || hasAuthError(result.data)) {
    const newAccessToken = await attemptTokenRefresh(event, config.internalApiHost)
    if (newAccessToken) {
      // Retry original request with the new token
      const retry = await forwardGraphQLRequest(gqlEndpoint, body, newAccessToken)
      return retry.data
    }

    // Refresh failed — clear cookies and return the original error
    clearAuthCookies(event)
  }

  return result.data
})

interface ForwardResult {
  httpStatus: number
  data: GraphQLResponse
}

async function forwardGraphQLRequest (
  endpoint: string,
  body: GraphQLRequest,
  accessToken?: string | null,
): Promise<ForwardResult> {
  const headers: Record<string, string> = {
    'Content-Type': 'application/json',
  }

  if (accessToken) {
    headers.Cookie = `tb_access=${accessToken}`
  }

  try {
    const data = await $fetch<GraphQLResponse>(endpoint, {
      method: 'POST',
      headers,
      body,
    })
    return { httpStatus: 200, data }
  } catch (err: unknown) {
    // $fetch throws on non-2xx responses — extract the status and body
    const fetchErr = err as { status?: number; data?: GraphQLResponse }
    if (fetchErr.status && fetchErr.status >= 400) {
      return {
        httpStatus: fetchErr.status,
        data: fetchErr.data ?? { errors: [{ message: 'Unauthorized' }] },
      }
    }
    throw err
  }
}

function hasAuthError (response: GraphQLResponse): boolean {
  if (!response.errors) { return false }
  return response.errors.some(
    err =>
      err.message.toLowerCase().includes('unauthorized') ||
      err.message.toLowerCase().includes('not authenticated') ||
      err.extensions?.code === 'UNAUTHENTICATED',
  )
}

/**
 * Attempt to refresh the access token, using a lock to prevent concurrent
 * refreshes from invalidating each other's tokens (thundering herd).
 * Returns the new access token string on success, or null on failure.
 */
async function attemptTokenRefresh (
  event: Parameters<Parameters<typeof defineEventHandler>[0]>[0],
  internalApiHost: string,
): Promise<string | null> {
  const refreshToken = getCookie(event, 'tb_refresh')
  const accessToken = getCookie(event, 'tb_access')
  if (!refreshToken) { return null }

  const refreshEndpoint = `${internalApiHost}/api/v2/auth/refresh`

  const newAccessToken = await withRefreshLock(async () => {
    try {
      const response = await fetch(refreshEndpoint, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'X-Requested-With': 'XMLHttpRequest',
          Cookie: `tb_access=${accessToken ?? ''}; tb_refresh=${refreshToken}`,
        },
      })

      if (!response.ok) {
        return null
      }

      // Forward Set-Cookie headers from the backend response to the client
      const setCookieHeaders = response.headers.getSetCookie?.() ?? []
      for (const cookieHeader of setCookieHeaders) {
        event.node.res.appendHeader('Set-Cookie', cookieHeader)
      }

      // Extract the new access token from Set-Cookie headers so we can use
      // it immediately for the retry (the h3 cookie jar won't reflect it yet)
      for (const header of setCookieHeaders) {
        const match = header.match(/tb_access=([^;]+)/)
        if (match) {
          return match[1]
        }
      }

      return null
    } catch {
      return null
    }
  })

  return newAccessToken
}

function clearAuthCookies (
  event: Parameters<Parameters<typeof defineEventHandler>[0]>[0],
): void {
  const secure = useRuntimeConfig().public.useHttps === true
  setCookie(event, 'tb_access', '', { maxAge: 0, path: '/', httpOnly: true, secure, sameSite: 'lax' })
  setCookie(event, 'tb_refresh', '', { maxAge: 0, path: '/', httpOnly: true, secure, sameSite: 'lax' })
}
