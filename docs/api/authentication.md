# Authentication System

TinyBoards uses a dual-token authentication system with `httpOnly` cookies for security.

## Table of Contents

- [Overview](#overview)
- [Token Types](#token-types)
- [Authentication Flow](#authentication-flow)
- [Session Management](#session-management)
- [Security Properties](#security-properties)

## Overview

```
┌────────┐     login      ┌──────────┐      verify       ┌────────────┐
│ Client │───────────────►│  Backend │─────────────────►│ PostgreSQL │
│        │                │          │                   │            │
│        │◄───────────────│          │◄─────────────────│            │
│        │  access_token  │          │  user + session   │            │
│        │  refresh_token │          │                   │            │
│        │  (httpOnly)    │          │                   │            │
└────────┘                └──────────┘                   └────────────┘
```

On login, the backend:
1. Validates credentials (Argon2 password hash comparison)
2. Creates a session row in `auth_sessions`
3. Issues a short-lived JWT (access token) and a long-lived refresh token
4. Returns both as `httpOnly` cookies

## Token Types

### Access Token (JWT)

| Property | Value |
|----------|-------|
| Format | Signed JWT (HS256) |
| Lifetime | 15 minutes |
| Storage | `httpOnly` cookie named `access_token` |
| Contains | `user_id`, `is_admin`, `issued_at`, `expires_at` |
| Signed with | `JWT_SECRET` environment variable |

The access token is verified on every authenticated request. It is not stored in the database — validation is purely cryptographic.

### Refresh Token

| Property | Value |
|----------|-------|
| Format | Opaque random string |
| Lifetime | 30 days |
| Storage | `httpOnly` cookie named `refresh_token` |
| Server-side | SHA-256 hash stored in `auth_sessions.refresh_token_hash` |

The refresh token is used to obtain new access tokens without re-entering credentials.

## Authentication Flow

### Login

```
Client                    Backend                   Database
  │                         │                          │
  │  POST /api/v2/graphql   │                          │
  │  login(user, pass)      │                          │
  │────────────────────────►│                          │
  │                         │  SELECT passhash          │
  │                         │─────────────────────────►│
  │                         │◄─────────────────────────│
  │                         │                          │
  │                         │  Verify Argon2 hash      │
  │                         │                          │
  │                         │  INSERT auth_sessions     │
  │                         │─────────────────────────►│
  │                         │◄─────────────────────────│
  │                         │                          │
  │  Set-Cookie:            │                          │
  │    access_token=JWT     │                          │
  │    refresh_token=opaque │                          │
  │◄────────────────────────│                          │
```

### Authenticated Request

```
Client                    Backend
  │                         │
  │  POST /api/v2/graphql   │
  │  Cookie: access_token   │
  │────────────────────────►│
  │                         │
  │                         │  Verify JWT signature
  │                         │  Check expiry
  │                         │  Extract user_id
  │                         │
  │  200 OK + data          │
  │◄────────────────────────│
```

### Token Refresh

When the access token expires, the client sends a refresh request:

```
Client                    Backend                   Database
  │                         │                          │
  │  POST /api/v2/graphql   │                          │
  │  refreshToken mutation  │                          │
  │  Cookie: refresh_token  │                          │
  │────────────────────────►│                          │
  │                         │  Hash(refresh_token)     │
  │                         │  SELECT auth_sessions    │
  │                         │  WHERE hash matches      │
  │                         │─────────────────────────►│
  │                         │◄─────────────────────────│
  │                         │                          │
  │                         │  Generate new tokens     │
  │                         │  UPDATE auth_sessions    │
  │                         │─────────────────────────►│
  │                         │                          │
  │  Set-Cookie:            │                          │
  │    access_token=newJWT  │                          │
  │    refresh_token=new    │                          │
  │◄────────────────────────│                          │
```

The old refresh token is invalidated (replaced by the new hash). This is **refresh token rotation** — each refresh token can only be used once.

### Logout

```
Client                    Backend                   Database
  │                         │                          │
  │  POST /api/v2/graphql   │                          │
  │  logout mutation        │                          │
  │────────────────────────►│                          │
  │                         │  DELETE auth_sessions    │
  │                         │  WHERE id = session_id   │
  │                         │─────────────────────────►│
  │                         │                          │
  │  Set-Cookie:            │                          │
  │    access_token=""      │                          │
  │    refresh_token=""     │                          │
  │    (Max-Age=0)          │                          │
  │◄────────────────────────│                          │
```

## Session Management

### auth_sessions Table

```sql
CREATE TABLE auth_sessions (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    refresh_token_hash TEXT NOT NULL,
    user_agent      TEXT,
    ip_address      TEXT,
    last_used_at    TIMESTAMPTZ,
    expires_at      TIMESTAMPTZ NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

Each session tracks:
- **user_id** — Which user owns this session
- **refresh_token_hash** — SHA-256 hash of the refresh token (never store raw tokens)
- **user_agent** / **ip_address** — For session management UI ("where am I logged in?")
- **last_used_at** — Updated on each token refresh
- **expires_at** — Hard expiry (30 days from creation)

### Session Cleanup

Expired sessions are cleaned up by a background task. Sessions also cascade-delete when the associated user is deleted.

### Revoking Sessions

Admins can revoke sessions (force logout) by deleting the corresponding `auth_sessions` row. The user's access token remains valid for up to 15 minutes, but no new access tokens can be obtained.

## Security Properties

| Property | Implementation |
|----------|---------------|
| **No XSS token theft** | Tokens stored in `httpOnly` cookies — inaccessible to JavaScript |
| **CSRF protection** | `SameSite=Lax` cookie attribute; GraphQL mutations require POST |
| **Short exposure window** | Access tokens expire in 15 minutes |
| **Refresh token rotation** | Each refresh invalidates the previous token |
| **Server-side revocation** | Deleting a session row immediately blocks refresh |
| **Password hashing** | Argon2 with per-user salt suffix |
| **Secure token storage** | Only SHA-256 hashes stored in the database |
