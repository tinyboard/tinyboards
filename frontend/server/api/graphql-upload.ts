import { defineEventHandler, getCookie, setCookie, createError, getHeader, readMultipartFormData } from 'h3'

/**
 * BFF proxy for GraphQL file uploads (multipart/form-data).
 *
 * Accepts multipart form data following the GraphQL multipart request spec:
 * - `operations`: JSON string of { query, variables } (files replaced with null)
 * - `map`: JSON mapping of field names to variable paths (e.g. {"file": ["variables.file"]})
 * - actual file fields referenced by the map keys
 *
 * This endpoint forwards the upload as multipart to the backend GraphQL endpoint,
 * handles auth cookies, and returns the GraphQL response.
 */
export default defineEventHandler(async (event) => {
  const config = useRuntimeConfig()
  const gqlEndpoint = config.internalGqlHost

  // CSRF check
  const requestedWith = getHeader(event, 'x-requested-with')
  if (requestedWith !== 'XMLHttpRequest') {
    throw createError({
      statusCode: 403,
      statusMessage: 'Missing or invalid X-Requested-With header',
    })
  }

  const parts = await readMultipartFormData(event)
  if (!parts || parts.length === 0) {
    throw createError({
      statusCode: 400,
      statusMessage: 'No multipart data received',
    })
  }

  // Reconstruct multipart form data for the backend
  const formData = new FormData()

  for (const part of parts) {
    const fieldName = part.name ?? ''

    if (part.filename) {
      // File field — create a Blob with proper content type
      const blob = new Blob([part.data], { type: part.type || 'application/octet-stream' })
      formData.append(fieldName, blob, part.filename)
    } else {
      // Text field (operations, map, etc.)
      formData.append(fieldName, part.data.toString('utf-8'))
    }
  }

  const accessToken = getCookie(event, 'tb_access')

  const headers: Record<string, string> = {}
  if (accessToken) {
    headers.Cookie = `tb_access=${accessToken}`
  }

  try {
    const response = await fetch(gqlEndpoint, {
      method: 'POST',
      headers,
      body: formData,
    })

    const data = await response.json()

    if (response.status === 401 || hasAuthError(data)) {
      const refreshed = await attemptTokenRefresh(event, config.internalApiHost)
      if (refreshed) {
        const newAccessToken = getCookie(event, 'tb_access')
        const retryHeaders: Record<string, string> = {}
        if (newAccessToken) {
          retryHeaders.Cookie = `tb_access=${newAccessToken}`
        }
        const retryResponse = await fetch(gqlEndpoint, {
          method: 'POST',
          headers: retryHeaders,
          body: formData,
        })
        return await retryResponse.json()
      }

      clearAuthCookies(event)
    }

    return data
  } catch (err) {
    throw createError({
      statusCode: 500,
      statusMessage: 'Upload proxy failed',
    })
  }
})

function hasAuthError (response: { errors?: Array<{ message: string; extensions?: Record<string, unknown> }> }): boolean {
  if (!response.errors) { return false }
  return response.errors.some(
    err =>
      err.message.toLowerCase().includes('unauthorized') ||
      err.message.toLowerCase().includes('not authenticated') ||
      err.extensions?.code === 'UNAUTHENTICATED',
  )
}

async function attemptTokenRefresh (
  event: Parameters<Parameters<typeof defineEventHandler>[0]>[0],
  internalApiHost: string,
): Promise<boolean> {
  const refreshToken = getCookie(event, 'tb_refresh')
  const accessToken = getCookie(event, 'tb_access')
  if (!refreshToken) { return false }

  const refreshEndpoint = `${internalApiHost}/api/v2/auth/refresh`

  try {
    const response = await fetch(refreshEndpoint, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'X-Requested-With': 'XMLHttpRequest',
        Cookie: `tb_access=${accessToken ?? ''}; tb_refresh=${refreshToken}`,
      },
    })

    if (!response.ok) { return false }

    const setCookieHeaders = response.headers.getSetCookie?.() ?? []
    for (const cookieHeader of setCookieHeaders) {
      event.node.res.appendHeader('Set-Cookie', cookieHeader)
    }

    return setCookieHeaders.length > 0
  } catch {
    return false
  }
}

function clearAuthCookies (
  event: Parameters<Parameters<typeof defineEventHandler>[0]>[0],
): void {
  const secure = useRuntimeConfig().public.useHttps === true
  setCookie(event, 'tb_access', '', { maxAge: 0, path: '/', httpOnly: true, secure, sameSite: 'lax' })
  setCookie(event, 'tb_refresh', '', { maxAge: 0, path: '/', httpOnly: true, secure, sameSite: 'lax' })
}
