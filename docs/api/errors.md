# Error Handling

TinyBoards returns structured errors through the standard GraphQL error format, extended with application-specific error codes.

## Table of Contents

- [Error Format](#error-format)
- [Error Codes by Category](#error-codes-by-category)
- [HTTP Status Codes](#http-status-codes)

## Error Format

All errors follow the GraphQL spec with an `extensions` object containing the error code:

```json
{
  "data": null,
  "errors": [
    {
      "message": "Invalid credentials",
      "locations": [{ "line": 1, "column": 1 }],
      "path": ["login"],
      "extensions": {
        "code": "INVALID_CREDENTIALS"
      }
    }
  ]
}
```

| Field | Type | Description |
|-------|------|-------------|
| `message` | `String` | Human-readable error description |
| `locations` | `[Location]` | Position in the GraphQL query where the error occurred |
| `path` | `[String]` | Path to the field that caused the error |
| `extensions.code` | `String` | Machine-readable error code (use this for programmatic handling) |

Multiple errors can be returned in a single response (e.g., validation errors for multiple fields).

## Error Codes by Category

### Authentication

| Code | Message | Description |
|------|---------|-------------|
| `NOT_AUTHENTICATED` | Authentication required | No valid access token provided |
| `INVALID_CREDENTIALS` | Invalid credentials | Wrong username/email or password |
| `INVALID_REFRESH_TOKEN` | Invalid refresh token | Refresh token is missing, expired, or revoked |
| `SESSION_EXPIRED` | Session has expired | The session has been terminated |
| `USER_BANNED` | User is banned | Account has been banned from the site |
| `EMAIL_NOT_VERIFIED` | Email verification required | Email must be verified before login |

### Authorization

| Code | Message | Description |
|------|---------|-------------|
| `NOT_AUTHORIZED` | Not authorized | User lacks permission for the requested action |
| `NOT_ADMIN` | Admin access required | Action requires site administrator privileges |
| `NOT_MODERATOR` | Moderator access required | Action requires board moderator privileges |

### Registration

| Code | Message | Description |
|------|---------|-------------|
| `USERNAME_TAKEN` | Username is already taken | The requested username is in use |
| `EMAIL_TAKEN` | Email is already registered | The email address is already associated with an account |
| `PASSWORDS_DONT_MATCH` | Passwords do not match | `password` and `passwordVerify` fields differ |
| `REGISTRATION_CLOSED` | Registration is closed | The site is not accepting new registrations |
| `INVALID_INVITE_CODE` | Invalid invite code | The invite code is invalid or expired |
| `APPLICATION_PENDING` | Application is pending review | Registration application submitted but not yet approved |
| `CAPTCHA_FAILED` | CAPTCHA verification failed | CAPTCHA answer was incorrect |

### Content

| Code | Message | Description |
|------|---------|-------------|
| `NOT_FOUND` | Resource not found | The requested post, comment, board, user, etc. does not exist |
| `CONTENT_REMOVED` | Content has been removed | The post or comment was removed by a moderator |
| `POST_LOCKED` | Post is locked | Cannot comment on or edit a locked post |
| `BOARD_BANNED` | Board is banned | The board has been banned by a site administrator |
| `DUPLICATE_VOTE` | Already voted | User has already voted on this content |
| `SELF_VOTE` | Cannot vote on own content | Users cannot upvote or downvote their own posts/comments |

### Moderation

| Code | Message | Description |
|------|---------|-------------|
| `CANNOT_BAN_ADMIN` | Cannot ban an admin | Moderators cannot ban site administrators |
| `CANNOT_BAN_SELF` | Cannot ban yourself | Users cannot ban themselves |
| `ALREADY_BANNED` | User is already banned | The target user is already banned |
| `NOT_BANNED` | User is not banned | Cannot unban a user who isn't banned |

### Validation

| Code | Message | Description |
|------|---------|-------------|
| `VALIDATION_ERROR` | Validation failed | Input failed validation (check `message` for details) |
| `INVALID_INPUT` | Invalid input | General input validation failure |
| `TITLE_TOO_LONG` | Title exceeds maximum length | Post title exceeds 200 characters |
| `BODY_TOO_LONG` | Body exceeds maximum length | Post or comment body exceeds the maximum |
| `INVALID_URL` | Invalid URL format | A URL field contains an invalid URL |

### Upload

| Code | Message | Description |
|------|---------|-------------|
| `FILE_TOO_LARGE` | File exceeds maximum size | Upload exceeds `MAX_FILE_SIZE_MB` |
| `INVALID_FILE_TYPE` | File type not allowed | The uploaded file type is not in the allowed list |
| `UPLOAD_FAILED` | Upload failed | File storage backend returned an error |

### Rate Limiting

| Code | Message | Description |
|------|---------|-------------|
| `RATE_LIMITED` | Rate limit exceeded | Too many requests; see [rate-limiting.md](rate-limiting.md) |

### Server

| Code | Message | Description |
|------|---------|-------------|
| `INTERNAL_ERROR` | Internal server error | Unexpected server-side failure |
| `DATABASE_ERROR` | Database error | Database operation failed |

## HTTP Status Codes

The GraphQL endpoint always returns HTTP `200 OK` for both successful responses and application errors (errors are in the response body). The only exceptions:

| Status | Cause |
|--------|-------|
| `200` | All GraphQL responses (check `errors` array in body) |
| `400` | Malformed GraphQL query (syntax error) |
| `429` | Rate limit exceeded |
| `500` | Server crash or unhandled error |
