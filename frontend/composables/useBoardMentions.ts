import { ref } from 'vue'
import { useGraphQL } from '~/composables/useGraphQL'
import type { Board } from '~/types/generated'

const SEARCH_BOARDS_QUERY = `
  query SearchBoards($searchTerm: String!, $limit: Int) {
    listBoards(searchTerm: $searchTerm, searchTitleAndDesc: true, limit: $limit) {
      id name title icon
    }
  }
`

export function useBoardMentions () {
  const { execute, loading, error } = useGraphQL<{ listBoards: Board[] }>()
  const suggestions = ref<Board[]>([])

  let debounceTimer: ReturnType<typeof setTimeout> | null = null

  async function search (query: string): Promise<void> {
    if (query.length < 1) {
      suggestions.value = []
      return
    }

    if (debounceTimer) { clearTimeout(debounceTimer) }

    debounceTimer = setTimeout(async () => {
      const result = await execute(SEARCH_BOARDS_QUERY, {
        variables: { searchTerm: query, limit: 8 },
      })
      if (result?.listBoards) {
        suggestions.value = result.listBoards
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
