import { ref } from 'vue'
import { useGraphQL } from '~/composables/useGraphQL'
import type { Post } from '~/types/generated'

const STREAM_FEED_QUERY = `
  query GetStreamFeed($boardName: String, $sort: SortType, $page: Int, $limit: Int) {
    listPosts(boardName: $boardName, sort: $sort, page: $page, limit: $limit) {
      id title body url createdAt updatedAt isDeleted isRemoved isLocked isFeaturedBoard isNSFW slug
      score upvotes downvotes commentCount myVote isSaved
      board { id name title icon }
      creator { id name displayName avatar }
    }
  }
`

interface StreamFeedResponse {
  listPosts: Post[]
}

export function useStreamFeed () {
  const { execute, loading, error } = useGraphQL<StreamFeedResponse>()
  const posts = ref<Post[]>([])
  const page = ref(1)
  const sort = ref<string>('hot')
  const limit = 25

  const hasMore = ref(false)

  async function fetchFeed (boardNames: string[]): Promise<void> {
    if (boardNames.length === 0) {
      posts.value = []
      hasMore.value = false
      return
    }
    // Fetch posts from each board in the stream in parallel and merge
    const perBoard = Math.ceil((limit + 1) / boardNames.length)
    const results = await Promise.all(
      boardNames.map((boardName) => {
        const { execute: exec } = useGraphQL<StreamFeedResponse>()
        return exec(STREAM_FEED_QUERY, {
          variables: { boardName, sort: sort.value, page: page.value, limit: perBoard },
        })
      }),
    )
    const allPosts: Post[] = []
    for (const result of results) {
      if (result?.listPosts) {
        allPosts.push(...result.listPosts)
      }
    }
    // Sort merged results by date
    const sorted = allPosts.sort((a, b) => new Date(b.createdAt).getTime() - new Date(a.createdAt).getTime())
    hasMore.value = sorted.length > limit
    posts.value = sorted.slice(0, limit)
  }

  async function nextPage (boardNames: string[]): Promise<void> {
    if (hasMore.value) {
      page.value++
      await fetchFeed(boardNames)
    }
  }

  return {
    posts,
    loading,
    error,
    page,
    sort,
    hasMore,
    fetchFeed,
    nextPage,
  }
}
