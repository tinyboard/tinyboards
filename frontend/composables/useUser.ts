import { ref } from 'vue'
import type { Ref } from 'vue'
import { useGraphQL } from '~/composables/useGraphQL'
import type { User } from '~/types/generated'
import type { ApiError } from '~/types/api'

const USER_QUERY = `
  query GetUser($username: String!) {
    user(username: $username) {
      id
      name
      displayName
      avatar
      banner
      bio
      bioHTML
      adminLevel
      createdAt
      postCount
      commentCount
      postScore
      commentScore
      isBanned
    }
  }
`

interface UserResponse {
  user: User
}

interface UseUserReturn {
  user: Ref<User | null>
  loading: Ref<boolean>
  error: Ref<ApiError | null>
  fetchUser: (username: string) => Promise<void>
}

export function useUser (): UseUserReturn {
  const { execute, loading, error } = useGraphQL<UserResponse>()
  const user = ref<User | null>(null)

  async function fetchUser (username: string): Promise<void> {
    const result = await execute(USER_QUERY, { variables: { username } })
    if (result?.user) {
      user.value = result.user
    }
  }

  return { user, loading, error, fetchUser }
}
