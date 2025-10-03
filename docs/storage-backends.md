# Storage Backends

TinyBoards supports multiple storage backends for media uploads, allowing you to choose between local filesystem storage or cloud storage providers.

## Overview

TinyBoards uses [Apache OpenDAL](https://opendal.apache.org/) to provide a unified storage interface that supports:

- **Local Filesystem** - Store files on your server's disk
- **AWS S3** - Amazon S3 or S3-compatible services (MinIO, DigitalOcean Spaces, Cloudflare R2, etc.)
- **Azure Blob Storage** - Microsoft Azure cloud storage
- **Google Cloud Storage** - Google Cloud Platform storage

This allows you to:
- ✅ Scale storage independently of your application
- ✅ Use CDN for faster media delivery
- ✅ Reduce server storage costs
- ✅ Improve upload/download performance with streaming

## Quick Start

### Filesystem (Default)

No configuration needed! Files are stored in the `media/` directory by default.

```hjson
storage: {
  backend: "fs"
  fs: {
    root: "media"
  }
}
```

### AWS S3

1. Create an S3 bucket
2. Create IAM user with S3 permissions
3. Set environment variables:
   ```bash
   export S3_ACCESS_KEY_ID="your-access-key"
   export S3_SECRET_KEY="your-secret-key"
   ```
4. Update `config/defaults.hjson`:
   ```hjson
   storage: {
     backend: "s3"
     s3: {
       bucket: "tinyboards-media"
       region: "us-east-1"
       access_key_id: "${S3_ACCESS_KEY_ID}"
       secret_access_key: "${S3_SECRET_KEY}"
     }
   }
   ```
5. Restart TinyBoards

### Azure Blob Storage

1. Create Azure Storage Account and container
2. Get account name and key from Azure Portal
3. Set environment variables:
   ```bash
   export AZURE_ACCOUNT_NAME="yourstorageaccount"
   export AZURE_ACCOUNT_KEY="your-account-key"
   ```
4. Update config:
   ```hjson
   storage: {
     backend: "azure"
     azure: {
       container: "tinyboards-media"
       account_name: "${AZURE_ACCOUNT_NAME}"
       account_key: "${AZURE_ACCOUNT_KEY}"
     }
   }
   ```

### Google Cloud Storage

1. Create GCS bucket
2. Create service account with "Storage Object Admin" role
3. Download JSON key file
4. Set environment variable:
   ```bash
   export GCS_CREDENTIAL_PATH="/path/to/service-account.json"
   ```
5. Update config:
   ```hjson
   storage: {
     backend: "gcs"
     gcs: {
       bucket: "tinyboards-media"
       credential: "${GCS_CREDENTIAL_PATH}"
     }
   }
   ```

## File Organization

All backends use the same directory structure:

```
avatars/          # User profile pictures
emojis/           # Custom emoji images
videos/           # Video uploads
audio/            # Audio files
documents/        # PDF and text files
```

## Performance Benefits

### Memory-Efficient Streaming

The new OpenDAL implementation uses **streaming uploads** instead of buffering entire files:

| File Size | Old Implementation | New Implementation |
|-----------|-------------------|-------------------|
| 10 MB     | 10 MB RAM         | ~10 MB RAM        |
| 100 MB    | 100 MB RAM        | ~40 MB RAM        |
| 500 MB    | 500 MB RAM        | ~40 MB RAM        |

**How it works:**
- Files are uploaded in 8MB chunks
- Up to 4 chunks processed concurrently
- Maximum memory usage: ~40MB regardless of file size

### Concurrent Uploads

Large files are uploaded using multipart upload with concurrent chunks:
- **S3/Azure/GCS**: Automatic multipart upload
- **Configurable**: Chunk size and concurrency
- **Faster uploads**: Parallel processing

## S3-Compatible Services

The S3 backend works with any S3-compatible service:

### MinIO (Self-hosted)
```hjson
s3: {
  bucket: "tinyboards"
  region: "us-east-1"
  access_key_id: "${MINIO_ACCESS_KEY}"
  secret_access_key: "${MINIO_SECRET_KEY}"
  endpoint: "http://localhost:9000"
}
```

### DigitalOcean Spaces
```hjson
s3: {
  bucket: "tinyboards-media"
  region: "nyc3"
  access_key_id: "${DO_SPACES_KEY}"
  secret_access_key: "${DO_SPACES_SECRET}"
  endpoint: "https://nyc3.digitaloceanspaces.com"
}
```

### Cloudflare R2
```hjson
s3: {
  bucket: "tinyboards-media"
  region: "auto"
  access_key_id: "${CF_R2_ACCESS_KEY}"
  secret_access_key: "${CF_R2_SECRET_KEY}"
  endpoint: "https://<account-id>.r2.cloudflarestorage.com"
}
```

## Docker Deployment

The Docker configuration (`docker/tinyboards.hjson`) is already set up for storage backends. To use cloud storage:

1. **Uncomment** the desired backend in `docker/tinyboards.hjson`
2. **Set environment variables** in `docker-compose.yml`:
   ```yaml
   environment:
     - S3_ACCESS_KEY_ID=your-key
     - S3_SECRET_KEY=your-secret
   ```
3. **Restart** containers: `docker-compose restart`

## Migration from Local to Cloud

**Important:** There is currently no automated migration tool.

### Recommended Approach

**For new deployments:** Configure cloud storage from the beginning.

**For existing deployments:** Manual migration required.

1. Sync files to cloud storage using CLI tools:
   ```bash
   # AWS S3
   aws s3 sync ./media/ s3://tinyboards-media/

   # Azure
   az storage blob upload-batch -d tinyboards-media -s ./media/

   # GCS
   gsutil -m rsync -r ./media/ gs://tinyboards-media/
   ```

2. Update database URLs (SQL migration - contact support for script)

3. Switch backend in config and restart

## Security Best Practices

### IAM Permissions (S3)

Create minimal IAM policy:
```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Action": [
        "s3:PutObject",
        "s3:GetObject",
        "s3:DeleteObject",
        "s3:ListBucket"
      ],
      "Resource": [
        "arn:aws:s3:::tinyboards-media/*",
        "arn:aws:s3:::tinyboards-media"
      ]
    }
  ]
}
```

### Bucket Security

- ✅ Enable encryption at rest
- ✅ Use HTTPS endpoints only
- ✅ Restrict public access (use signed URLs for sensitive content)
- ✅ Enable versioning for backups
- ✅ Set up lifecycle policies

### Credentials Management

- ✅ Use environment variables for secrets
- ✅ Never commit credentials to git
- ✅ Rotate keys regularly
- ✅ Use IAM roles when possible (AWS EC2, ECS, etc.)

## Troubleshooting

### Common Issues

**"Failed to initialize storage backend"**
- Check environment variables are set correctly
- Verify credentials have proper permissions
- Ensure bucket/container exists

**"Access Denied" errors**
- Review IAM/storage permissions
- Check bucket policies
- Verify credentials are not expired

**"Bucket does not exist"**
- Create bucket in cloud provider console
- Verify bucket name in config matches exactly
- Check region is correct

**Large uploads fail**
- Increase request timeout in reverse proxy (nginx, caddy)
- Check network connectivity to storage backend
- Verify storage quota is not exceeded

### Enable Debug Logging

```bash
RUST_LOG=debug cargo run
```

Look for:
```
Storage backend initialized: S3
File uploaded: avatars/avatar_123.png (12345 bytes)
```

## Cost Optimization

### Storage Tiers

**AWS S3:**
- Standard: Frequent access
- Infrequent Access (IA): Less frequent, cheaper
- Glacier: Archive, very cheap

**Azure:**
- Hot: Frequent access
- Cool: Infrequent access
- Archive: Long-term storage

**GCS:**
- Standard: Frequent access
- Nearline: Once per month
- Coldline: Once per quarter
- Archive: Once per year

### Lifecycle Policies

Automatically move old files to cheaper storage:

**S3 Example:**
```json
{
  "Rules": [
    {
      "Id": "ArchiveOldUploads",
      "Status": "Enabled",
      "Transitions": [
        {
          "Days": 90,
          "StorageClass": "STANDARD_IA"
        },
        {
          "Days": 365,
          "StorageClass": "GLACIER"
        }
      ]
    }
  ]
}
```

## Performance Optimization

### CDN Integration

Use CDN for faster global delivery:

**CloudFront (AWS S3):**
1. Create CloudFront distribution
2. Set S3 bucket as origin
3. Update TinyBoards URL generation (future feature)

**Azure CDN:**
1. Enable Azure CDN for storage account
2. Configure custom domain
3. Update configuration

**Cloud CDN (GCS):**
1. Enable Cloud CDN
2. Configure load balancer
3. Update configuration

### Caching Headers

Files are served with appropriate cache headers:
- Images/videos: Long cache (1 year)
- User uploads: Versioned URLs
- Browser caching enabled

## Monitoring & Metrics

### What to Monitor

- **Storage usage**: Track growth over time
- **Request count**: PUT/GET operations
- **Data transfer**: Bandwidth usage
- **Error rates**: Failed uploads/downloads
- **Costs**: Monthly expenses

### Tools

- **AWS CloudWatch**: S3 metrics
- **Azure Monitor**: Storage analytics
- **GCP Cloud Monitoring**: GCS metrics
- **OpenDAL Metrics**: Built-in Prometheus support (future)

## API Reference

See [STORAGE_BACKENDS.md](../STORAGE_BACKENDS.md) for complete configuration reference.

## Support

- [GitHub Issues](https://github.com/tinyboards/tinyboards/issues)
- [Implementation Plan](../OPENDAL_INTEGRATION_PLAN.md)
- [Frontend Integration](../../tinyboards-fe/OPENDAL_FRONTEND_IMPLEMENTATION.md)

---

**Last Updated:** 2025-10-02
