# TinyBoards GraphQL API - Authentication & Security Guide

## Overview

TinyBoards uses a robust JWT-based authentication system with role-based access control, comprehensive rate limiting, and multiple security layers. This guide covers all authentication and security features for developers integrating with the TinyBoards GraphQL API.

## Table of Contents

1. [Authentication System](#authentication-system)
2. [Authorization & Permission Levels](#authorization--permission-levels)
3. [Security Features](#security-features)
4. [Error Handling](#error-handling)
5. [Best Practices](#best-practices)

---

## Authentication System

### JWT Token-Based Authentication

TinyBoards uses JSON Web Tokens (JWT) for stateless authentication with the following characteristics:

**Token Structure:**
```json
{
  "sub": 123,                    // User ID
  "uname": "username",           // Username
  "iss": "tinyboards",          // Issuer
  "iat": 1634567890,            // Issued at (UNIX timestamp)
  "exp": 1634654290             // Expires at (UNIX timestamp, +24 hours)
}
```

**Token Properties:**
- **Algorithm**: HS256 (HMAC with SHA-256)
- **Expiration**: 24 hours from issuance
- **Secret**: Stored securely in database (`secret` table)
- **Format**: Standard JWT format with header, payload, and signature

### Login Process

**Login Mutation:**
```graphql
mutation Login($username_or_email: String!, $password: String!) {
  login(usernameOrEmail: $username_or_email, password: $password) {
    token
  }
}
```

**Login Flow:**
1. User submits username/email and password
2. System validates credentials against database
3. Password verified using bcrypt with salt
4. Additional checks performed:
   - Account not deleted (`is_deleted = false`)
   - If application mode enabled, account must be approved
5. JWT token generated and returned on success

**Login Response:**
```json
{
  "data": {
    "login": {
      "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
    }
  }
}
```

### Registration Process

**Registration Mutation:**
```graphql
mutation Register(
  $username: String!,
  $display_name: String,
  $email: String,
  $password: String!,
  $invite_code: String,
  $application_answer: String
) {
  register(
    username: $username,
    displayName: $display_name,
    email: $email,
    password: $password,
    inviteCode: $invite_code,
    applicationAnswer: $application_answer
  ) {
    token
    account_created
    application_submitted
  }
}
```

**Registration Modes:**
- **Open**: Anyone can register immediately
- **OpenWithEmailVerification**: Email verification required
- **InviteOnlyAdmin**: Admin-generated invite codes required
- **InviteOnlyUser**: User-generated invite codes required
- **RequireApplication**: Application must be approved by admin
- **Closed**: Registration disabled

**Registration Validation:**
- Username: Must match pattern `^[A-Za-z][A-Za-z0-9_]{0,29}$`
- Password: Must be 10-60 characters long
- Content filtering applied to usernames
- Duplicate email/username checking

### Using Authentication Tokens

**Include Token in Requests:**
```http
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

**JavaScript Example:**
```javascript
const client = new GraphQLClient('http://localhost:8536/api/v2/graphql', {
  headers: {
    'Authorization': `Bearer ${token}`,
    'Content-Type': 'application/json'
  }
});
```

**Token Validation Process:**
1. Extract token from `Authorization: Bearer <token>` header
2. Verify JWT signature using stored secret
3. Check token expiration
4. Load user from database using token's `sub` (user ID)
5. Verify user account is not deleted

---

## Authorization & Permission Levels

### User States

**Authentication States:**
- **Anonymous**: No token provided
- **Authenticated**: Valid token provided
- **Authenticated (Not Banned)**: Valid token + user not banned

**Account States:**
- **Active**: Normal user account
- **Banned**: Account suspended (cannot perform actions)
- **Deleted**: Account marked as deleted (cannot login)
- **Pending Application**: Awaiting admin approval

### Permission Levels

TinyBoards implements a hierarchical admin permission system:

**Regular Users (admin_level = 0):**
- Create posts and comments
- Vote on content
- Subscribe to boards
- Send messages
- Manage own profile

**Admin Permission Levels:**
```rust
pub enum AdminPerms {
    Null,          // Level 0 - No admin permissions
    Appearance,    // Level 1 - Modify site appearance
    Config,        // Level 2 - Basic site configuration
    Content,       // Level 3 - Content moderation
    Users,         // Level 4 - User management
    Boards,        // Level 5 - Board management
    Emoji,         // Level 2 - Emoji management
    Full,          // Level 6 - Full administration
    Owner,         // Level 7 - Owner permissions
    System,        // Level 8 - System-level access
}
```

**Board Moderators:**
- Special role assigned per board
- Can moderate content within their boards
- Separate from site-wide admin permissions

### Permission Checking Patterns

**Require Login:**
```rust
let user = ctx.data::<LoggedInUser>()?.require_user()?;
```

**Require Non-Banned User:**
```rust
let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;
```

**Check Admin Permission:**
```rust
if !user.has_permission(AdminPerms::Users) {
    return Err("Insufficient permissions".into());
}
```

### GraphQL Field-Level Authorization

**Protected Queries/Mutations:**
- Most mutations require authentication
- Admin operations require specific permission levels
- User data access restricted based on privacy settings
- Board management requires moderator or admin status

**Permission Examples:**
```graphql
# Requires login
mutation SubmitPost($title: String!, $boardId: Int!) {
  submitPost(title: $title, boardId: $boardId) {
    id
  }
}

# Requires admin permissions
mutation BanUser($userId: Int!, $reason: String) {
  banUser(userId: $userId, reason: $reason)
}

# Board moderator or admin only
mutation RemovePost($postId: Int!) {
  removePost(postId: $postId)
}
```

---

## Security Features

### Password Security

**Hashing:**
- **Algorithm**: bcrypt
- **Salt**: UUID + configurable suffix
- **Work Factor**: Configurable cost parameter
- **Validation**: 10-60 character length requirement

**Password Storage:**
```rust
// Hash password with salt
let passhash = hash_password(password);

// Verify password
if !verify_password(&user.passhash, &provided_password) {
    return Err("Invalid credentials".into());
}
```

### Rate Limiting

**IP-Based Rate Limiting:**
TinyBoards implements token bucket rate limiting per IP address for different operation types.

**Default Rate Limits:**
```hjson
rate_limit: {
  message: 180,           // 180 messages per minute
  post: 6,                // 6 posts per 10 minutes
  register: 3,            // 3 registrations per hour
  image: 6,               // 6 image uploads per hour
  comment: 6,             // 6 comments per 10 minutes
  search: 60,             // 60 searches per 10 minutes
}
```

**Rate Limit Types:**
- **Message**: Private messaging
- **Post**: Creating new posts
- **Register**: Account registration
- **Image**: File uploads
- **Comment**: Creating comments
- **Search**: Search operations

**Algorithm**: Token bucket with configurable rates and time windows

### CORS Configuration

**Default CORS Settings:**
```hjson
cors: {
  allowed_origins: [
    "http://localhost:3000",
    "http://127.0.0.1:3000",
    "http://localhost:3001",
    "http://127.0.0.1:3001"
  ],
  allow_credentials: true,
  max_age: 3600,
  allowed_methods: ["GET", "POST", "PUT", "DELETE", "OPTIONS", "PATCH"],
  allowed_headers: [
    "Content-Type",
    "Authorization",
    "Accept",
    "Origin",
    "X-Requested-With"
  ]
}
```

### Input Validation & Sanitization

**Username Validation:**
- Regex pattern: `^[A-Za-z][A-Za-z0-9_]{0,29}$`
- Content filtering against banned words
- Maximum length: 30 characters
- Must start with letter

**Content Filtering:**
- Word filter system for usernames and content
- Configurable filter lists
- Optional enforcement on usernames

**SQL Injection Prevention:**
- Diesel ORM with parameterized queries
- No raw SQL in user input handling
- Type-safe database operations

**GraphQL Schema Validation:**
- Input type validation at schema level
- Required field enforcement
- Type coercion with validation

### Session Management

**Token Lifecycle:**
- **Generation**: On successful login/registration
- **Storage**: Client-side (localStorage/sessionStorage recommended)
- **Transmission**: Authorization header only
- **Validation**: On every protected request
- **Expiration**: Automatic after 24 hours
- **Revocation**: No explicit revocation (stateless tokens)

**Security Considerations:**
- No token refresh mechanism (must re-login after expiration)
- Tokens are stateless (cannot be revoked server-side)
- Secret rotation requires all users to re-authenticate

---

## Error Handling

### Authentication Error Responses

**Common Authentication Errors:**

**401 Unauthorized - Login Required:**
```json
{
  "errors": [
    {
      "message": "Login required",
      "extensions": {
        "code": 401
      }
    }
  ]
}
```

**401 Unauthorized - Invalid Credentials:**
```json
{
  "errors": [
    {
      "message": "Username, email address or password invalid.",
      "extensions": {
        "code": 401
      }
    }
  ]
}
```

**401 Unauthorized - Invalid Token:**
```json
{
  "errors": [
    {
      "message": "Invalid or expired token",
      "extensions": {
        "code": 401
      }
    }
  ]
}
```

**403 Forbidden - Account Banned:**
```json
{
  "errors": [
    {
      "message": "Your account is banned",
      "extensions": {
        "code": 403
      }
    }
  ]
}
```

**403 Forbidden - Insufficient Permissions:**
```json
{
  "errors": [
    {
      "message": "Not an admin",
      "extensions": {
        "code": 403
      }
    }
  ]
}
```

**403 Forbidden - Application Pending:**
```json
{
  "errors": [
    {
      "message": "You cannot use your account yet because your application hasn't been accepted.",
      "extensions": {
        "code": 403
      }
    }
  ]
}
```

### Rate Limiting Errors

**429 Too Many Requests:**
```json
{
  "errors": [
    {
      "message": "Too many requests, try again later.",
      "extensions": {
        "code": 429,
        "rate_limit_type": "post"
      }
    }
  ]
}
```

### Registration Errors

**400 Bad Request - Invalid Username:**
```json
{
  "errors": [
    {
      "message": "Invalid username.",
      "extensions": {
        "code": 400
      }
    }
  ]
}
```

**400 Bad Request - Password Too Short:**
```json
{
  "errors": [
    {
      "message": "password length must be between 10-60 characters",
      "extensions": {
        "code": 400
      }
    }
  ]
}
```

**500 Internal Server Error - Duplicate User:**
```json
{
  "errors": [
    {
      "message": "A user with that username already exists.",
      "extensions": {
        "code": 500
      }
    }
  ]
}
```

---

## Best Practices

### Client-Side Implementation

**Token Storage:**
```javascript
// Store token securely
localStorage.setItem('auth_token', token);

// Retrieve token for requests
const token = localStorage.getItem('auth_token');

// Clear token on logout
localStorage.removeItem('auth_token');
```

**Automatic Token Handling:**
```javascript
class TinyBoardsClient {
  constructor(baseURL) {
    this.baseURL = baseURL;
    this.token = localStorage.getItem('auth_token');
  }

  async request(query, variables = {}) {
    const headers = {
      'Content-Type': 'application/json',
    };

    if (this.token) {
      headers['Authorization'] = `Bearer ${this.token}`;
    }

    const response = await fetch(`${this.baseURL}/api/v2/graphql`, {
      method: 'POST',
      headers,
      body: JSON.stringify({ query, variables })
    });

    const result = await response.json();

    // Handle authentication errors
    if (result.errors) {
      for (const error of result.errors) {
        if (error.extensions?.code === 401) {
          this.clearAuth();
          // Redirect to login or show auth modal
        }
      }
    }

    return result;
  }

  async login(usernameOrEmail, password) {
    const query = `
      mutation Login($usernameOrEmail: String!, $password: String!) {
        login(usernameOrEmail: $usernameOrEmail, password: $password) {
          token
        }
      }
    `;

    const result = await this.request(query, { usernameOrEmail, password });

    if (result.data?.login?.token) {
      this.token = result.data.login.token;
      localStorage.setItem('auth_token', this.token);
    }

    return result;
  }

  clearAuth() {
    this.token = null;
    localStorage.removeItem('auth_token');
  }
}
```

### Error Handling Strategy

**Centralized Error Handling:**
```javascript
function handleGraphQLErrors(errors) {
  for (const error of errors) {
    const code = error.extensions?.code;

    switch (code) {
      case 401:
        // Clear auth and redirect to login
        clearAuthToken();
        window.location.href = '/login';
        break;

      case 403:
        // Show permission denied message
        showNotification('Permission denied', 'error');
        break;

      case 429:
        // Show rate limit message
        showNotification('Too many requests, please try again later', 'warning');
        break;

      default:
        // Show generic error
        showNotification(error.message, 'error');
    }
  }
}
```

### Security Considerations

**Token Handling:**
- Store tokens in localStorage or sessionStorage (not cookies for CSRF protection)
- Clear tokens on logout
- Don't expose tokens in logs or console
- Implement automatic token refresh UX when tokens expire

**Request Security:**
- Always use HTTPS in production
- Validate server certificates
- Implement request timeouts
- Use Content Security Policy (CSP) headers

**Rate Limiting Compliance:**
- Implement client-side rate limiting hints
- Show appropriate loading states
- Queue requests when rate limited
- Provide user feedback on rate limit status

**Permission Management:**
- Check user permissions before showing UI elements
- Gracefully handle permission changes
- Implement role-based UI rendering
- Cache permission states appropriately

### Development & Testing

**Authentication Testing:**
```javascript
// Test login flow
const testAuth = async () => {
  const client = new TinyBoardsClient('http://localhost:8536');

  // Test login
  const loginResult = await client.login('testuser', 'testpassword');
  console.log('Login result:', loginResult);

  // Test authenticated request
  const meQuery = `
    query Me {
      me {
        id
        username
        isAdmin
      }
    }
  `;

  const meResult = await client.request(meQuery);
  console.log('Me result:', meResult);
};
```

**Permission Testing:**
```javascript
// Test admin functionality
const testAdminPermissions = async () => {
  const adminQuery = `
    query BannedUsers {
      bannedUsers {
        id
        username
      }
    }
  `;

  const result = await client.request(adminQuery);

  if (result.errors) {
    console.log('Expected error for non-admin user:', result.errors);
  } else {
    console.log('Admin data:', result.data);
  }
};
```

This comprehensive guide covers all aspects of authentication and security in the TinyBoards GraphQL API. For specific implementation details, refer to the source code in the respective crate directories.