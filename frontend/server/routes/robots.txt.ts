import { loadConfig } from '~/server/utils/config'

/**
 * Serves robots.txt.
 * Disallows crawling of hidden boards and private/admin areas.
 * Fetches the list of hidden boards from the backend GraphQL API.
 */
export default defineEventHandler(async (event) => {
  const config = loadConfig()
  const runtimeConfig = useRuntimeConfig()
  const domain = runtimeConfig.public.domain || config.hostname || 'localhost'
  const protocol = runtimeConfig.public.useHttps ? 'https' : 'http'

  const lines: string[] = [
    'User-agent: *',
    'Allow: /',
    '',
    '# Private areas',
    'Disallow: /settings',
    'Disallow: /admin',
    'Disallow: /inbox',
    'Disallow: /api/',
    '',
  ]

  // Fetch hidden boards from backend to disallow crawling
  try {
    const backendUrl = runtimeConfig.backendUrl || runtimeConfig.internalGqlHost?.replace('/api/v2/graphql', '')
    if (backendUrl) {
      const response = await $fetch<{ data?: { listBoards: { name: string }[] } }>(`${backendUrl}/api/v2/graphql`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: {
          query: 'query { listBoards(limit: 500) { name isHidden } }',
        },
      }).catch(() => null)

      if (response?.data?.listBoards) {
        const hiddenBoards = response.data.listBoards.filter((b: { name: string; isHidden?: boolean }) => (b as { isHidden?: boolean }).isHidden)
        if (hiddenBoards.length > 0) {
          lines.push('# Hidden boards')
          for (const board of hiddenBoards) {
            lines.push(`Disallow: /b/${board.name}`)
          }
          lines.push('')
        }
      }
    }
  } catch {
    // Silently continue without hidden board rules
  }

  lines.push(`Sitemap: ${protocol}://${domain}/sitemap.xml`)

  setResponseHeader(event, 'Content-Type', 'text/plain; charset=utf-8')
  setResponseHeader(event, 'Cache-Control', 'public, max-age=3600')
  return lines.join('\n')
})
