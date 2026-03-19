// API request/response wrapper types
// These types represent the shape of data flowing through the BFF proxy

export interface ApiResponse<T> {
  data: T | null
  errors: ApiError[] | null
}

export interface ApiError {
  message: string
  path?: string[]
  extensions?: Record<string, unknown>
}

export interface PaginationParams {
  page?: number
  limit?: number
}

export interface PaginatedResult<T> {
  items: T[]
  total: number
  page: number
  limit: number
  hasMore: boolean
}

// REST auth types (login/register are REST-only, not in GraphQL schema)
export type LoginInput = {
  usernameOrEmail: string
  password: string
}

export type RegisterInput = {
  username: string
  password: string
  displayName?: string
  email?: string
  inviteCode?: string
  applicationAnswer?: string
}

export type AuthRestResponse = {
  success: boolean
  message?: string | null
  user?: {
    id: string
    name: string
    is_admin: boolean
    admin_level: number
  } | null
}

export type RegisterRestResponse = {
  success: boolean
  account_created: boolean
  application_submitted: boolean
  user?: {
    id: string
    name: string
    is_admin: boolean
    admin_level: number
  } | null
  message?: string | null
}
