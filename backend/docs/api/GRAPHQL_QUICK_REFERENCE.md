# TinyBoards GraphQL API - Quick Reference

## Endpoint
```
POST http://localhost:8536/api/v2/graphql
```

## Authentication
```bash
# Login and get token
curl -X POST http://localhost:8536/api/v2/graphql \
  -H "Content-Type: application/json" \
  -d '{
    "query": "mutation Login($usernameOrEmail: String!, $password: String!) { login(usernameOrEmail: $usernameOrEmail, password: $password) { token } }",
    "variables": { "usernameOrEmail": "username", "password": "password" }
  }'

# Use token in subsequent requests
curl -X POST http://localhost:8536/api/v2/graphql \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_TOKEN_HERE" \
  -d '{ "query": "query { me { name email } }" }'
```

## Quick Examples

### Register User
```graphql
mutation {
  register(
    username: "newuser"
    email: "user@example.com"
    password: "SecurePass123!"
  ) {
    token
    accountCreated
  }
}
```

### Get Current User
```graphql
query {
  me {
    id
    name
    email
    isAdmin
    postCount
    reputation
  }
}
```

### List Posts
```graphql
query {
  listPosts(listingType: local, sort: hot, limit: 25) {
    id
    title
    score
    commentCount
    creator { name }
    board { name title }
  }
}
```

### Get Single Post with Comments
```graphql
query {
  post(id: 123) {
    id
    title
    bodyHTML
    score
    commentCount
    comments(sort: hot, limit: 50) {
      id
      bodyHTML
      score
      level
      creator { name }
    }
  }
}
```

### Create Text Post
```graphql
mutation {
  createPost(
    title: "My Post Title"
    board: "technology"
    body: "Post content here..."
  ) {
    id
    title
    score
  }
}
```

### Create Link Post
```graphql
mutation {
  createPost(
    title: "Interesting Link"
    board: "technology"
    link: "https://example.com"
  ) {
    id
    title
    url
  }
}
```

### Create Comment
```graphql
# Reply to post
mutation {
  createComment(
    replyToPostId: 123
    body: "Great post!"
  ) {
    id
    bodyHTML
    score
  }
}

# Reply to comment
mutation {
  createComment(
    replyToCommentId: 456
    body: "I agree!"
  ) {
    id
    bodyHTML
    level
  }
}
```

### Vote on Post/Comment
```graphql
# Upvote post
mutation {
  votePost(postId: 123, vote: 1) {
    id
    score
    myVote
  }
}

# Downvote comment
mutation {
  voteComment(commentId: 456, vote: -1) {
    id
    score
    myVote
  }
}
```

### List Boards
```graphql
query {
  listBoards(listingType: local, sort: hot, limit: 25) {
    id
    name
    title
    description
    subscriberCount
    postCount
  }
}
```

### Get Board Details
```graphql
query {
  board(name: "technology") {
    id
    name
    title
    description
    subscriberCount
    postCount
    isSubscribed
  }
}
```

### Create Board
```graphql
mutation {
  createBoard(
    name: "programming"
    title: "Programming Discussion"
    description: "A place for programming topics"
  ) {
    id
    name
    title
  }
}
```

### Get User Profile
```graphql
query {
  user(username: "someuser") {
    id
    name
    displayName
    bio
    avatar
    postCount
    commentCount
    reputation
    posts(limit: 5) {
      id
      title
      score
    }
  }
}
```

## JavaScript Client Template
```javascript
class TinyBoardsClient {
  constructor(baseUrl = 'http://localhost:8536') {
    this.url = `${baseUrl}/api/v2/graphql`;
    this.token = null;
  }

  setToken(token) {
    this.token = token;
  }

  async request(query, variables = {}) {
    const headers = { 'Content-Type': 'application/json' };
    if (this.token) headers['Authorization'] = `Bearer ${this.token}`;

    const response = await fetch(this.url, {
      method: 'POST',
      headers,
      body: JSON.stringify({ query, variables })
    });

    const result = await response.json();
    if (result.errors) throw new Error(result.errors[0].message);
    return result.data;
  }

  async login(usernameOrEmail, password) {
    const data = await this.request(`
      mutation Login($usernameOrEmail: String!, $password: String!) {
        login(usernameOrEmail: $usernameOrEmail, password: $password) {
          token
        }
      }
    `, { usernameOrEmail, password });

    this.setToken(data.login.token);
    return data.login.token;
  }

  async getPosts(options = {}) {
    return this.request(`
      query ListPosts($listingType: ListingType!, $sort: SortType, $limit: Int) {
        listPosts(listingType: $listingType, sort: $sort, limit: $limit) {
          id
          title
          score
          commentCount
          creator { name }
          board { name title }
        }
      }
    `, {
      listingType: options.listingType || 'local',
      sort: options.sort || 'hot',
      limit: options.limit || 25
    });
  }

  async createPost(title, board, body = null, link = null) {
    return this.request(`
      mutation CreatePost($title: String!, $board: String, $body: String, $link: String) {
        createPost(title: $title, board: $board, body: $body, link: $link) {
          id
          title
          score
        }
      }
    `, { title, board, body, link });
  }
}

// Usage
const client = new TinyBoardsClient();
await client.login('username', 'password');
const posts = await client.getPosts({ sort: 'new' });
```

## Error Codes
- `400` - Bad Request (validation error)
- `401` - Unauthorized (login required)
- `403` - Forbidden (banned or insufficient permissions)
- `404` - Not Found
- `410` - Gone (content deleted/banned)
- `500` - Internal Server Error

## Rate Limits
- Posts: 6 per 10 minutes
- Comments: 6 per 10 minutes
- Images: 6 per hour
- Messages: 180 per minute
- Search: 60 per 10 minutes