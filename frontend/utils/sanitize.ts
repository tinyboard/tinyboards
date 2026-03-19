import DOMPurify from 'dompurify'

/**
 * Sanitize HTML content using DOMPurify.
 * Must be called before any v-html binding. Fixes BUG-012.
 *
 * Only usable on the client side. For server-side rendering,
 * content should already be sanitized by the backend.
 */
export function sanitizeHtml (dirty: string): string {
  if (!import.meta.client) {
    // On the server, return as-is — backend sanitizes with ammonia
    return dirty
  }

  return DOMPurify.sanitize(dirty, {
    ALLOWED_TAGS: [
      'a', 'b', 'i', 'em', 'strong', 'p', 'br', 'ul', 'ol', 'li',
      'h1', 'h2', 'h3', 'h4', 'h5', 'h6', 'blockquote', 'pre', 'code',
      'img', 'table', 'thead', 'tbody', 'tr', 'th', 'td', 'hr', 'span',
      'del', 'sup', 'sub', 'details', 'summary',
    ],
    ALLOWED_ATTR: [
      'href', 'src', 'alt', 'title', 'class', 'id', 'target', 'rel',
      'width', 'height', 'loading',
    ],
    ALLOW_DATA_ATTR: false,
  })
}

/**
 * Strip all HTML tags, returning plain text.
 * Useful for generating excerpts or meta descriptions.
 */
export function stripHtml (html: string): string {
  if (!import.meta.client) {
    return html.replace(/<[^>]*>/g, '')
  }

  return DOMPurify.sanitize(html, { ALLOWED_TAGS: [] })
}
