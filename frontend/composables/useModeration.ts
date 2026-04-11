import { ref } from 'vue'
import { useGraphQL } from '~/composables/useGraphQL'
import { useToast } from '~/composables/useToast'

interface PostReportView {
  id: string
  creatorId: string
  postId: string
  originalPostTitle: string
  originalPostUrl: string | null
  originalPostBody: string | null
  reason: string
  status: string
  resolverId: string | null
  createdAt: string
  updatedAt: string
}

interface CommentReportView {
  id: string
  creatorId: string
  commentId: string
  originalCommentText: string
  reason: string
  status: string
  resolverId: string | null
  createdAt: string
  updatedAt: string
}

const POST_REPORTS_QUERY = `
  query GetPostReports($boardId: ID, $statusFilter: String, $limit: Int, $offset: Int) {
    getPostReports(boardId: $boardId, statusFilter: $statusFilter, limit: $limit, offset: $offset) {
      id creatorId postId originalPostTitle originalPostUrl originalPostBody reason status resolverId createdAt updatedAt
    }
  }
`

const COMMENT_REPORTS_QUERY = `
  query GetCommentReports($boardId: ID, $statusFilter: String, $limit: Int, $offset: Int) {
    getCommentReports(boardId: $boardId, statusFilter: $statusFilter, limit: $limit, offset: $offset) {
      id creatorId commentId originalCommentText reason status resolverId createdAt updatedAt
    }
  }
`

const REMOVE_POST_MUTATION = `
  mutation RemovePost($postId: ID!, $reason: String) {
    removePost(postId: $postId, reason: $reason) { id isRemoved }
  }
`

const RESTORE_POST_MUTATION = `
  mutation RestorePost($postId: ID!) {
    restorePost(postId: $postId) { id isRemoved }
  }
`

const REMOVE_COMMENT_MUTATION = `
  mutation RemoveComment($commentId: ID!, $reason: String) {
    removeComment(commentId: $commentId, reason: $reason) { id isRemoved }
  }
`

const RESTORE_COMMENT_MUTATION = `
  mutation RestoreComment($commentId: ID!) {
    restoreComment(commentId: $commentId) { id isRemoved }
  }
`

const LOCK_POST_MUTATION = `
  mutation LockPost($postId: ID!) {
    lockPost(postId: $postId) { id isLocked }
  }
`

const UNLOCK_POST_MUTATION = `
  mutation UnlockPost($postId: ID!) {
    unlockPost(postId: $postId) { id isLocked }
  }
`

const FEATURE_POST_MUTATION = `
  mutation FeaturePost($postId: ID!, $featured: Boolean!, $featureType: String) {
    featurePost(postId: $postId, featured: $featured, featureType: $featureType) { id isFeaturedBoard isFeaturedLocal }
  }
`

const RESOLVE_REPORT_MUTATION = `
  mutation ResolveReport($reportId: ID!, $reportType: String!) {
    resolveReport(reportId: $reportId, reportType: $reportType) { success }
  }
`

const DISMISS_REPORT_MUTATION = `
  mutation DismissReport($reportId: ID!, $reportType: String!) {
    dismissReport(reportId: $reportId, reportType: $reportType) { success }
  }
`

const DISTINGUISH_POST_MUTATION = `
  mutation DistinguishPost($postId: ID!) {
    distinguishPost(postId: $postId) { id distinguishedAs }
  }
`

const DISTINGUISH_COMMENT_MUTATION = `
  mutation DistinguishComment($commentId: ID!) {
    distinguishComment(commentId: $commentId) { id distinguishedAs }
  }
`

const MARK_NSFW_POST_MUTATION = `
  mutation MarkNsfwPost($postId: ID!) {
    markNsfwPost(postId: $postId) { id isNSFW }
  }
`

const UNMARK_NSFW_POST_MUTATION = `
  mutation UnmarkNsfwPost($postId: ID!) {
    unmarkNsfwPost(postId: $postId) { id isNSFW }
  }
`

export { type PostReportView, type CommentReportView }

