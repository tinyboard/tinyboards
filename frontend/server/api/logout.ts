import { defineEventHandler, setCookie } from 'h3'

/**
 * Server-side logout endpoint.
 * Clears the httpOnly auth cookies that the client cannot access directly.
 */
export default defineEventHandler((event) => {
  const secure = useRuntimeConfig().public.useHttps === true
  setCookie(event, 'tb_access', '', { maxAge: 0, path: '/', httpOnly: true, secure, sameSite: 'lax' })
  setCookie(event, 'tb_refresh', '', { maxAge: 0, path: '/', httpOnly: true, secure, sameSite: 'lax' })

  return { success: true }
})
