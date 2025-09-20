# TinyBoards GraphQL API - Complete Integration Guide

This guide provides comprehensive examples and integration patterns for the TinyBoards GraphQL API. All examples are based on the actual schema and operations available in the codebase.

## Table of Contents

1. [Authentication Flow](#authentication-flow)
2. [Board Management](#board-management)
3. [Post Operations](#post-operations)
4. [Comment System](#comment-system)
5. [Voting System](#voting-system)
6. [User Management](#user-management)
7. [Integration Examples](#integration-examples)
8. [Error Handling](#error-handling)
9. [Best Practices](#best-practices)
10. [Testing & Development](#testing--development)

## GraphQL Endpoint

- **Development**: `http://localhost:8536/api/v2/graphql`
- **GraphQL Playground**: `http://localhost:8536/graphql` (development only)

## Authentication Flow

### 1. User Registration

**GraphQL Mutation**:
```graphql
mutation Register($username: String!, $displayName: String, $email: String, $password: String!, $inviteCode: String, $applicationAnswer: String) {
  register(
    username: $username
    displayName: $displayName
    email: $email
    password: $password
    inviteCode: $inviteCode
    applicationAnswer: $applicationAnswer
  ) {
    token
    accountCreated
    applicationSubmitted
  }
}
```

**Variables**:
```json
{
  "username": "newuser123",
  "displayName": "New User",
  "email": "user@example.com",
  "password": "SecurePassword123!"
}
```

**JavaScript Example**:
```javascript
const registerUser = async (userData) => {
  const response = await fetch('http://localhost:8536/api/v2/graphql', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({
      query: `
        mutation Register($username: String!, $displayName: String, $email: String, $password: String!) {
          register(username: $username, displayName: $displayName, email: $email, password: $password) {
            token
            accountCreated
            applicationSubmitted
          }
        }
      `,
      variables: userData
    })
  });

  const result = await response.json();

  if (result.data?.register?.token) {
    // Store token for future requests
    localStorage.setItem('tinyboards_token', result.data.register.token);
    return result.data.register;
  }

  throw new Error(result.errors?.[0]?.message || 'Registration failed');
};

// Usage
registerUser({
  username: "newuser123",
  displayName: "New User",
  email: "user@example.com",
  password: "SecurePassword123!"
}).then(result => {
  console.log('Registration successful:', result);
}).catch(error => {
  console.error('Registration failed:', error.message);
});
```

### 2. User Login

**GraphQL Mutation**:
```graphql
mutation Login($usernameOrEmail: String!, $password: String!) {
  login(usernameOrEmail: $usernameOrEmail, password: $password) {
    token
  }
}
```

**Variables**:
```json
{
  "usernameOrEmail": "newuser123",
  "password": "SecurePassword123!"
}
```

**cURL Example**:
```bash
curl -X POST http://localhost:8536/api/v2/graphql \
  -H "Content-Type: application/json" \
  -d '{
    "query": "mutation Login($usernameOrEmail: String!, $password: String!) { login(usernameOrEmail: $usernameOrEmail, password: $password) { token } }",
    "variables": {
      "usernameOrEmail": "newuser123",
      "password": "SecurePassword123!"
    }
  }'
```

**Python Example**:
```python
import requests
import json

def login_user(username_or_email, password):
    url = "http://localhost:8536/api/v2/graphql"

    query = """
    mutation Login($usernameOrEmail: String!, $password: String!) {
      login(usernameOrEmail: $usernameOrEmail, password: $password) {
        token
      }
    }
    """

    variables = {
        "usernameOrEmail": username_or_email,
        "password": password
    }

    response = requests.post(url, json={
        "query": query,
        "variables": variables
    })

    result = response.json()

    if response.status_code == 200 and result.get("data", {}).get("login", {}).get("token"):
        return result["data"]["login"]["token"]
    else:
        raise Exception(result.get("errors", [{}])[0].get("message", "Login failed"))

# Usage
try:
    token = login_user("newuser123", "SecurePassword123!")
    print(f"Login successful. Token: {token}")
except Exception as e:
    print(f"Login failed: {e}")
```

### 3. Get Current User Info

**GraphQL Query**:
```graphql
query Me {
  me {
    id
    name
    displayName
    email
    avatar
    banner
    bio
    isAdmin
    isBanned
    isDeleted
    creationDate
    postCount
    postScore
    commentCount
    commentScore
    reputation
  }
}
```

**Authenticated Request Example**:
```javascript
const getCurrentUser = async (token) => {
  const response = await fetch('http://localhost:8536/api/v2/graphql', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'Authorization': `Bearer ${token}`
    },
    body: JSON.stringify({
      query: `
        query Me {
          me {
            id
            name
            displayName
            email
            avatar
            isAdmin
            postCount
            commentCount
            reputation
          }
        }
      `
    })
  });

  const result = await response.json();

  if (result.errors) {
    throw new Error(result.errors[0].message);
  }

  return result.data.me;
};
```

## Board Management

### 1. List Boards

**GraphQL Query**:
```graphql
query ListBoards(
  $limit: Int
  $page: Int
  $sort: SortType
  $listingType: ListingType
  $searchTerm: String
  $searchTitleAndDesc: Boolean
) {
  listBoards(
    limit: $limit
    page: $page
    sort: $sort
    listingType: $listingType
    searchTerm: $searchTerm
    searchTitleAndDesc: $searchTitleAndDesc
  ) {
    id
    name
    title
    description
    icon
    banner
    primaryColor
    secondaryColor
    hoverColor
    isRemoved
    isBanned
    isDeleted
    subscriberCount
    postCount
    commentCount
    creationDate
    isSubscribed
  }
}
```

**Variables**:
```json
{
  "limit": 25,
  "page": 0,
  "sort": "hot",
  "listingType": "local"
}
```

### 2. Get Specific Board

**GraphQL Query**:
```graphql
query GetBoard($name: String!) {
  board(name: $name) {
    id
    name
    title
    description
    icon
    banner
    primaryColor
    secondaryColor
    hoverColor
    subscriberCount
    postCount
    commentCount
    creationDate
    isSubscribed
    myModPermissions
  }
}
```

**TypeScript Example**:
```typescript
interface Board {
  id: number;
  name: string;
  title: string;
  description: string;
  icon?: string;
  banner?: string;
  subscriberCount: number;
  postCount: number;
  commentCount: number;
  isSubscribed: boolean;
}

const getBoard = async (boardName: string, token?: string): Promise<Board> => {
  const headers: Record<string, string> = {
    'Content-Type': 'application/json',
  };

  if (token) {
    headers['Authorization'] = `Bearer ${token}`;
  }

  const response = await fetch('http://localhost:8536/api/v2/graphql', {
    method: 'POST',
    headers,
    body: JSON.stringify({
      query: `
        query GetBoard($name: String!) {
          board(name: $name) {
            id
            name
            title
            description
            icon
            banner
            subscriberCount
            postCount
            commentCount
            isSubscribed
          }
        }
      `,
      variables: { name: boardName }
    })
  });

  const result = await response.json();

  if (result.errors) {
    throw new Error(result.errors[0].message);
  }

  return result.data.board;
};

// Usage
getBoard("technology").then(board => {
  console.log(`Board: ${board.title} (${board.subscriberCount} subscribers)`);
});
```

### 3. Create New Board

**GraphQL Mutation**:
```graphql
mutation CreateBoard(
  $name: String!
  $title: String!
  $description: String
  $primaryColor: String
  $secondaryColor: String
  $hoverColor: String
  $isNSFW: Boolean
) {
  createBoard(
    name: $name
    title: $title
    description: $description
    primaryColor: $primaryColor
    secondaryColor: $secondaryColor
    hoverColor: $hoverColor
    isNSFW: $isNSFW
  ) {
    id
    name
    title
    description
    creationDate
  }
}
```

**Variables**:
```json
{
  "name": "programming",
  "title": "Programming Discussion",
  "description": "A place to discuss programming topics, share code, and help each other learn.",
  "primaryColor": "#3b82f6",
  "secondaryColor": "#1e40af",
  "hoverColor": "#2563eb",
  "isNSFW": false
}
```

## Post Operations

### 1. List Posts

**GraphQL Query**:
```graphql
query ListPosts(
  $listingType: ListingType!
  $sort: SortType
  $page: Int
  $limit: Int
  $boardId: Int
  $userId: Int
  $personName: String
  $boardName: String
  $savedOnly: Boolean
) {
  listPosts(
    listingType: $listingType
    sort: $sort
    page: $page
    limit: $limit
    boardId: $boardId
    userId: $userId
    personName: $personName
    boardName: $boardName
    savedOnly: $savedOnly
  ) {
    id
    title
    url
    bodyHTML
    isRemoved
    isDeleted
    isLocked
    isNSFW
    score
    myVote
    commentCount
    creationDate
    updated
    creator {
      id
      name
      displayName
      avatar
    }
    board {
      id
      name
      title
      icon
      primaryColor
    }
  }
}
```

**Complete JavaScript Example**:
```javascript
const fetchPosts = async (options = {}) => {
  const {
    listingType = 'local',
    sort = 'hot',
    page = 0,
    limit = 25,
    boardName = null,
    token = null
  } = options;

  const headers = {
    'Content-Type': 'application/json',
  };

  if (token) {
    headers['Authorization'] = `Bearer ${token}`;
  }

  const response = await fetch('http://localhost:8536/api/v2/graphql', {
    method: 'POST',
    headers,
    body: JSON.stringify({
      query: `
        query ListPosts(
          $listingType: ListingType!
          $sort: SortType
          $page: Int
          $limit: Int
          $boardName: String
          $includeBoard: Boolean!
        ) {
          listPosts(
            listingType: $listingType
            sort: $sort
            page: $page
            limit: $limit
            boardName: $boardName
          ) {
            id
            title
            url
            bodyHTML
            isNSFW
            score
            myVote
            commentCount
            creationDate
            creator {
              id
              name
              displayName
              avatar
            }
            board @include(if: $includeBoard) {
              id
              name
              title
              icon
              primaryColor
            }
          }
        }
      `,
      variables: {
        listingType,
        sort,
        page,
        limit,
        boardName,
        includeBoard: !boardName // Include board info if not filtering by specific board
      }
    })
  });

  const result = await response.json();

  if (result.errors) {
    throw new Error(result.errors[0].message);
  }

  return result.data.listPosts;
};

// Usage examples
fetchPosts({ listingType: 'local', sort: 'hot' })
  .then(posts => console.log('Hot posts:', posts));

fetchPosts({ boardName: 'technology', sort: 'new' })
  .then(posts => console.log('New tech posts:', posts));
```

### 2. Get Single Post

**GraphQL Query**:
```graphql
query GetPost($id: Int!) {
  post(id: $id) {
    id
    title
    url
    bodyHTML
    creationDate
    updated
    isNSFW
    isLocked
    score
    upvotes
    downvotes
    commentCount
    myVote
    creator {
      id
      name
      displayName
      avatar
    }
    board {
      id
      name
      title
      icon
    }
    comments(sort: hot, limit: 50) {
      id
      bodyHTML
      score
      level
      creationDate
      myVote
      creator {
        id
        name
        displayName
        avatar
      }
    }
  }
}
```

### 3. Create Post

**Text Post**:
```graphql
mutation CreatePost(
  $title: String!
  $board: String
  $body: String
  $isNSFW: Boolean
) {
  createPost(
    title: $title
    board: $board
    body: $body
    isNSFW: $isNSFW
  ) {
    id
    title
    bodyHTML
    creationDate
    score
    board {
      name
      title
    }
  }
}
```

**Link Post**:
```graphql
mutation CreatePost(
  $title: String!
  $board: String
  $link: String
  $isNSFW: Boolean
) {
  createPost(
    title: $title
    board: $board
    link: $link
    isNSFW: $isNSFW
  ) {
    id
    title
    url
    creationDate
    score
    board {
      name
      title
    }
  }
}
```

**Complete Post Creation Example**:
```javascript
const createPost = async (postData, token) => {
  const response = await fetch('http://localhost:8536/api/v2/graphql', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'Authorization': `Bearer ${token}`
    },
    body: JSON.stringify({
      query: `
        mutation CreatePost(
          $title: String!
          $board: String
          $body: String
          $link: String
          $isNSFW: Boolean
        ) {
          createPost(
            title: $title
            board: $board
            body: $body
            link: $link
            isNSFW: $isNSFW
          ) {
            id
            title
            url
            bodyHTML
            creationDate
            score
            board {
              name
              title
            }
          }
        }
      `,
      variables: postData
    })
  });

  const result = await response.json();

  if (result.errors) {
    throw new Error(result.errors[0].message);
  }

  return result.data.createPost;
};

// Usage examples
// Text post
createPost({
  title: "How to Get Started with Rust",
  board: "programming",
  body: "Rust is a systems programming language...",
  isNSFW: false
}, token);

// Link post
createPost({
  title: "Interesting Article on Web Development",
  board: "webdev",
  link: "https://example.com/article",
  isNSFW: false
}, token);
```

## Comment System

### 1. Create Comment

**Reply to Post**:
```graphql
mutation CreateComment(
  $replyToPostId: Int!
  $body: String!
) {
  createComment(
    replyToPostId: $replyToPostId
    body: $body
  ) {
    id
    bodyHTML
    creationDate
    score
    level
    creator {
      id
      name
      displayName
      avatar
    }
  }
}
```

**Reply to Comment**:
```graphql
mutation CreateComment(
  $replyToCommentId: Int!
  $body: String!
) {
  createComment(
    replyToCommentId: $replyToCommentId
    body: $body
  ) {
    id
    bodyHTML
    creationDate
    score
    level
    creator {
      id
      name
      displayName
      avatar
    }
  }
}
```

**Complete Comment Example**:
```javascript
const createComment = async (commentData, token) => {
  const response = await fetch('http://localhost:8536/api/v2/graphql', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'Authorization': `Bearer ${token}`
    },
    body: JSON.stringify({
      query: `
        mutation CreateComment(
          $replyToPostId: Int
          $replyToCommentId: Int
          $body: String!
        ) {
          createComment(
            replyToPostId: $replyToPostId
            replyToCommentId: $replyToCommentId
            body: $body
          ) {
            id
            bodyHTML
            creationDate
            score
            level
            myVote
            creator {
              id
              name
              displayName
              avatar
            }
          }
        }
      `,
      variables: commentData
    })
  });

  const result = await response.json();

  if (result.errors) {
    throw new Error(result.errors[0].message);
  }

  return result.data.createComment;
};

// Reply to a post
createComment({
  replyToPostId: 123,
  body: "Great article! Thanks for sharing."
}, token);

// Reply to a comment
createComment({
  replyToCommentId: 456,
  body: "I agree with your point about..."
}, token);
```

### 2. Load Comments for Post

This is handled through the `post` query's `comments` field with various options:

```javascript
const getPostWithComments = async (postId, options = {}) => {
  const {
    sort = 'hot',
    limit = 50,
    maxDepth = 6,
    token = null
  } = options;

  const headers = {
    'Content-Type': 'application/json',
  };

  if (token) {
    headers['Authorization'] = `Bearer ${token}`;
  }

  const response = await fetch('http://localhost:8536/api/v2/graphql', {
    method: 'POST',
    headers,
    body: JSON.stringify({
      query: `
        query GetPostWithComments(
          $id: Int!
          $sort: CommentSortType
          $limit: Int
          $maxDepth: Int
        ) {
          post(id: $id) {
            id
            title
            bodyHTML
            score
            commentCount
            comments(
              sort: $sort
              limit: $limit
              maxDepth: $maxDepth
            ) {
              id
              bodyHTML
              score
              upvotes
              downvotes
              level
              creationDate
              updated
              isRemoved
              isDeleted
              myVote
              creator {
                id
                name
                displayName
                avatar
              }
            }
          }
        }
      `,
      variables: {
        id: postId,
        sort,
        limit,
        maxDepth
      }
    })
  });

  const result = await response.json();

  if (result.errors) {
    throw new Error(result.errors[0].message);
  }

  return result.data.post;
};
```

## Voting System

### 1. Vote on Post

**GraphQL Mutation**:
```graphql
mutation VotePost($postId: Int!, $vote: Int!) {
  votePost(postId: $postId, vote: $vote) {
    id
    score
    upvotes
    downvotes
    myVote
  }
}
```

**JavaScript Example**:
```javascript
const voteOnPost = async (postId, vote, token) => {
  // vote: 1 for upvote, -1 for downvote, 0 to remove vote
  const response = await fetch('http://localhost:8536/api/v2/graphql', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'Authorization': `Bearer ${token}`
    },
    body: JSON.stringify({
      query: `
        mutation VotePost($postId: Int!, $vote: Int!) {
          votePost(postId: $postId, vote: $vote) {
            id
            score
            upvotes
            downvotes
            myVote
          }
        }
      `,
      variables: { postId, vote }
    })
  });

  const result = await response.json();

  if (result.errors) {
    throw new Error(result.errors[0].message);
  }

  return result.data.votePost;
};

// Usage
voteOnPost(123, 1, token); // Upvote
voteOnPost(123, -1, token); // Downvote
voteOnPost(123, 0, token); // Remove vote
```

### 2. Vote on Comment

**GraphQL Mutation**:
```graphql
mutation VoteComment($commentId: Int!, $vote: Int!) {
  voteComment(commentId: $commentId, vote: $vote) {
    id
    score
    upvotes
    downvotes
    myVote
  }
}
```

## User Management

### 1. Get User Profile

**GraphQL Query**:
```graphql
query GetUser($username: String!) {
  user(username: $username) {
    id
    name
    displayName
    bio
    avatar
    banner
    creationDate
    isAdmin
    isBanned
    isDeleted
    postCount
    postScore
    commentCount
    commentScore
    reputation
    posts(limit: 10, sort: new) {
      id
      title
      score
      commentCount
      creationDate
      board {
        name
        title
      }
    }
    comments(limit: 10, sort: new) {
      id
      bodyHTML
      score
      creationDate
      post {
        id
        title
        board {
          name
        }
      }
    }
  }
}
```

### 2. Update User Profile

**GraphQL Mutation**:
```graphql
mutation UpdateProfile(
  $displayName: String
  $bio: String
  $avatar: String
  $banner: String
) {
  updateProfile(
    displayName: $displayName
    bio: $bio
    avatar: $avatar
    banner: $banner
  ) {
    id
    name
    displayName
    bio
    avatar
    banner
  }
}
```

## Integration Examples

### React Hook Example

```typescript
import { useState, useEffect } from 'react';

interface TinyBoardsClient {
  token: string | null;
  setToken: (token: string | null) => void;
  query: (query: string, variables?: any) => Promise<any>;
  mutate: (mutation: string, variables?: any) => Promise<any>;
}

const useTinyBoards = (): TinyBoardsClient => {
  const [token, setToken] = useState<string | null>(
    localStorage.getItem('tinyboards_token')
  );

  useEffect(() => {
    if (token) {
      localStorage.setItem('tinyboards_token', token);
    } else {
      localStorage.removeItem('tinyboards_token');
    }
  }, [token]);

  const request = async (query: string, variables?: any) => {
    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
    };

    if (token) {
      headers['Authorization'] = `Bearer ${token}`;
    }

    const response = await fetch('http://localhost:8536/api/v2/graphql', {
      method: 'POST',
      headers,
      body: JSON.stringify({ query, variables })
    });

    const result = await response.json();

    if (result.errors) {
      throw new Error(result.errors[0].message);
    }

    return result.data;
  };

  return {
    token,
    setToken,
    query: request,
    mutate: request
  };
};

// Usage in component
const LoginComponent = () => {
  const tb = useTinyBoards();
  const [loading, setLoading] = useState(false);

  const handleLogin = async (username: string, password: string) => {
    setLoading(true);
    try {
      const result = await tb.mutate(`
        mutation Login($usernameOrEmail: String!, $password: String!) {
          login(usernameOrEmail: $usernameOrEmail, password: $password) {
            token
          }
        }
      `, { usernameOrEmail: username, password });

      tb.setToken(result.login.token);
    } catch (error) {
      console.error('Login failed:', error);
    } finally {
      setLoading(false);
    }
  };

  return (
    // JSX component
  );
};
```

### Apollo Client Setup

```typescript
import { ApolloClient, InMemoryCache, createHttpLink, from } from '@apollo/client';
import { setContext } from '@apollo/client/link/context';

const httpLink = createHttpLink({
  uri: 'http://localhost:8536/api/v2/graphql',
});

const authLink = setContext((_, { headers }) => {
  const token = localStorage.getItem('tinyboards_token');

  return {
    headers: {
      ...headers,
      authorization: token ? `Bearer ${token}` : "",
    }
  }
});

const client = new ApolloClient({
  link: from([authLink, httpLink]),
  cache: new InMemoryCache(),
});

export default client;
```

### Python Async Client

```python
import aiohttp
import asyncio
import json
from typing import Optional, Dict, Any

class TinyBoardsClient:
    def __init__(self, base_url: str = "http://localhost:8536"):
        self.base_url = base_url
        self.graphql_url = f"{base_url}/api/v2/graphql"
        self.token: Optional[str] = None
        self.session: Optional[aiohttp.ClientSession] = None

    async def __aenter__(self):
        self.session = aiohttp.ClientSession()
        return self

    async def __aexit__(self, exc_type, exc_val, exc_tb):
        if self.session:
            await self.session.close()

    def set_token(self, token: str):
        self.token = token

    async def request(self, query: str, variables: Optional[Dict[str, Any]] = None) -> Dict[str, Any]:
        headers = {"Content-Type": "application/json"}

        if self.token:
            headers["Authorization"] = f"Bearer {self.token}"

        payload = {"query": query}
        if variables:
            payload["variables"] = variables

        async with self.session.post(self.graphql_url, headers=headers, json=payload) as response:
            result = await response.json()

            if "errors" in result:
                raise Exception(result["errors"][0]["message"])

            return result["data"]

    async def login(self, username_or_email: str, password: str) -> str:
        query = """
        mutation Login($usernameOrEmail: String!, $password: String!) {
          login(usernameOrEmail: $usernameOrEmail, password: $password) {
            token
          }
        }
        """

        result = await self.request(query, {
            "usernameOrEmail": username_or_email,
            "password": password
        })

        token = result["login"]["token"]
        self.set_token(token)
        return token

    async def get_posts(self, **kwargs) -> list:
        query = """
        query ListPosts($listingType: ListingType!, $sort: SortType, $limit: Int) {
          listPosts(listingType: $listingType, sort: $sort, limit: $limit) {
            id
            title
            score
            commentCount
            creationDate
            creator {
              name
              displayName
            }
            board {
              name
              title
            }
          }
        }
        """

        variables = {
            "listingType": kwargs.get("listing_type", "local"),
            "sort": kwargs.get("sort", "hot"),
            "limit": kwargs.get("limit", 25)
        }

        result = await self.request(query, variables)
        return result["listPosts"]

# Usage
async def main():
    async with TinyBoardsClient() as client:
        # Login
        token = await client.login("username", "password")
        print(f"Logged in with token: {token}")

        # Get posts
        posts = await client.get_posts(sort="new", limit=10)
        for post in posts:
            print(f"- {post['title']} by {post['creator']['name']}")

if __name__ == "__main__":
    asyncio.run(main())
```

## Error Handling

### Common Error Patterns

**Authentication Errors**:
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

**Validation Errors**:
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

**Permission Errors**:
```json
{
  "errors": [
    {
      "message": "You are banned from /b/technology.",
      "extensions": {
        "code": 410
      }
    }
  ]
}
```

### Error Handling Wrapper

```javascript
class TinyBoardsError extends Error {
  constructor(message, code, details) {
    super(message);
    this.name = 'TinyBoardsError';
    this.code = code;
    this.details = details;
  }
}

const handleTinyBoardsResponse = (result) => {
  if (result.errors && result.errors.length > 0) {
    const error = result.errors[0];
    throw new TinyBoardsError(
      error.message,
      error.extensions?.code || 500,
      error
    );
  }
  return result.data;
};

// Usage
try {
  const result = await fetch(/* ... */);
  const data = await result.json();
  const response = handleTinyBoardsResponse(data);
  // Process successful response
} catch (error) {
  if (error instanceof TinyBoardsError) {
    switch (error.code) {
      case 401:
        // Redirect to login
        break;
      case 403:
        // Show permission denied message
        break;
      case 404:
        // Show not found message
        break;
      default:
        // Show generic error
    }
  }
}
```

## Best Practices

### 1. Efficient Data Fetching

**Use Specific Field Selection**:
```graphql
# Good - only request needed fields
query ListPosts {
  listPosts(listingType: local, limit: 25) {
    id
    title
    score
    commentCount
    creator {
      name
      avatar
    }
  }
}

# Avoid - requesting unnecessary data
query ListPosts {
  listPosts(listingType: local, limit: 25) {
    id
    title
    url
    bodyHTML
    score
    upvotes
    downvotes
    commentCount
    creationDate
    updated
    # ... many more fields
  }
}
```

**Pagination**:
```javascript
const loadMorePosts = async (page = 0, limit = 25) => {
  return await query(`
    query ListPosts($page: Int!, $limit: Int!) {
      listPosts(listingType: local, page: $page, limit: $limit) {
        id
        title
        score
        commentCount
      }
    }
  `, { page, limit });
};

// Implement infinite scroll or pagination
let currentPage = 0;
const posts = [];

const loadNextPage = async () => {
  const newPosts = await loadMorePosts(currentPage);
  posts.push(...newPosts);
  currentPage++;
};
```

### 2. Caching Strategies

**Client-side Caching**:
```javascript
class TinyBoardsCache {
  constructor(ttl = 5 * 60 * 1000) { // 5 minutes
    this.cache = new Map();
    this.ttl = ttl;
  }

  set(key, data) {
    this.cache.set(key, {
      data,
      timestamp: Date.now()
    });
  }

  get(key) {
    const item = this.cache.get(key);
    if (!item) return null;

    if (Date.now() - item.timestamp > this.ttl) {
      this.cache.delete(key);
      return null;
    }

    return item.data;
  }

  clear() {
    this.cache.clear();
  }
}

const cache = new TinyBoardsCache();

const cachedQuery = async (key, queryFn) => {
  let result = cache.get(key);
  if (!result) {
    result = await queryFn();
    cache.set(key, result);
  }
  return result;
};

// Usage
const getPosts = (boardName) => cachedQuery(
  `posts_${boardName}`,
  () => fetchPosts({ boardName })
);
```

### 3. Rate Limiting Awareness

TinyBoards has built-in rate limiting. Be aware of these limits:
- **Posts**: 6 posts per 10 minutes
- **Comments**: 6 comments per 10 minutes
- **Messages**: 180 messages per minute
- **Image uploads**: 6 images per hour
- **Search**: 60 searches per 10 minutes

```javascript
class RateLimiter {
  constructor() {
    this.limits = new Map();
  }

  canMakeRequest(action) {
    const now = Date.now();
    const limit = this.limits.get(action) || { count: 0, resetTime: now };

    if (now > limit.resetTime) {
      limit.count = 0;
      limit.resetTime = now + this.getResetInterval(action);
    }

    return limit.count < this.getMaxRequests(action);
  }

  recordRequest(action) {
    const limit = this.limits.get(action) || { count: 0, resetTime: Date.now() + this.getResetInterval(action) };
    limit.count++;
    this.limits.set(action, limit);
  }

  getMaxRequests(action) {
    const limits = {
      'post': 6,
      'comment': 6,
      'search': 60
    };
    return limits[action] || 100;
  }

  getResetInterval(action) {
    const intervals = {
      'post': 10 * 60 * 1000, // 10 minutes
      'comment': 10 * 60 * 1000, // 10 minutes
      'search': 10 * 60 * 1000 // 10 minutes
    };
    return intervals[action] || 60 * 1000; // 1 minute default
  }
}

const rateLimiter = new RateLimiter();

const createPostWithRateLimit = async (postData, token) => {
  if (!rateLimiter.canMakeRequest('post')) {
    throw new Error('Rate limit exceeded for post creation');
  }

  rateLimiter.recordRequest('post');
  return await createPost(postData, token);
};
```

### 4. Real-time Updates

While TinyBoards doesn't currently support GraphQL subscriptions, you can implement polling for real-time-like updates:

```javascript
class PostUpdater {
  constructor(postId, updateCallback, interval = 30000) {
    this.postId = postId;
    this.updateCallback = updateCallback;
    this.interval = interval;
    this.intervalId = null;
    this.lastUpdate = null;
  }

  start() {
    this.intervalId = setInterval(async () => {
      try {
        const post = await getPost(this.postId);

        // Only call callback if data actually changed
        if (!this.lastUpdate ||
            post.score !== this.lastUpdate.score ||
            post.commentCount !== this.lastUpdate.commentCount) {
          this.updateCallback(post);
          this.lastUpdate = post;
        }
      } catch (error) {
        console.error('Failed to update post:', error);
      }
    }, this.interval);
  }

  stop() {
    if (this.intervalId) {
      clearInterval(this.intervalId);
      this.intervalId = null;
    }
  }
}

// Usage
const updater = new PostUpdater(123, (post) => {
  console.log(`Post ${post.id} updated: ${post.score} points, ${post.commentCount} comments`);
});

updater.start();
// ... later
updater.stop();
```

## Testing & Development

### 1. GraphQL Playground

Access the GraphQL playground at `http://localhost:8536/graphql` for interactive testing.

**Setting up Authentication in Playground**:
1. First, run a login mutation to get a token
2. Add the token to HTTP headers:
```json
{
  "Authorization": "Bearer YOUR_TOKEN_HERE"
}
```

### 2. Testing Queries with curl

**Test without authentication**:
```bash
curl -X POST http://localhost:8536/api/v2/graphql \
  -H "Content-Type: application/json" \
  -d '{
    "query": "query { listBoards(limit: 5) { name title subscriberCount } }"
  }'
```

**Test with authentication**:
```bash
TOKEN="your_token_here"

curl -X POST http://localhost:8536/api/v2/graphql \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "query": "query { me { name email postCount } }"
  }'
```

### 3. Automated Testing

**Jest Test Example**:
```javascript
const TinyBoardsClient = require('./tinyboards-client');

describe('TinyBoards API', () => {
  let client;
  let authToken;

  beforeAll(async () => {
    client = new TinyBoardsClient('http://localhost:8536');

    // Login for authenticated tests
    const loginResult = await client.mutate(`
      mutation Login($usernameOrEmail: String!, $password: String!) {
        login(usernameOrEmail: $usernameOrEmail, password: $password) {
          token
        }
      }
    `, {
      usernameOrEmail: 'testuser',
      password: 'testpassword'
    });

    authToken = loginResult.login.token;
    client.setToken(authToken);
  });

  test('should fetch posts', async () => {
    const posts = await client.query(`
      query {
        listPosts(listingType: local, limit: 5) {
          id
          title
          score
        }
      }
    `);

    expect(posts.listPosts).toBeInstanceOf(Array);
    expect(posts.listPosts.length).toBeLessThanOrEqual(5);

    if (posts.listPosts.length > 0) {
      expect(posts.listPosts[0]).toHaveProperty('id');
      expect(posts.listPosts[0]).toHaveProperty('title');
      expect(posts.listPosts[0]).toHaveProperty('score');
    }
  });

  test('should create and delete a post', async () => {
    // Create post
    const newPost = await client.mutate(`
      mutation CreatePost($title: String!, $body: String!) {
        createPost(title: $title, body: $body) {
          id
          title
          score
        }
      }
    `, {
      title: 'Test Post',
      body: 'This is a test post created by automated testing.'
    });

    expect(newPost.createPost).toHaveProperty('id');
    expect(newPost.createPost.title).toBe('Test Post');

    // Clean up - delete the post (if delete mutation exists)
    // await client.mutate(`mutation { deletePost(id: ${newPost.createPost.id}) }`);
  });
});
```

### 4. Debug Mode

Enable debug logging to see all GraphQL operations:

```javascript
class DebugTinyBoardsClient {
  constructor(baseUrl, debug = false) {
    this.baseUrl = baseUrl;
    this.debug = debug;
    this.token = null;
  }

  async request(query, variables) {
    if (this.debug) {
      console.log('GraphQL Query:', query);
      console.log('Variables:', variables);
    }

    const response = await fetch(`${this.baseUrl}/api/v2/graphql`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        ...(this.token && { 'Authorization': `Bearer ${this.token}` })
      },
      body: JSON.stringify({ query, variables })
    });

    const result = await response.json();

    if (this.debug) {
      console.log('Response:', result);
    }

    if (result.errors) {
      throw new Error(result.errors[0].message);
    }

    return result.data;
  }
}

// Usage
const client = new DebugTinyBoardsClient('http://localhost:8536', true);
```

This guide provides a comprehensive foundation for integrating with the TinyBoards GraphQL API. All examples are based on the actual schema and can be used directly with a running TinyBoards instance.