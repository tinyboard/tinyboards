import { ref } from 'vue'
import type { Ref } from 'vue'
import { useGraphQL } from '~/composables/useGraphQL'
import type { ApiError } from '~/types/api'

export interface ParticipantUser {
  id: string
  name: string
  displayName: string | null
  avatar: string | null
}

export interface ActivityEntry {
  postId: string
  postTitle: string
  postSlug: string
  commenterName: string
  commenterAvatar: string | null
  createdAt: string
}

interface CommentResult {
  id: string
  postId: string
  createdAt: string
  creator: {
    id: string
    name: string
    displayName: string | null
    avatar: string | null
  } | null
  post: {
    id: string
    title: string
    slug: string
  }
}

interface CommentsResponse {
  comments: CommentResult[]
}

const BOARD_COMMENTS_QUERY = `
  query BoardRecentComments($boardName: String!, $sort: CommentSortType, $limit: Int) {
    comments(boardName: $boardName, sort: $sort, limit: $limit) {
      id
      postId
      createdAt
      creator {
        id
        name
        displayName
        avatar
      }
      post {
        id
        title
        slug
      }
    }
  }
`

interface UseBoardActivityReturn {
  threadParticipants: Ref<Map<string, ParticipantUser[]>>
  threadLastReply: Ref<Map<string, { creatorName: string; createdAt: string }>>
  latestActivity: Ref<ActivityEntry[]>
  loading: Ref<boolean>
  error: Ref<ApiError | null>
  fetchActivity: () => Promise<void>
}

export function useBoardActivity (boardName: string): UseBoardActivityReturn {
  const { execute, loading, error } = useGraphQL<CommentsResponse>()

  const threadParticipants = ref<Map<string, ParticipantUser[]>>(new Map())
  const threadLastReply = ref<Map<string, { creatorName: string; createdAt: string }>>(new Map())
  const latestActivity = ref<ActivityEntry[]>([])

  async function fetchActivity (): Promise<void> {
    const result = await execute(BOARD_COMMENTS_QUERY, {
      variables: {
        boardName,
        sort: 'new',
        limit: 50,
      },
    })

    if (!result?.comments) return

    const comments = result.comments
    const participantsMap = new Map<string, ParticipantUser[]>()
    const lastReplyMap = new Map<string, { creatorName: string; createdAt: string }>()
    const activityList: ActivityEntry[] = []
    const seenPosts = new Set<string>()

    for (const comment of comments) {
      if (!comment.creator) continue

      const postId = comment.postId

      // Build last reply map (first occurrence per post = most recent since sorted by new)
      if (!lastReplyMap.has(postId)) {
        lastReplyMap.set(postId, {
          creatorName: comment.creator.displayName || comment.creator.name,
          createdAt: comment.createdAt,
        })
      }

      // Build latest activity list (deduplicated by post, max 5)
      if (!seenPosts.has(postId) && activityList.length < 5) {
        seenPosts.add(postId)
        activityList.push({
          postId,
          postTitle: comment.post.title,
          postSlug: comment.post.slug,
          commenterName: comment.creator.displayName || comment.creator.name,
          commenterAvatar: comment.creator.avatar,
          createdAt: comment.createdAt,
        })
      }

      // Build participants map (distinct by creator per post, max 5 per post)
      if (!participantsMap.has(postId)) {
        participantsMap.set(postId, [])
      }
      const participants = participantsMap.get(postId)!
      const alreadyAdded = participants.some(p => p.id === comment.creator!.id)
      if (!alreadyAdded && participants.length < 5) {
        participants.push({
          id: comment.creator.id,
          name: comment.creator.name,
          displayName: comment.creator.displayName,
          avatar: comment.creator.avatar,
        })
      }
    }

    threadParticipants.value = participantsMap
    threadLastReply.value = lastReplyMap
    latestActivity.value = activityList
  }

  return {
    threadParticipants,
    threadLastReply,
    latestActivity,
    loading,
    error,
    fetchActivity,
  }
}
