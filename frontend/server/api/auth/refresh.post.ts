import { defineEventHandler, setResponseStatus } from 'h3'
import { proxyAuthRequest } from '~/server/utils/authProxy'

/**
 * POST /api/auth/refresh
 * Proxies to backend POST /api/v2/auth/refresh.
 * Called automatically by the GraphQL BFF when it receives a 401.
 * Forwards the refresh_token cookie to the backend for token rotation.
 */
export default defineEventHandler(async (event) => {
  const { status, data } = await proxyAuthRequest(event, '/refresh', { includeBody: false })
  setResponseStatus(event, status)
  return data
})
