import { defineEventHandler, getCookie, getHeader } from 'h3'

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
    }
  }
`

/**
 * Server middleware that reads the auth cookie and fetches user data from the
 * backend during SSR. The result is stored in event.context.auth so the Nuxt
 * route middleware can populate the Pinia store without a second round-trip.
 */
export default defineEventHandler(async (event) => {
  const accessToken = getCookie(event, 'tb_access')

  if (!accessToken) {
    event.context.auth = { isAuthenticated: false, user: null, subscribedBoards: null }
    return
  }

  const config = useRuntimeConfig()
  const gqlEndpoint = config.internalGqlHost

  try {
    // Fetch user data directly from the backend GraphQL endpoint
    const meResponse = await $fetch<{ data?: { me?: { user: unknown; unreadNotificationsCount: number } }; errors?: unknown[] }>(gqlEndpoint, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        Cookie: `tb_access=${accessToken}`,
      },
      body: { query: ME_QUERY },
    })

    if (meResponse?.data?.me?.user) {
      // Fetch subscribed boards in parallel with the user data already confirmed
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

      event.context.auth = {
        isAuthenticated: true,
        user: meResponse.data.me.user,
        unreadNotificationsCount: meResponse.data.me.unreadNotificationsCount,
        subscribedBoards,
      }
      return
    }
  } catch {
    // Token is invalid or expired — the user will see unauthenticated state
  }

  event.context.auth = { isAuthenticated: false, user: null, subscribedBoards: null }
})
