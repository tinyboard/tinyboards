import { defineEventHandler, setResponseStatus } from 'h3'
import { proxyAuthRequest } from '~/server/utils/authProxy'

/**
 * POST /api/auth/password-reset-complete
 * Proxies to backend POST /api/v2/auth/password-reset/complete.
 */
export default defineEventHandler(async (event) => {
  const { status, data } = await proxyAuthRequest(event, '/password-reset/complete')
  setResponseStatus(event, status)
  return data
})
