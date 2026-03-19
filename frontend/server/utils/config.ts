import { readFileSync } from 'node:fs'
import hjson from 'hjson'

export interface TinyboardsConfig {
  database: {
    user: string
    password: string
    host: string
    port: number
    database: string
    pool_size: number
  }
  hostname: string
  bind: string
  port: number
  tls_enabled: boolean
  environment: string
  name_max_length: number
  salt_suffix: string
  setup?: {
    admin_username: string
    admin_password: string
    site_name: string
    default_board_name: string
    default_board_description?: string
  }
  cors: {
    allowed_origins: string[]
    allow_credentials: boolean
    max_age: number
    allowed_methods: string[]
    allowed_headers: string[]
  }
  rate_limit?: Record<string, number>
  captcha: { enabled: boolean; difficulty: string }
  media: {
    media_path: string
    max_file_size_mb: number
    max_avatar_size_mb: number
    max_banner_size_mb: number
    max_board_icon_size_mb: number
    max_board_banner_size_mb: number
    max_site_icon_size_mb: number
  }
  storage: {
    backend: string
    fs?: { root: string }
    s3?: { bucket: string; region: string; access_key_id: string; secret_access_key: string; endpoint?: string }
    azure?: { container: string; account_name: string; account_key: string }
    gcs?: { bucket: string; credential: string }
  }
  email?: {
    smtp_server: string
    smtp_login?: string
    smtp_password?: string
    smtp_from_address: string
    tls_type: string
  }
  frontend?: {
    use_https: boolean
    internal_api_host: string
  }
  logging?: {
    rust_log: string
  }
  docker?: {
    image_tag: string
    compose_project_name: string
  }
}

let cached: TinyboardsConfig | null = null

/**
 * Load and cache the tinyboards.hjson configuration.
 * Looks for TB_CONFIG_LOCATION env var first, then common default paths.
 */
export function loadConfig(): TinyboardsConfig {
  if (cached) return cached

  const configPath = process.env.TB_CONFIG_LOCATION
    || '/app/config/defaults.hjson'

  const raw = readFileSync(configPath, 'utf-8')
  cached = hjson.parse(raw) as TinyboardsConfig
  return cached
}
