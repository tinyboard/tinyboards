# Configuration Reference

TinyBoards uses a single configuration file — `tinyboards.hjson` — as the source of truth for all settings. This file lives in the project root and controls database connections, security, email, storage, and everything else.

The `.env` file still exists but is **auto-generated** by `configure.sh` from your hjson config. Do not edit `.env` directly — your changes will be overwritten the next time `configure.sh` runs.

The backend itself reads only three environment variables:

| Variable | Default | Description |
|----------|---------|-------------|
| `TB_CONFIG_LOCATION` | `./tinyboards.hjson` | Path to the hjson configuration file. |
| `RUST_LOG` | `info` | Log level filter. Format: `level` or `module=level`. Example: `info,tinyboards_api=debug,tinyboards_db=warn` |
| `TLS_ENABLED` | `false` | Whether TLS termination is handled by the backend. Typically `false` when nginx handles SSL. |

Everything else is configured in `tinyboards.hjson`.

## Table of Contents

- [Domain and URL](#domain-and-url)
- [Database](#database)
- [Security](#security)
- [Initial Admin Account](#initial-admin-account)
- [Backend Server](#backend-server)
- [Media and Uploads](#media-and-uploads)
- [Email / SMTP](#email--smtp)
- [Storage Backend](#storage-backend)
- [Redis](#redis)
- [Rate Limiting](#rate-limiting)
- [Site Defaults](#site-defaults)
- [CORS](#cors)
- [SSL / TLS](#ssl--tls)

## Domain and URL

```hjson
{
  domain: boards.example.com
  use_https: true
  site_url: "https://boards.example.com"
}
```

| Key | Required | Default | Description |
|-----|----------|---------|-------------|
| `domain` | Yes | `example.com` | Public domain for your instance. No protocol prefix. |
| `use_https` | No | `true` | Whether the site uses HTTPS. Set to `false` for local development. |
| `site_url` | No | `https://${domain}` | Full public URL used in meta tags, emails, and link generation. |

## Database

```hjson
{
  database: {
    user: tinyboards
    password: your_secure_password_here
    name: tinyboards
    host: postgres
    port: 5432
  }
}
```

| Key | Required | Default | Description |
|-----|----------|---------|-------------|
| `database.user` | Yes | `tinyboards` | PostgreSQL username. |
| `database.password` | Yes | — | PostgreSQL password. Use a strong random value. |
| `database.name` | Yes | `tinyboards` | PostgreSQL database name. |
| `database.host` | No | `postgres` | Database hostname. Use `postgres` for Docker, `localhost` for from-source. |
| `database.port` | No | `5432` | Database port. |

The `DATABASE_URL` connection string is constructed automatically by `configure.sh` and written to `.env`.

## Security

```hjson
{
  security: {
    jwt_secret: your_jwt_secret_here
    salt_suffix: your_salt_here
  }
}
```

| Key | Required | Default | Description |
|-----|----------|---------|-------------|
| `security.jwt_secret` | Yes | — | Secret for signing JWT access tokens. Generate with `openssl rand -base64 48`. Must be at least 32 characters. |
| `security.salt_suffix` | Yes | — | Salt appended to password hashes. Generate with `openssl rand -hex 32`. |

## Initial Admin Account

```hjson
{
  admin: {
    username: admin
    password: your_admin_password
    email: "admin@example.com"
  }
}
```

| Key | Required | Default | Description |
|-----|----------|---------|-------------|
| `admin.username` | First run | `admin` | Username for the initial admin account. |
| `admin.password` | First run | — | Password for the initial admin account. Use something strong. |
| `admin.email` | First run | `admin@example.com` | Email for the initial admin account. |

These values are only used during the seed migration on first startup. Changing them after initial setup has no effect — use the admin panel to manage accounts.

## Backend Server

```hjson
{
  server: {
    hostname: boards.example.com
    media_path: /app/media
  }
}
```

| Key | Required | Default | Description |
|-----|----------|---------|-------------|
| `server.hostname` | No | Value of `domain` | Hostname the backend uses for generating links. |
| `server.media_path` | No | `/app/media` | Path where uploaded media files are stored. In Docker this maps to the `media_data` volume. |

## Media and Uploads

```hjson
{
  media: {
    max_file_size_mb: 50
  }
}
```

| Key | Required | Default | Description |
|-----|----------|---------|-------------|
| `media.max_file_size_mb` | No | `50` | Maximum upload file size in megabytes. Must match `client_max_body_size` in your nginx config. |

## Email / SMTP

```hjson
{
  email: {
    smtp_server: "smtp.mailgun.org:587"
    smtp_login: your_smtp_user
    smtp_password: your_smtp_password
    smtp_from_address: "noreply@boards.example.com"
    tls_type: tls
  }
}
```

| Key | Required | Default | Description |
|-----|----------|---------|-------------|
| `email.smtp_server` | No | — | SMTP server address and port. Example: `smtp.mailgun.org:587` |
| `email.smtp_login` | No | — | SMTP authentication username. |
| `email.smtp_password` | No | — | SMTP authentication password. |
| `email.smtp_from_address` | No | `noreply@${domain}` | Sender address for outgoing emails. |
| `email.tls_type` | No | `tls` | SMTP encryption mode. Options: `none`, `tls`, `starttls` |

SMTP is optional but required for email verification and password reset features. Without SMTP configured, set `require_email_verification` to `false` in site settings.

## Storage Backend

TinyBoards uses [OpenDAL](https://opendal.apache.org/) for file storage. By default, files are stored on the local filesystem. You can configure S3-compatible, Azure Blob, or Google Cloud Storage as alternatives.

### S3-Compatible (AWS S3, Wasabi, Backblaze B2, Cloudflare R2, MinIO)

```hjson
{
  storage: {
    type: s3
    s3: {
      bucket: my-tinyboards-bucket
      access_key: AKIAIOSFODNN7EXAMPLE
      secret_key: wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY
      region: us-east-1
      endpoint: "https://s3.us-west-1.wasabisys.com"
    }
  }
}
```

| Key | Required | Default | Description |
|-----|----------|---------|-------------|
| `storage.type` | No | `local` | Storage backend. Options: `local`, `s3`, `azure`, `gcs` |
| `storage.s3.bucket` | When type is `s3` | — | S3 bucket name. |
| `storage.s3.access_key` | When type is `s3` | — | AWS access key or equivalent. |
| `storage.s3.secret_key` | When type is `s3` | — | AWS secret key or equivalent. |
| `storage.s3.region` | No | `us-east-1` | AWS region or equivalent. |
| `storage.s3.endpoint` | No | — | Custom endpoint URL for non-AWS S3-compatible services. |

> **Wasabi note:** The endpoint must match the bucket's region (e.g., `https://s3.us-west-1.wasabisys.com` for a `us-west-1` bucket).

### Azure Blob Storage

```hjson
{
  storage: {
    type: azure
    azure: {
      account_name: mystorageaccount
      account_key: your_account_key_here
    }
  }
}
```

| Key | Required | Default | Description |
|-----|----------|---------|-------------|
| `storage.azure.account_name` | When type is `azure` | — | Azure storage account name. |
| `storage.azure.account_key` | When type is `azure` | — | Azure storage account key. |

### Google Cloud Storage

```hjson
{
  storage: {
    type: gcs
    gcs: {
      credential_path: /path/to/service-account.json
    }
  }
}
```

| Key | Required | Default | Description |
|-----|----------|---------|-------------|
| `storage.gcs.credential_path` | When type is `gcs` | — | Path to service account JSON key file. |

## Redis

Redis is optional and used for rate limiting and caching.

```hjson
{
  redis: {
    host: redis
    port: 6379
    password: ""
    db: 0
  }
}
```

| Key | Required | Default | Description |
|-----|----------|---------|-------------|
| `redis.host` | No | `redis` | Redis hostname. |
| `redis.port` | No | `6379` | Redis port. |
| `redis.password` | No | — | Redis password. Leave empty if no authentication. |
| `redis.db` | No | `0` | Redis database number. |

## Rate Limiting

Rate limits are specified in requests per minute.

```hjson
{
  rate_limit: {
    message: 1800
    post: 60
    register: 30
    image: 60
    comment: 60
    search: 600
  }
}
```

| Key | Required | Default | Description |
|-----|----------|---------|-------------|
| `rate_limit.message` | No | `1800` | Private message sends per minute. |
| `rate_limit.post` | No | `60` | Post submissions per minute. |
| `rate_limit.register` | No | `30` | Registration attempts per minute. |
| `rate_limit.image` | No | `60` | Image uploads per minute. |
| `rate_limit.comment` | No | `60` | Comment submissions per minute. |
| `rate_limit.search` | No | `600` | Search queries per minute. |

## Site Defaults

These configure the initial site settings. They can be changed later through the admin panel.

```hjson
{
  site: {
    name: TinyBoards
    default_board_name: general
    default_board_description: General discussion
    captcha_enabled: false
    captcha_difficulty: easy
    registration_open: true
  }
}
```

| Key | Required | Default | Description |
|-----|----------|---------|-------------|
| `site.name` | No | `TinyBoards` | Display name for your instance. |
| `site.default_board_name` | No | `general` | Name of the default board created on first run. |
| `site.default_board_description` | No | `General discussion` | Description for the default board. |
| `site.captcha_enabled` | No | `false` | Enable CAPTCHA on registration. |
| `site.captcha_difficulty` | No | `easy` | CAPTCHA difficulty level. Options: `easy`, `medium`, `hard` |
| `site.registration_open` | No | `true` | Whether new user registration is open. |

## CORS

```hjson
{
  cors: {
    allowed_origins: "https://boards.example.com"
    allow_credentials: true
  }
}
```

| Key | Required | Default | Description |
|-----|----------|---------|-------------|
| `cors.allowed_origins` | No | `https://${domain}` | Comma-separated list of allowed CORS origins. |
| `cors.allow_credentials` | No | `true` | Whether to include credentials in CORS responses. |

## SSL / TLS

```hjson
{
  ssl: {
    cert_path: ./certs/fullchain.pem
    key_path: ./certs/privkey.pem
    letsencrypt_email: "you@example.com"
  }
}
```

| Key | Required | Default | Description |
|-----|----------|---------|-------------|
| `ssl.cert_path` | No | `./certs/fullchain.pem` | Path to your SSL certificate (PEM format). |
| `ssl.key_path` | No | `./certs/privkey.pem` | Path to your SSL private key (PEM format). |
| `ssl.letsencrypt_email` | No | — | Email for Let's Encrypt certificate expiry notifications. |

See [ssl.md](ssl.md) for SSL setup instructions.
