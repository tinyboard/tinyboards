import type { H3Event } from 'h3'
import { readBody, appendResponseHeader, getHeader } from 'h3'

/**
 * Proxy a request to a backend auth REST endpoint.
 * Forwards cookies from the client, includes the CSRF header,
 * and forwards Set-Cookie headers from the backend response back to the client.
 */
export async function proxyAuthRequest (
  event: H3Event,
  path: string,
  options?: { includeBody?: boolean },
): Promise<{ status: number; data: unknown }> {
  const config = useRuntimeConfig()
  const backendUrl = `${config.internalApiHost}/api/v2/auth${path}`

  const headers: Record<string, string> = {
    'Content-Type': 'application/json',
    'X-Requested-With': 'XMLHttpRequest',
  }

  // Forward cookies from the client request to the backend
  const cookie = getHeader(event, 'cookie')
  if (cookie) {
    headers.Cookie = cookie
  }

  // Forward User-Agent for session tracking
  const userAgent = getHeader(event, 'user-agent')
  if (userAgent) {
    headers['User-Agent'] = userAgent
  }

  // Forward X-Forwarded-For for IP tracking
  const forwarded = getHeader(event, 'x-forwarded-for')
  if (forwarded) {
    headers['X-Forwarded-For'] = forwarded
  }

  const fetchOptions: RequestInit = {
    method: 'POST',
    headers,
  }

  if (options?.includeBody !== false) {
    const body = await readBody(event)
    if (body) {
      fetchOptions.body = JSON.stringify(body)
    }
  }

  const response = await fetch(backendUrl, fetchOptions)

  // Forward Set-Cookie headers from the backend response back to the client
  const setCookieHeaders = response.headers.getSetCookie?.() ?? []
  for (const cookieHeader of setCookieHeaders) {
    appendResponseHeader(event, 'Set-Cookie', cookieHeader)
  }

  const data = await response.json()

  return { status: response.status, data }
}
