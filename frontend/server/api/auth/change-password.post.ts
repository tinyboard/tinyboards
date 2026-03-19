import { defineEventHandler, setResponseStatus } from 'h3'
import { proxyAuthRequest } from '~/server/utils/authProxy'

/**
 * POST /api/auth/change-password
 * Proxies to backend POST /api/v2/auth/change-password.
 * Requires authentication (access_token cookie).
 */
export default defineEventHandler(async (event) => {
  const { status, data } = await proxyAuthRequest(event, '/change-password')
  setResponseStatus(event, status)
  return data
})
