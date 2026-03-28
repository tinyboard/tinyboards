import { defineEventHandler, getRequestURL, setResponseHeader, sendStream, createError } from 'h3'
import { Readable } from 'node:stream'

/**
 * Proxy for /media/* requests to the backend.
 *
 * In production, nginx serves media files directly. In local bare-metal
 * development (no nginx), this route forwards media requests to the Rust
 * backend so that URLs like /media/avatars/foo.jpg resolve correctly.
 */
export default defineEventHandler(async (event) => {
  const config = useRuntimeConfig()
  const backendUrl = config.backendUrl

  // Reconstruct the path from the original request URL
  const requestUrl = getRequestURL(event)
  const mediaPath = requestUrl.pathname // e.g. /media/avatars/foo.jpg

  const targetUrl = `${backendUrl}${mediaPath}`

  try {
    const response = await fetch(targetUrl, {
      headers: {
        // Forward range requests for video seeking
        ...(event.node.req.headers.range
          ? { Range: event.node.req.headers.range }
          : {}),
      },
    })

    if (!response.ok && response.status !== 206) {
      throw createError({
        statusCode: response.status,
        statusMessage: response.statusText,
      })
    }

    // Forward relevant headers
    const headersToForward = [
      'content-type',
      'content-length',
      'content-range',
      'accept-ranges',
      'cache-control',
      'etag',
      'last-modified',
    ]

    for (const header of headersToForward) {
      const value = response.headers.get(header)
      if (value) {
        setResponseHeader(event, header, value)
      }
    }

    // Set status for partial content (range requests)
    if (response.status === 206) {
      event.node.res.statusCode = 206
    }

    if (response.body) {
      // Convert web ReadableStream to Node.js Readable
      const reader = response.body.getReader()
      const nodeStream = new Readable({
        async read() {
          const { done, value } = await reader.read()
          if (done) {
            this.push(null)
          } else {
            this.push(Buffer.from(value))
          }
        },
      })
      return sendStream(event, nodeStream)
    }

    return ''
  } catch (err: unknown) {
    if ((err as { statusCode?: number }).statusCode) {
      throw err
    }
    throw createError({
      statusCode: 502,
      statusMessage: 'Failed to fetch media from backend',
    })
  }
})
