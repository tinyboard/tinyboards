import { ref } from 'vue'
import type { Ref } from 'vue'
import { useGraphQL } from '~/composables/useGraphQL'
import type { Post, ListingType, SortType } from '~/types/generated'
import type { ApiError } from '~/types/api'

const POSTS_QUERY = `
  query GetPosts($listingType: ListingType, $sort: SortType, $page: Int, $limit: Int, $boardName: String) {
    listPosts(listingType: $listingType, sort: $sort, page: $page, limit: $limit, boardName: $boardName) {
      id
      title
      body
      bodyHTML
      postType
      url
      createdAt
      updatedAt
      isDeleted
      isRemoved
      isLocked
      isFeaturedBoard
      isFeaturedLocal
      isNSFW
      isThread
      slug
      score
      upvotes
      downvotes
      commentCount
      myVote
      isSaved
      image
      altText
      embedTitle
      embedDescription
      distinguishedAs
      newestCommentTime
      board {
        id
        name
        title
        icon
      }
      creator {
        id
        name
        displayName
        avatar
        isAdmin
      }
    }
  }
`

interface PostsResponse {
  listPosts: Post[]
}

interface UsePostsOptions {
  listingType?: ListingType
  boardName?: string
  /** Filter posts by type. If not set, all types are returned. */
  postType?: 'thread' | 'feed'
  /** Base path for URL sort sync (e.g. '/home', '/all'). If set, sort changes update the URL. */
  basePath?: string
}

interface UsePostsReturn {
  posts: Ref<Post[]>
  loading: Ref<boolean>
  error: Ref<ApiError | null>
  page: Ref<number>
  sort: Ref<string>
  hasMore: Ref<boolean>
  fetchPosts: () => Promise<void>
  nextPage: () => Promise<void>
  prevPage: () => Promise<void>
  setSort: (newSort: SortType | string) => Promise<void>
}

export function usePosts (options: UsePostsOptions = {}): UsePostsReturn {
  const { execute, loading, error } = useGraphQL<PostsResponse>()
  const router = options.basePath ? useRouter() : null

  const posts = ref<Post[]>([])
  const page = ref(1)
  const sort = ref<string>('hot')
  const limit = 25
  const hasMore = ref(false)

  async function fetchPosts (): Promise<void> {
    // Fetch extra posts to account for client-side filtering
    const fetchLimit = options.postType ? (limit + 1) * 3 : limit + 1

    const result = await execute(POSTS_QUERY, {
      variables: {
        listingType: options.listingType ?? 'all',
        sort: sort.value,
        page: page.value,
        limit: fetchLimit,
        boardName: options.boardName,
      },
    })

    if (result?.listPosts) {
      let filtered = result.listPosts

      // Client-side section filtering using the is_thread flag
      if (options.postType === 'thread') {
        filtered = filtered.filter(p => p.isThread)
      } else if (options.postType === 'feed') {
        filtered = filtered.filter(p => !p.isThread)
      }

      hasMore.value = filtered.length > limit
      posts.value = filtered.slice(0, limit)
    }
  }

  async function nextPage (): Promise<void> {
    if (hasMore.value) {
      page.value++
      await fetchPosts()
    }
  }

  async function prevPage (): Promise<void> {
    if (page.value > 1) {
      page.value--
      await fetchPosts()
    }
  }

  async function setSort (newSort: SortType | string): Promise<void> {
    sort.value = newSort
    page.value = 1

    // Sync sort to URL if basePath is configured
    if (router && import.meta.client) {
      const newPath = newSort === 'hot'
        ? options.basePath!
        : `${options.basePath}/${newSort}`
      router.replace({ path: newPath })
    }

    await fetchPosts()
  }

  return {
    posts,
    loading,
    error,
    page,
    sort,
    hasMore,
    fetchPosts,
    nextPage,
    prevPage,
    setSort,
  }
}
