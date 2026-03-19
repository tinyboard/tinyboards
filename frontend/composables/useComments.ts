import { ref } from 'vue'
import type { Ref } from 'vue'
import { useGraphQL } from '~/composables/useGraphQL'
import { useToast } from '~/composables/useToast'
import type { Comment } from '~/types/generated'
import type { ApiError } from '~/types/api'

const COMMENTS_QUERY = `
  query GetComments($postId: ID, $sort: CommentSortType, $page: Int, $limit: Int) {
    comments(postId: $postId, sort: $sort, page: $page, limit: $limit) {
      id
      body
      bodyHTML
      createdAt
      updatedAt
      isDeleted
      isRemoved
      isLocked
      isPinned
      level
      parentId
      score
      upvotes
      downvotes
      replyCount
      myVote
      creator {
        id
        name
        displayName
        avatar
      }
    }
  }
`

const CREATE_COMMENT_MUTATION = `
  mutation CreateComment($postId: ID!, $body: String!, $parentId: ID) {
    createComment(postId: $postId, body: $body, parentId: $parentId) {
      id
      body
      bodyHTML
      createdAt
      level
      parentId
      score
      upvotes
      downvotes
      myVote
      creator {
        id
        name
        displayName
        avatar
      }
    }
  }
`

interface CommentsResponse {
  comments: Comment[]
}

interface CreateCommentResponse {
  createComment: Comment
}

interface UseCommentsReturn {
  comments: Ref<Comment[]>
  loading: Ref<boolean>
  error: Ref<ApiError | null>
  fetchComments: (postId: string) => Promise<void>
  submitComment: (postId: string, body: string, parentId?: string) => Promise<Comment | null>
}

export function useComments (): UseCommentsReturn {
  const { execute, loading, error } = useGraphQL<CommentsResponse>()
  const comments = ref<Comment[]>([])

  async function fetchComments (postId: string): Promise<void> {
    const result = await execute(COMMENTS_QUERY, {
      variables: { postId, sort: 'hot', limit: 50 },
    })
    if (result?.comments) {
      comments.value = result.comments
    }
  }

  async function submitComment (postId: string, body: string, parentId?: string): Promise<Comment | null> {
    const toast = useToast()
    const { execute: exec, error: submitError } = useGraphQL<CreateCommentResponse>()
    const result = await exec(CREATE_COMMENT_MUTATION, {
      variables: { postId, body, parentId },
    })

    if (submitError.value || !result?.createComment) {
      toast.error('Failed to post comment')
      return null
    }

    toast.success('Comment posted')
    return result.createComment
  }

  return { comments, loading, error, fetchComments, submitComment }
}
