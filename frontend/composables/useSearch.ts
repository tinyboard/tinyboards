import { ref, computed } from 'vue'
import { useGraphQL } from '~/composables/useGraphQL'
import type { Post, Comment, User, Board } from '~/types/generated'

const SEARCH_QUERY = `
  query SearchContent($q: String!, $searchType: SearchType, $sort: SortType, $page: Int, $limit: Int) {
    searchContent(q: $q, searchType: $searchType, sort: $sort, page: $page, limit: $limit) {
      posts {
        id title body url createdAt slug score commentCount
        board { id name title icon }
        creator { id name displayName avatar }
      }
      comments {
        id body createdAt score postId
        creator { id name displayName avatar }
      }
      users {
        id name displayName avatar postScore commentScore
      }
      boards {
        id name title description icon subscribers
      }
    }
  }
`

interface SearchResult {
  posts: Post[]
  comments: Comment[]
  users: User[]
  boards: Board[]
}

interface SearchResponse {
  searchContent: SearchResult
}

export function useSearch () {
  const { execute, loading, error } = useGraphQL<SearchResponse>()
  const results = ref<SearchResult | null>(null)
  const page = ref(1)
  const searchType = ref<string>('all')

  async function search (query: string): Promise<void> {
    if (query.trim().length < 2) { return }

    const result = await execute(SEARCH_QUERY, {
      variables: {
        q: query,
        searchType: searchType.value,
        page: page.value,
        limit: 20,
      },
    })
    if (result?.searchContent) {
      results.value = result.searchContent
    }
  }

  function setSearchType (type: string): void {
    searchType.value = type
    page.value = 1
  }

  return {
    results,
    loading,
    error,
    page,
    searchType,
    search,
    setSearchType,
  }
}
