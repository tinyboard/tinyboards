import { defineEventHandler, setResponseStatus } from 'h3'
import { proxyAuthRequest } from '~/server/utils/authProxy'

/**
 * POST /api/auth/email-verify
 * Proxies to backend POST /api/v2/auth/email/verify.
 */
export default defineEventHandler(async (event) => {
  const { status, data } = await proxyAuthRequest(event, '/email/verify')
  setResponseStatus(event, status)
  return data
})