export function useModeration () {
  const { execute, loading, error } = useGraphQL<{ getPostReports: PostReportView[] }>()
  const postReports = ref<PostReportView[]>([])
  const commentReports = ref<CommentReportView[]>([])

  async function fetchPostReports (options?: { boardId?: string; statusFilter?: string; limit?: number; offset?: number }): Promise<void> {
    const result = await execute(POST_REPORTS_QUERY, {
      variables: {
        boardId: options?.boardId,
        statusFilter: options?.statusFilter ?? 'pending',
        limit: options?.limit ?? 25,
        offset: options?.offset ?? 0,
      },
    })
    if (result?.getPostReports) {
      postReports.value = result.getPostReports
    }
  }

  async function fetchCommentReports (options?: { boardId?: string; statusFilter?: string; limit?: number; offset?: number }): Promise<void> {
    const { execute: exec } = useGraphQL<{ getCommentReports: CommentReportView[] }>()
    const result = await exec(COMMENT_REPORTS_QUERY, {
      variables: {
        boardId: options?.boardId,
        statusFilter: options?.statusFilter ?? 'pending',
        limit: options?.limit ?? 25,
        offset: options?.offset ?? 0,
      },
    })
    if (result?.getCommentReports) {
      commentReports.value = result.getCommentReports
    }
  }

  async function removePost (postId: string, reason?: string): Promise<boolean> {
    const toast = useToast()
    const { execute: exec, error: mutError } = useGraphQL()
    const result = await exec(REMOVE_POST_MUTATION, { variables: { postId, reason: reason ?? null } })
    if (result) { toast.success('Post removed'); return true }
    toast.error(mutError.value?.message ?? 'Failed to remove post')
    return false
  }

  async function restorePost (postId: string): Promise<boolean> {
    const toast = useToast()
    const { execute: exec, error: mutError } = useGraphQL()
    const result = await exec(RESTORE_POST_MUTATION, { variables: { postId } })
    if (result) { toast.success('Post restored'); return true }
    toast.error(mutError.value?.message ?? 'Failed to restore post')
    return false
  }

  async function removeComment (commentId: string, reason?: string): Promise<boolean> {
    const toast = useToast()
    const { execute: exec, error: mutError } = useGraphQL()
    const result = await exec(REMOVE_COMMENT_MUTATION, { variables: { commentId, reason: reason ?? null } })
    if (result) { toast.success('Comment removed'); return true }
    toast.error(mutError.value?.message ?? 'Failed to remove comment')
    return false
  }

  async function restoreComment (commentId: string): Promise<boolean> {
    const toast = useToast()
    const { execute: exec, error: mutError } = useGraphQL()
    const result = await exec(RESTORE_COMMENT_MUTATION, { variables: { commentId } })
    if (result) { toast.success('Comment restored'); return true }
    toast.error(mutError.value?.message ?? 'Failed to restore comment')
    return false
  }

  async function lockPost (postId: string): Promise<boolean> {
    const toast = useToast()
    const { execute: exec, error: mutError } = useGraphQL()
    const result = await exec(LOCK_POST_MUTATION, { variables: { postId } })
    if (result) { toast.success('Post locked'); return true }
    toast.error(mutError.value?.message ?? 'Failed to lock post')
    return false
  }

  async function unlockPost (postId: string): Promise<boolean> {
    const toast = useToast()
    const { execute: exec, error: mutError } = useGraphQL()
    const result = await exec(UNLOCK_POST_MUTATION, { variables: { postId } })
    if (result) { toast.success('Post unlocked'); return true }
    toast.error(mutError.value?.message ?? 'Failed to unlock post')
    return false
  }

  async function featurePost (postId: string, featured: boolean, featureType?: string): Promise<boolean> {
    const toast = useToast()
    const { execute: exec, error: mutError } = useGraphQL()
    const result = await exec(FEATURE_POST_MUTATION, { variables: { postId, featured, featureType } })
    if (result) { toast.success(featured ? 'Post featured' : 'Post unfeatured'); return true }
    toast.error(mutError.value?.message ?? 'Failed to update feature status')
    return false
  }

  async function resolveReport (reportId: string, reportType: 'post' | 'comment'): Promise<boolean> {
    const toast = useToast()
    const { execute: exec, error: mutError } = useGraphQL()
    const result = await exec(RESOLVE_REPORT_MUTATION, { variables: { reportId, reportType } })
    if (result) { toast.success('Report resolved'); return true }
    toast.error(mutError.value?.message ?? 'Failed to resolve report')
    return false
  }

  async function dismissReport (reportId: string, reportType: 'post' | 'comment'): Promise<boolean> {
    const toast = useToast()
    const { execute: exec, error: mutError } = useGraphQL()
    const result = await exec(DISMISS_REPORT_MUTATION, { variables: { reportId, reportType } })
    if (result) { toast.success('Report dismissed'); return true }
    toast.error(mutError.value?.message ?? 'Failed to dismiss report')
    return false
  }

  async function distinguishPost (postId: string): Promise<boolean> {
    const toast = useToast()
    const { execute: exec, error: mutError } = useGraphQL()
    const result = await exec(DISTINGUISH_POST_MUTATION, { variables: { postId } })
    if (result) { toast.success('Post distinguish toggled'); return true }
    toast.error(mutError.value?.message ?? 'Failed to toggle distinguish')
    return false
  }

  async function markNsfwPost (postId: string): Promise<boolean> {
    const toast = useToast()
    const { execute: exec, error: mutError } = useGraphQL()
    const result = await exec(MARK_NSFW_POST_MUTATION, { variables: { postId } })
    if (result) { toast.success('Post marked as NSFW'); return true }
    toast.error(mutError.value?.message ?? 'Failed to mark post as NSFW')
    return false
  }

  async function unmarkNsfwPost (postId: string): Promise<boolean> {
    const toast = useToast()
    const { execute: exec, error: mutError } = useGraphQL()
    const result = await exec(UNMARK_NSFW_POST_MUTATION, { variables: { postId } })
    if (result) { toast.success('NSFW mark removed'); return true }
    toast.error(mutError.value?.message ?? 'Failed to remove NSFW mark')
    return false
  }

  async function distinguishComment (commentId: string): Promise<boolean> {
    const toast = useToast()
    const { execute: exec, error: mutError } = useGraphQL()
    const result = await exec(DISTINGUISH_COMMENT_MUTATION, { variables: { commentId } })
    if (result) { toast.success('Comment distinguish toggled'); return true }
    toast.error(mutError.value?.message ?? 'Failed to toggle distinguish')
    return false
  }

  return {
    postReports,
    commentReports,
    loading,
    error,
    fetchPostReports,
    fetchCommentReports,
    removePost,
    restorePost,
    removeComment,
    restoreComment,
    lockPost,
    unlockPost,
    featurePost,
    resolveReport,
    dismissReport,
    distinguishPost,
    distinguishComment,
    markNsfwPost,
    unmarkNsfwPost,
  }
}
