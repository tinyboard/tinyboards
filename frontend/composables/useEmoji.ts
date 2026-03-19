import { ref } from 'vue'
import { useGraphQL } from '~/composables/useGraphQL'

interface CustomEmoji {
  id: string
  shortcode: string
  imageUrl: string
  category: string | null
}

const EMOJIS_QUERY = `
  query ListEmojis {
    listEmojis {
      id shortcode imageUrl category
    }
  }
`

export function useEmoji () {
  const { execute, loading, error } = useGraphQL<{ listEmojis: CustomEmoji[] }>()
  const emojis = ref<CustomEmoji[]>([])

  async function fetchEmojis (): Promise<void> {
    const result = await execute(EMOJIS_QUERY)
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
    findEmoji,
  }
}
