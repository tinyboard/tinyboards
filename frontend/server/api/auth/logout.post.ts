import { defineEventHandler, setResponseStatus } from 'h3'
import { proxyAuthRequest } from '~/server/utils/authProxy'

/**
 * POST /api/auth/logout
 * Proxies to backend POST /api/v2/auth/logout.
 * The backend clears both auth cookies via Set-Cookie headers.
 */
export default defineEventHandler(async (event) => {
  const { status, data } = await proxyAuthRequest(event, '/logout', { includeBody: false })
  setResponseStatus(event, status)
  return data
})
