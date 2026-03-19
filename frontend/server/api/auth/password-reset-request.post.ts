import { defineEventHandler, setResponseStatus } from 'h3'
import { proxyAuthRequest } from '~/server/utils/authProxy'

/**
 * POST /api/auth/password-reset-request
 * Proxies to backend POST /api/v2/auth/password-reset/request.
 */
export default defineEventHandler(async (event) => {
  const { status, data } = await proxyAuthRequest(event, '/password-reset/request')
  setResponseStatus(event, status)
  return data
})
