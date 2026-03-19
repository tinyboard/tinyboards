import DOMPurify from 'dompurify'
import { defineNuxtPlugin } from '#app'

export default defineNuxtPlugin(() => {
  // DOMPurify only works in the browser; provide a no-op on the server
  // since all v-html content must be sanitized before rendering
  const sanitize = import.meta.client
    ? (dirty: string): string => DOMPurify.sanitize(dirty, {
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
    : (dirty: string): string => dirty

  return {
    provide: {
      sanitize,
    },
  }
})
