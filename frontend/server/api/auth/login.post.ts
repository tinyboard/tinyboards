import { defineEventHandler, setResponseStatus } from 'h3'
import { proxyAuthRequest } from '~/server/utils/authProxy'

/**
 * POST /api/auth/login
 * Proxies to backend POST /api/v2/auth/login.
 * Forwards cookies both directions — the backend sets httpOnly auth cookies.
 */
export default defineEventHandler(async (event) => {
  const { status, data } = await proxyAuthRequest(event, '/login')
  setResponseStatus(event, status)
  return data
})
