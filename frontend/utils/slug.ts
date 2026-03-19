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
  title: string
  isThread?: boolean
  board?: { name: string } | null
}

/**
 * Build a post URL path from a post object or explicit components.
 * Uses the post's isThread field, defaulting to 'feed'.
 */
export function postUrl (post: PostLike, postType?: 'feed' | 'threads'): string {
  const section = postType ?? (post.isThread ? 'threads' : 'feed')
  const slug = post.title ? `/${slugify(post.title)}` : ''
  const boardName = post.board?.name ?? 'unknown'
  return `/b/${boardName}/${section}/${post.id}${slug}`
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
