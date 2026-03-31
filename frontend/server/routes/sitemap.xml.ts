import { loadConfig } from '~/server/utils/config'

/**
 * Generates a dynamic sitemap.xml listing public boards and recent public posts.
 * Caps at 1000 entries total. Excludes NSFW boards when enable_nsfw is false in site config.
 */
export default defineEventHandler(async (event) => {
  const config = loadConfig()
  const runtimeConfig = useRuntimeConfig()
  const domain = runtimeConfig.public.domain || config.hostname || 'localhost'
  const protocol = runtimeConfig.public.useHttps ? 'https' : 'http'
  const baseUrl = `${protocol}://${domain}`
  const backendUrl = runtimeConfig.backendUrl || runtimeConfig.internalGqlHost?.replace('/api/v2/graphql', '')

  const urls: { loc: string; lastmod?: string; priority?: string; changefreq?: string }[] = [
    { loc: `${baseUrl}/`, priority: '1.0', changefreq: 'daily' },
    { loc: `${baseUrl}/home`, priority: '0.9', changefreq: 'hourly' },
    { loc: `${baseUrl}/all`, priority: '0.9', changefreq: 'hourly' },
    { loc: `${baseUrl}/boards`, priority: '0.8', changefreq: 'daily' },
  ]

  if (!backendUrl) {
    return buildXml(urls, event)
  }

  try {
    // Fetch site config to check NSFW setting
    const siteResponse = await $fetch<{ data?: { site: { enableNsfw: boolean } } }>(`${backendUrl}/api/v2/graphql`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: { query: '{ site { enableNsfw } }' },
    }).catch(() => null)

    const nsfwEnabled = siteResponse?.data?.site?.enableNsfw ?? false

    // Fetch public boards
    const boardsResponse = await $fetch<{ data?: { listBoards: { name: string; isHidden: boolean; isNSFW: boolean; updatedAt: string }[] } }>(`${backendUrl}/api/v2/graphql`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: { query: '{ listBoards(limit: 200) { name isHidden isNSFW updatedAt } }' },
    }).catch(() => null)

    if (boardsResponse?.data?.listBoards) {
      for (const board of boardsResponse.data.listBoards) {
        if (board.isHidden) continue
        if (board.isNSFW && !nsfwEnabled) continue
        urls.push({
          loc: `${baseUrl}/b/${board.name}`,
          lastmod: board.updatedAt ? new Date(board.updatedAt).toISOString().split('T')[0] : undefined,
          priority: '0.7',
          changefreq: 'daily',
        })
      }
    }

    // Fetch recent public posts (cap total entries at 1000)
    const remaining = 1000 - urls.length
    if (remaining > 0) {
      const postsResponse = await $fetch<{ data?: { listPosts: { id: string; slug: string; createdAt: string; board: { name: string } | null }[] } }>(`${backendUrl}/api/v2/graphql`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: {
          query: `{ listPosts(limit: ${Math.min(remaining, 500)}, sort: new, listingType: all) { id slug createdAt board { name } } }`,
        },
      }).catch(() => null)

      if (postsResponse?.data?.listPosts) {
        for (const post of postsResponse.data.listPosts) {
          if (!post.board) continue
          const slugPart = post.slug ? `/${post.slug}` : ''
          urls.push({
            loc: `${baseUrl}/b/${post.board.name}/${post.id}${slugPart}`,
            lastmod: post.createdAt ? new Date(post.createdAt).toISOString().split('T')[0] : undefined,
            priority: '0.5',
            changefreq: 'weekly',
          })
        }
      }
    }
  } catch {
    // Serve sitemap with static entries if backend is unavailable
  }

  return buildXml(urls, event)
})

function buildXml (
  urls: { loc: string; lastmod?: string; priority?: string; changefreq?: string }[],
  event: Parameters<typeof setResponseHeader>[0],
): string {
  setResponseHeader(event, 'Content-Type', 'application/xml; charset=utf-8')
  setResponseHeader(event, 'Cache-Control', 'public, max-age=3600')

  const entries = urls.map((u) => {
    let entry = `  <url>\n    <loc>${escapeXml(u.loc)}</loc>`
    if (u.lastmod) entry += `\n    <lastmod>${u.lastmod}</lastmod>`
    if (u.changefreq) entry += `\n    <changefreq>${u.changefreq}</changefreq>`
    if (u.priority) entry += `\n    <priority>${u.priority}</priority>`
    entry += '\n  </url>'
    return entry
  })

  return `<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
${entries.join('\n')}
</urlset>`
}

function escapeXml (str: string): string {
  return str
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&apos;')
}
