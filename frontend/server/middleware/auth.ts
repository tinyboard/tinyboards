import { defineEventHandler, getCookie, setCookie } from 'h3'
import { withRefreshLock } from '~/server/utils/refreshLock'

const ME_QUERY = `
  query Me {
    me {
      user {
        id
        name
        displayName
        avatar
        adminLevel
        postCount
        commentCount
        isBanned
        bio
        banner
      }
      unreadNotificationsCount
    }
  }
`

const SUBSCRIBED_BOARDS_QUERY = `
  query SubscribedBoards {
    listBoards(listingType: subscribed, limit: 50) {
      name
      title
      icon
      subscribers
      mode
    }
  }
`

/**
 * Server middleware that reads the auth cookie and fetches user data from the
 * backend during SSR. The result is stored in event.context.auth so the Nuxt
 * route middleware can populate the Pinia store without a second round-trip.
 *
 * If the access token is expired but a valid refresh token exists, this
 * middleware will refresh the session automatically using the shared refresh
 * lock to avoid thundering herd issues.
 */
export default defineEventHandler(async (event) => {
  const accessToken = getCookie(event, 'tb_access')
  const refreshToken = getCookie(event, 'tb_refresh')

  if (!accessToken && !refreshToken) {
    event.context.auth = { isAuthenticated: false, user: null, subscribedBoards: null }
    return
  }

  const config = useRuntimeConfig()
  const gqlEndpoint = config.internalGqlHost

  // Try fetching user data with the current access token
  if (accessToken) {
    const result = await fetchUserData(gqlEndpoint, accessToken)
    if (result) {
      event.context.auth = result
      return
    }
  }

  // Access token missing or expired — attempt refresh
  if (refreshToken) {
    const newAccessToken = await attemptTokenRefresh(event, config.internalApiHost, accessToken, refreshToken)
    if (newAccessToken) {
      const result = await fetchUserData(gqlEndpoint, newAccessToken)
      if (result) {
        event.context.auth = result
        return
      }
    }
  }

  event.context.auth = { isAuthenticated: false, user: null, subscribedBoards: null }
})

async function fetchUserData (
  gqlEndpoint: string,
  accessToken: string,
): Promise<{
  isAuthenticated: boolean
  user: unknown
  unreadNotificationsCount?: number
  subscribedBoards: unknown
} | null> {
  try {
    const meResponse = await $fetch<{ data?: { me?: { user: unknown; unreadNotificationsCount: number } }; errors?: unknown[] }>(gqlEndpoint, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        Cookie: `tb_access=${accessToken}`,
      },
      body: { query: ME_QUERY },
    })

    if (!meResponse?.data?.me?.user) {
      return null
    }

    // Fetch subscribed boards
    let subscribedBoards = null
    try {
      const boardsResponse = await $fetch<{ data?: { listBoards?: unknown[] } }>(gqlEndpoint, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          Cookie: `tb_access=${accessToken}`,
        },
        body: { query: SUBSCRIBED_BOARDS_QUERY },
      })
      subscribedBoards = boardsResponse?.data?.listBoards ?? null
    } catch {
      // Non-critical — sidebar just won't have subscribed boards on first render
    }

    return {
      isAuthenticated: true,
      user: meResponse.data.me.user,
      unreadNotificationsCount: meResponse.data.me.unreadNotificationsCount,
      subscribedBoards,
    }
  } catch {
    return null
  }
}

/**
 * Attempt to refresh the access token using the shared lock to prevent
 * concurrent refresh attempts from invalidating each other.
 */
async function attemptTokenRefresh (
  event: Parameters<Parameters<typeof defineEventHandler>[0]>[0],
  internalApiHost: string,
  accessToken: string | undefined,
  refreshToken: string,
): Promise<string | null> {
  const refreshEndpoint = `${internalApiHost}/api/v2/auth/refresh`

  return withRefreshLock(async () => {
    try {
      const response = await fetch(refreshEndpoint, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'X-Requested-With': 'XMLHttpRequest',
          Cookie: `tb_access=${accessToken ?? ''}; tb_refresh=${refreshToken}`,
        },
      })

      if (!response.ok) {
        return null
      }

      // Forward Set-Cookie headers from the backend to the client
      const setCookieHeaders = response.headers.getSetCookie?.() ?? []
      for (const cookieHeader of setCookieHeaders) {
        event.node.res.appendHeader('Set-Cookie', cookieHeader)
      }

      // Extract the new access token from Set-Cookie headers
      for (const header of setCookieHeaders) {
        const match = header.match(/tb_access=([^;]+)/)
        if (match) {
          return match[1]
        }
      }
    } catch {
      // Refresh failed
    }

    return null
  })
}
