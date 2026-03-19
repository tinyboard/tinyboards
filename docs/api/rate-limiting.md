# Rate Limiting

TinyBoards applies per-IP rate limits to prevent abuse. Limits are configurable via environment variables.

## Table of Contents

- [Rate Limit Tiers](#rate-limit-tiers)
- [Response Format](#response-format)
- [Configuration](#configuration)
- [Handling Rate Limits](#handling-rate-limits)

## Rate Limit Tiers

Different actions have different rate limits, measured in requests per minute:

| Action | Default (req/min) | Env Variable | Description |
|--------|--------------------|--------------|-------------|
| Post submission | 60 | `RATE_LIMIT_POST` | Creating new posts |
| Comment submission | 60 | `RATE_LIMIT_COMMENT` | Creating new comments |
| Registration | 30 | `RATE_LIMIT_REGISTER` | Account registration attempts |
| Image upload | 60 | `RATE_LIMIT_IMAGE` | File/image uploads |
| Message send | 1800 | `RATE_LIMIT_MESSAGE` | Private message sends |
| Search | 600 | `RATE_LIMIT_SEARCH` | Search queries |

General GraphQL queries (reading data) are not individually rate-limited beyond standard connection limits.

## Response Format

When a rate limit is exceeded, the API returns a GraphQL error:

```json
{
  "data": null,
  "errors": [
    {
      "message": "Rate limit exceeded. Try again in 45 seconds.",
      "extensions": {
        "code": "RATE_LIMITED"
      }
    }
  ]
}
```

The HTTP response may also return status `429 Too Many Requests` with headers indicating the limit state:

| Header | Description |
|--------|-------------|
| `X-RateLimit-Limit` | Maximum requests allowed in the window |
| `X-RateLimit-Remaining` | Requests remaining in the current window |
| `X-RateLimit-Reset` | Unix timestamp when the window resets |
| `Retry-After` | Seconds until the rate limit resets |

## Configuration

Rate limits are configured in the `rate_limit` section of `tinyboards.hjson`:

```hjson
rate_limit: {
  message: 180
  message_per_second: 60
  post: 6
  post_per_second: 600
  register: 3
  register_per_second: 3600
  image: 6
  image_per_second: 3600
  comment: 6
  comment_per_second: 600
  search: 60
  search_per_second: 600
}
```

These can also be adjusted through the admin panel after initial setup.

## Handling Rate Limits

### Retry Strategy

When you receive a `RATE_LIMITED` error:

1. Read the `Retry-After` header (or parse the error message for the wait time).
2. Wait the specified duration.
3. Retry the request.

### Example (JavaScript)

```javascript
async function graphqlRequest(query, variables) {
  const response = await fetch('/api/v2/graphql', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ query, variables }),
    credentials: 'include',
  });

  if (response.status === 429) {
    const retryAfter = parseInt(response.headers.get('Retry-After') || '60', 10);
    await new Promise(resolve => setTimeout(resolve, retryAfter * 1000));
    return graphqlRequest(query, variables);
  }

  return response.json();
}
```

### Best Practices

- **Batch reads** — Use a single GraphQL query with multiple fields instead of multiple separate requests.
- **Debounce search** — Add a delay (300–500ms) between keystrokes in search fields before sending queries.
- **Cache responses** — Cache read-only data client-side to reduce request volume.
- **Handle errors gracefully** — Show a user-friendly message when rate-limited, not a raw error.
