import { defineEventHandler, setResponseStatus } from 'h3'
import { proxyAuthRequest } from '~/server/utils/authProxy'

/**
 * POST /api/auth/register
 * Proxies to backend POST /api/v2/auth/register.
 * On success (open/invite modes), the backend sets httpOnly auth cookies.
 */
export default defineEventHandler(async (event) => {
  const { status, data } = await proxyAuthRequest(event, '/register')
  setResponseStatus(event, status)
  return data
})
