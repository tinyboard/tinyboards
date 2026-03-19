import { ref } from 'vue'
import { useGraphQL } from '~/composables/useGraphQL'

const SEARCH_USERNAMES_QUERY = `
  query SearchUsernames($query: String!, $limit: Int) {
    searchUsernames(query: $query, limit: $limit)
  }
`

export function useMentionAutocomplete () {
  const { execute, loading, error } = useGraphQL<{ searchUsernames: string[] }>()
  const suggestions = ref<string[]>([])

  let debounceTimer: ReturnType<typeof setTimeout> | null = null

  async function search (query: string): Promise<void> {
    if (query.length < 1) {
      suggestions.value = []
      return
    }

    if (debounceTimer) { clearTimeout(debounceTimer) }

    debounceTimer = setTimeout(async () => {
      const result = await execute(SEARCH_USERNAMES_QUERY, {
        variables: { query, limit: 8 },
      })
      if (result?.searchUsernames) {
        suggestions.value = result.searchUsernames
      }
    }, 200)
  }

  function clear (): void {
    suggestions.value = []
  }

  return {
    suggestions,
    loading,
    error,
    search,
    clear,
  }
}
