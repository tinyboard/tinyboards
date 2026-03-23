import DOMPurify from 'dompurify'

// Register hook once (idempotent — DOMPurify deduplicates)
let hookRegistered = false

function registerStyleHook (): void {
  if (hookRegistered || !import.meta.client) return
  hookRegistered = true

  DOMPurify.addHook('uponSanitizeAttribute', (_node, data) => {
    if (data.attrName === 'style') {
      // Only allow safe CSS properties: color and background-color
      const safe = data.attrValue
        .split(';')
        .map(prop => prop.trim())
        .filter(prop => /^\s*(color|background-color)\s*:/i.test(prop))
        .join('; ')
      data.attrValue = safe || ''
    }
  })
}

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

  registerStyleHook()

  return DOMPurify.sanitize(dirty, {
    ALLOWED_TAGS: [
      'a', 'b', 'i', 'em', 'strong', 'u', 'p', 'br', 'ul', 'ol', 'li',
      'h1', 'h2', 'h3', 'h4', 'h5', 'h6', 'blockquote', 'pre', 'code',
      'img', 'table', 'thead', 'tbody', 'tr', 'th', 'td', 'hr', 'span',
      'del', 'sup', 'sub', 'details', 'summary', 'mark', 'div',
      'iframe',
    ],
    ALLOWED_ATTR: [
      'href', 'src', 'alt', 'title', 'class', 'id', 'target', 'rel',
      'width', 'height', 'loading', 'style',
      // Forum quote attributes
      'data-author', 'data-post-number',
      // YouTube iframe
      'allowfullscreen', 'frameborder', 'allow',
      // Code block language
      'data-language',
    ],
    ALLOW_DATA_ATTR: false,
    // Allow YouTube embeds
    ADD_TAGS: ['iframe'],
    ADD_ATTR: ['allowfullscreen', 'frameborder', 'allow'],
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
