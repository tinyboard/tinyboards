# Auth Endpoints

All authentication operations go through the GraphQL API at `POST /api/v2/graphql`.

## Table of Contents

- [Register](#register)
- [Login](#login)
- [Get Current User](#get-current-user)
- [Refresh Token](#refresh-token)
- [Logout](#logout)
- [Change Password](#change-password)
- [Delete Account](#delete-account)

## Register

Create a new user account.

### Request

```bash
curl -X POST https://yourdomain.com/api/v2/graphql \
  -H "Content-Type: application/json" \
  -c cookies.txt \
  -d '{
    "query": "mutation Register($input: RegisterInput!) { register(input: $input) { jwt user { id name } } }",
    "variables": {
      "input": {
        "username": "newuser",
        "password": "securepassword123",
        "passwordVerify": "securepassword123",
        "email": "newuser@example.com"
      }
    }
  }'
```

### Response (Success)

```json
{
  "data": {
    "register": {
      "jwt": "eyJhbGciOiJIUzI1NiIs...",
      "user": {
        "id": "550e8400-e29b-41d4-a716-446655440000",
        "name": "newuser"
      }
    }
  }
}
```

The response also sets `httpOnly` cookies for the access token and refresh token.

### Registration Modes

The site administrator configures the registration mode:

| Mode | Behavior |
|------|----------|
| `open` | Anyone can register |
| `invite_only` | Requires an invite code: add `"inviteCode": "abc123"` to the input |
| `application_required` | Requires an application: add `"applicationAnswer": "Your answer"` to the input |
| `closed` | Registration is disabled |

### Possible Errors

| Error | Description |
|-------|-------------|
| `USERNAME_TAKEN` | Username is already in use |
| `EMAIL_TAKEN` | Email address is already registered |
| `PASSWORDS_DONT_MATCH` | `password` and `passwordVerify` differ |
| `INVALID_INVITE_CODE` | The invite code is invalid or expired |
| `REGISTRATION_CLOSED` | Site registration is closed |
| `CAPTCHA_FAILED` | CAPTCHA verification failed (if enabled) |

## Login

Authenticate with username/email and password.

### Request

```bash
curl -X POST https://yourdomain.com/api/v2/graphql \
  -H "Content-Type: application/json" \
  -c cookies.txt \
  -d '{
    "query": "mutation Login($usernameOrEmail: String!, $password: String!) { login(usernameOrEmail: $usernameOrEmail, password: $password) { jwt } }",
    "variables": {
      "usernameOrEmail": "admin",
      "password": "yourpassword"
    }
  }'
```

### Response (Success)

```json
{
  "data": {
    "login": {
      "jwt": "eyJhbGciOiJIUzI1NiIs..."
    }
  }
}
```

The server sets two `httpOnly` cookies:
- `access_token` — JWT valid for 15 minutes
- `refresh_token` — Opaque token valid for 30 days

### Possible Errors

| Error | Description |
|-------|-------------|
| `INVALID_CREDENTIALS` | Wrong username/email or password |
| `USER_BANNED` | Account has been banned |
| `EMAIL_NOT_VERIFIED` | Email verification is required but pending |

## Get Current User

Retrieve the authenticated user's profile and unread counts.

### Request

```bash
curl -X POST https://yourdomain.com/api/v2/graphql \
  -H "Content-Type: application/json" \
  -b cookies.txt \
  -d '{
    "query": "{ me { id name displayName email isAdmin avatar bio } unreadRepliesCount unreadMentionsCount }"
  }'
```

### Response

```json
{
  "data": {
    "me": {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "name": "admin",
      "displayName": "Admin User",
      "email": "admin@example.com",
      "isAdmin": true,
      "avatar": "/media/avatars/admin.webp",
      "bio": "Site administrator"
    },
    "unreadRepliesCount": 3,
    "unreadMentionsCount": 1
  }
}
```

Returns `null` for `me` if not authenticated.

## Refresh Token

When the access token expires (after 15 minutes), the client automatically uses the refresh token cookie to obtain a new access token. This happens transparently via the `httpOnly` cookie — no manual intervention required.

For programmatic clients, if you receive an authentication error, re-send the request; the refresh token cookie will trigger automatic renewal.

### Manual Refresh (if needed)

```bash
curl -X POST https://yourdomain.com/api/v2/graphql \
  -H "Content-Type: application/json" \
  -b cookies.txt \
  -c cookies.txt \
  -d '{
    "query": "mutation { refreshToken { jwt } }"
  }'
```

### Response

```json
{
  "data": {
    "refreshToken": {
      "jwt": "eyJhbGciOiJIUzI1NiIs..."
    }
  }
}
```

New `httpOnly` cookies are set. The old refresh token is invalidated.

### Possible Errors

| Error | Description |
|-------|-------------|
| `INVALID_REFRESH_TOKEN` | Refresh token is missing, expired, or revoked |
| `SESSION_EXPIRED` | The session has been terminated (e.g., by admin) |

## Logout

Invalidate the current session.

### Request

```bash
curl -X POST https://yourdomain.com/api/v2/graphql \
  -H "Content-Type: application/json" \
  -b cookies.txt \
  -d '{
    "query": "mutation { logout }"
  }'
```

### Response

```json
{
  "data": {
    "logout": true
  }
}
```

This clears the `httpOnly` cookies and deletes the session from the `auth_sessions` table.

## Change Password

### Request

```bash
curl -X POST https://yourdomain.com/api/v2/graphql \
  -H "Content-Type: application/json" \
  -b cookies.txt \
  -d '{
    "query": "mutation ChangePassword($old: String!, $new: String!, $newVerify: String!) { changePassword(oldPassword: $old, newPassword: $new, newPasswordVerify: $newVerify) }",
    "variables": {
      "old": "currentpassword",
      "new": "newsecurepassword",
      "newVerify": "newsecurepassword"
    }
  }'
```

### Possible Errors

| Error | Description |
|-------|-------------|
| `INVALID_CREDENTIALS` | Current password is incorrect |
| `PASSWORDS_DONT_MATCH` | New password and verification don't match |

## Delete Account

Soft-deletes the user account. Requires password confirmation.

### Request

```bash
curl -X POST https://yourdomain.com/api/v2/graphql \
  -H "Content-Type: application/json" \
  -b cookies.txt \
  -d '{
    "query": "mutation DeleteAccount($password: String!) { deleteAccount(password: $password) }",
    "variables": {
      "password": "yourpassword"
    }
  }'
```

### Response

```json
{
  "data": {
    "deleteAccount": true
  }
}
```

The account is soft-deleted (sets `deleted_at` timestamp on the `users` row). Content created by the user is preserved but attributed to a deleted account.
