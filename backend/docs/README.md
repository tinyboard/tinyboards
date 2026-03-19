# TinyBoards Documentation

This directory contains comprehensive documentation for the TinyBoards social media platform.

## API Documentation

### GraphQL API
- **[GraphQL API Guide](api/GRAPHQL_API_GUIDE.md)** - Complete integration guide with examples for all major operations
- **[GraphQL Quick Reference](api/GRAPHQL_QUICK_REFERENCE.md)** - Concise reference with ready-to-use queries and mutations
- **[Authentication & Security](api/API_AUTHENTICATION_SECURITY.md)** - JWT authentication, permissions, and security features
- **[Data Models](api/DATA_MODELS.md)** - Entity relationships and database schema mapping

## Configuration & Deployment

### Storage Backends
- **[Storage Backends Guide](storage-backends.md)** - Configure cloud storage (S3, Azure, GCS) or local filesystem for media uploads
  - Quick start guides for all backends
  - Performance optimization tips
  - Security best practices
  - Migration strategies

## Quick Start

For developers looking to integrate with TinyBoards:

1. **Start with**: [GraphQL API Guide](api/GRAPHQL_API_GUIDE.md) for comprehensive examples
2. **Reference**: [Quick Reference](api/GRAPHQL_QUICK_REFERENCE.md) for specific operations
3. **Security**: [Authentication Guide](api/API_AUTHENTICATION_SECURITY.md) for implementing auth
4. **Data**: [Data Models](api/DATA_MODELS.md) for understanding the schema
5. **Storage**: [Storage Backends](storage-backends.md) for configuring media uploads

## API Endpoint

The GraphQL API is available at: `http://localhost:8536/api/v2/graphql`

GraphQL Playground: `http://localhost:8536/graphql`