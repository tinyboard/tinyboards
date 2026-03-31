/**
 * Generate a URL-safe slug from a title string.
 */
export function slugify (text: string): string {
  return text
    .toLowerCase()
    .trim()
    .replace(/[^\w\s-]/g, '') // Remove non-word chars (except spaces and hyphens)
    .replace(/[\s_]+/g, '-') // Replace spaces and underscores with hyphens
    .replace(/-+/g, '-') // Collapse multiple hyphens
    .replace(/^-+|-+$/g, '') // Trim leading/trailing hyphens
}

interface PostLike {
  id: string
  title?: string | null
  slug?: string | null
  board?: { name: string } | null
}

/**
 * Build a post URL path from a post object.
 * Canonical format: /b/{board}/{id}/{slug}
 */
export function postUrl (post: PostLike): string {
  const postSlug = post.slug || (post.title ? slugify(post.title) : '')
  const slugPart = postSlug ? `/${postSlug}` : ''
  const boardName = post.board?.name ?? 'unknown'
  return `/b/${boardName}/${post.id}${slugPart}`
}

/**
 * Build a user profile URL.
 */
export function userUrl (username: string): string {
  return `/@${username}`
}

/**
 * Build a board URL.
 */
export function boardUrl (boardName: string): string {
  return `/b/${boardName}`
}
