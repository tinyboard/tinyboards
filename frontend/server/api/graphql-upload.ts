import { defineEventHandler, getCookie, setCookie, createError, getHeader, readMultipartFormData } from 'h3'
import { withRefreshLock } from '~/server/utils/refreshLock'

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
  const formData = buildFormData(parts)

  const accessToken = getCookie(event, 'tb_access')

  try {
    const { status, data } = await forwardUpload(gqlEndpoint, formData, accessToken)

    if (status === 401 || hasAuthError(data as { errors?: Array<{ message: string; extensions?: Record<string, unknown> }> })) {
      const newAccessToken = await attemptTokenRefresh(event, config.internalApiHost)
      if (newAccessToken) {
        // Rebuild form data for retry (original may have been consumed)
        const retryFormData = buildFormData(parts)
        const retry = await forwardUpload(gqlEndpoint, retryFormData, newAccessToken)
        return rewriteMediaUrls(retry.data, config.public.domain as string)
      }

      clearAuthCookies(event)
    }

    return rewriteMediaUrls(data, config.public.domain as string)
  } catch (err) {
    throw createError({
      statusCode: 500,
      statusMessage: 'Upload proxy failed',
    })
  }
})

/**
 * Rewrite absolute media URLs to relative paths so they resolve correctly
 * in all deployment configurations (with or without nginx).
 */
function rewriteMediaUrls (data: unknown, domain: string): unknown {
  if (!data) return data
  const json = JSON.stringify(data)
  const escaped = domain.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
  const pattern = new RegExp(`https?://${escaped}(?::\\d+)?/media/`, 'g')
  return JSON.parse(json.replace(pattern, '/media/'))
}

function buildFormData (parts: { name?: string; filename?: string; data: Buffer; type?: string }[]): FormData {
  const formData = new FormData()
  for (const part of parts) {
    const fieldName = part.name ?? ''
    if (part.filename) {
      const blob = new Blob([new Uint8Array(part.data)], { type: part.type || 'application/octet-stream' })
      formData.append(fieldName, blob, part.filename)
    } else {
      formData.append(fieldName, part.data.toString('utf-8'))
    }
  }
  return formData
}

async function forwardUpload (
  endpoint: string,
  formData: FormData,
  accessToken?: string | null,
): Promise<{ status: number; data: unknown }> {
  const headers: Record<string, string> = {}
  if (accessToken) {
    headers.Cookie = `tb_access=${accessToken}`
  }

  const response = await fetch(endpoint, {
    method: 'POST',
    headers,
    body: formData,
  })

  const data = await response.json()
  return { status: response.status, data }
}

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
): Promise<string | null> {
  const refreshToken = getCookie(event, 'tb_refresh')
  const accessToken = getCookie(event, 'tb_access')
  if (!refreshToken) { return null }

  const refreshEndpoint = `${internalApiHost}/api/v2/auth/refresh`

  return withRefreshLock(async () => {
    try {
      const response = await fetch(refreshEndpoint, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'X-Requested-With': 'XMLHttpRequest',
          Cookie: `tb_access=${accessToken ?? ''}; tb_refresh=${refreshToken}`,
        },
      })

      if (!response.ok) { return null }

      const setCookieHeaders = response.headers.getSetCookie?.() ?? []
      for (const cookieHeader of setCookieHeaders) {
        event.node.res.appendHeader('Set-Cookie', cookieHeader)
      }

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
}

function clearAuthCookies (
  event: Parameters<Parameters<typeof defineEventHandler>[0]>[0],
): void {
  const secure = useRuntimeConfig().public.useHttps === true
  setCookie(event, 'tb_access', '', { maxAge: 0, path: '/', httpOnly: true, secure, sameSite: 'lax' })
  setCookie(event, 'tb_refresh', '', { maxAge: 0, path: '/', httpOnly: true, secure, sameSite: 'lax' })
}
