import { sanitizeHtml } from '~/utils/sanitize'

/**
 * Content conversion utilities for rendering user-generated content.
 * Handles markdown-to-HTML conversion and sanitization.
 */
export function useContentConverter () {
  function toSafeHTML (content: string): string {
    return sanitizeHtml(content)
  }

  function truncate (content: string, maxLength: number): string {
    if (content.length <= maxLength) { return content }
    return content.substring(0, maxLength).trim() + '...'
  }

  function stripHTML (html: string): string {
    if (typeof document !== 'undefined') {
      const div = document.createElement('div')
      div.innerHTML = html
      return div.textContent ?? ''
    }
    // SSR fallback: basic tag stripping
    return html.replace(/<[^>]*>/g, '')
  }

  function extractFirstImage (html: string): string | null {
    const match = html.match(/<img[^>]+src="([^"]+)"/)
    return match ? match[1] : null
  }

  return {
    toSafeHTML,
    truncate,
    stripHTML,
    extractFirstImage,
  }
}
