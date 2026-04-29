import { ref } from 'vue'
import { useGraphQL } from '~/composables/useGraphQL'
import { useToast } from '~/composables/useToast'

export interface ReactionCount {
  emoji: string
  count: number
  reacted: boolean
}

export interface ReactionAggregateInput {
  emoji: string
  count: number
}

const ADD_REACTION = `
  mutation AddReaction($input: AddReactionInput!) {
    addReaction(input: $input) {
      reaction { id emoji score }
    }
  }
`

const REMOVE_REACTION = `
  mutation RemoveReaction($input: RemoveReactionInput!) {
    removeReaction(input: $input) { success }
  }
`

export function useReactions (targetType: 'post' | 'comment', targetId: string) {
  const toast = useToast()
  const reactions = ref<ReactionCount[]>([])
  const acting = ref(false)

  function setInitial (
    aggregates: ReactionAggregateInput[] | null | undefined,
    myEmoji: string | null | undefined,
  ): void {
    if (!aggregates || aggregates.length === 0) {
      reactions.value = []
      return
    }
    reactions.value = aggregates
      .filter(a => a.count > 0)
      .map(a => ({
        emoji: a.emoji,
        count: a.count,
        reacted: !!myEmoji && a.emoji === myEmoji,
      }))
  }

  async function addReaction (emoji: string): Promise<boolean> {
    acting.value = true
    const { execute } = useGraphQL()
    const input: Record<string, string> = { emoji }
    if (targetType === 'post') { input.postId = targetId }
    else { input.commentId = targetId }

    const result = await execute(ADD_REACTION, { variables: { input } })
    acting.value = false

    if (result) {
      // Update local state
      const existing = reactions.value.find(r => r.emoji === emoji)
      if (existing) {
        existing.count++
        existing.reacted = true
      } else {
        reactions.value.push({ emoji, count: 1, reacted: true })
      }
      return true
    }
    toast.error('Failed to add reaction')
    return false
  }

  async function removeReaction (emoji: string): Promise<boolean> {
    acting.value = true
    const { execute } = useGraphQL()
    const input: Record<string, string> = { emoji }
    if (targetType === 'post') { input.postId = targetId }
    else { input.commentId = targetId }

    const result = await execute(REMOVE_REACTION, { variables: { input } })
    acting.value = false

    if (result) {
      const existing = reactions.value.find(r => r.emoji === emoji)
      if (existing) {
        existing.count--
        existing.reacted = false
        if (existing.count <= 0) {
          reactions.value = reactions.value.filter(r => r.emoji !== emoji)
        }
      }
      return true
    }
    toast.error('Failed to remove reaction')
    return false
  }

  async function toggleReaction (emoji: string): Promise<void> {
    const existing = reactions.value.find(r => r.emoji === emoji)
    if (existing?.reacted) {
      await removeReaction(emoji)
    } else {
      await addReaction(emoji)
    }
  }

  return {
    reactions,
    acting,
    addReaction,
    removeReaction,
    toggleReaction,
    setInitial,
  }
}
