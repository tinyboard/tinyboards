import { defineEventHandler, setResponseStatus } from 'h3'
import { proxyAuthRequest } from '~/server/utils/authProxy'

/**
 * POST /api/auth/email-request-verification
 * Proxies to backend POST /api/v2/auth/email/request-verification.
 */
export default defineEventHandler(async (event) => {
  const { status, data } = await proxyAuthRequest(event, '/email/request-verification')
  setResponseStatus(event, status)
  return data
})
