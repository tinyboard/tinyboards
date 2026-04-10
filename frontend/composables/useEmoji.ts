import { ref } from 'vue'
import { useGraphQL } from '~/composables/useGraphQL'

export interface CustomEmoji {
  id: string
  shortcode: string
  imageUrl: string
  category: string | null
  scope?: string
  boardId?: string | null
}

const EMOJIS_QUERY = `
  query ListEmojis($input: ListEmojisInput) {
    listEmojis(input: $input) {
      id shortcode imageUrl category scope boardId
    }
  }
`

export function useEmoji () {
  const { execute, loading, error } = useGraphQL<{ listEmojis: CustomEmoji[] }>()
  const emojis = ref<CustomEmoji[]>([])

  /** Fetch all site-wide custom emojis (no board filter). */
  async function fetchEmojis (): Promise<void> {
    const result = await execute(EMOJIS_QUERY)
    if (result?.listEmojis) {
      emojis.value = result.listEmojis
    }
  }

  /** Fetch emojis scoped to a specific board only (not site-wide). */
  async function fetchBoardEmojis (boardId: string): Promise<void> {
    const result = await execute(EMOJIS_QUERY, {
      variables: { input: { boardId, scope: 'Board' } },
    })
    if (result?.listEmojis) {
      emojis.value = result.listEmojis
    }
  }

  /** Fetch all emojis available in a board context (site-wide + board-specific). */
  async function fetchAllAvailableEmojis (boardId?: string): Promise<void> {
    const input: Record<string, unknown> = {}
    if (boardId) {
      input.boardId = boardId
    }
    const result = await execute(EMOJIS_QUERY, {
      variables: { input },
    })
    if (result?.listEmojis) {
      emojis.value = result.listEmojis
    }
  }

  function findEmoji (shortcode: string): CustomEmoji | undefined {
    return emojis.value.find(e => e.shortcode === shortcode)
  }

  return {
    emojis,
    loading,
    error,
    fetchEmojis,
    fetchBoardEmojis,
    fetchAllAvailableEmojis,
    findEmoji,
  }
}
