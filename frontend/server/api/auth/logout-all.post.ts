import { defineEventHandler, setResponseStatus } from 'h3'
import { proxyAuthRequest } from '~/server/utils/authProxy'

/**
 * POST /api/auth/logout-all
 * Proxies to backend POST /api/v2/auth/logout-all.
 * Clears all sessions for the authenticated user.
 */
export default defineEventHandler(async (event) => {
  const { status, data } = await proxyAuthRequest(event, '/logout-all', { includeBody: false })
  setResponseStatus(event, status)
  return data
})
