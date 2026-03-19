<script setup lang="ts">
import { ref } from 'vue'
import { useGraphQL } from '~/composables/useGraphQL'
import { useToast } from '~/composables/useToast'
import type { User } from '~/types/generated'

const props = defineProps<{
  user: User
}>()

const toast = useToast()

const FOLLOW_MUTATION = `mutation FollowUser($userId: ID!) { followUser(userId: $userId) }`
const UNFOLLOW_MUTATION = `mutation UnfollowUser($userId: ID!) { unfollowUser(userId: $userId) }`
const BLOCK_MUTATION = `mutation BlockUser($userId: ID!) { blockUser(userId: $userId) }`
const UNBLOCK_MUTATION = `mutation UnblockUser($userId: ID!) { unblockUser(userId: $userId) }`

const isFollowing = ref(false)
const isBlocked = ref(false)
const acting = ref(false)

async function toggleFollow (): Promise<void> {
  acting.value = true
  const { execute } = useGraphQL()
  const mutation = isFollowing.value ? UNFOLLOW_MUTATION : FOLLOW_MUTATION
  const result = await execute(mutation, { variables: { userId: props.user.id } })
  if (result) {
    isFollowing.value = !isFollowing.value
    toast.success(isFollowing.value ? 'Followed' : 'Unfollowed')
  }
  acting.value = false
}

async function toggleBlock (): Promise<void> {
  acting.value = true
  const { execute } = useGraphQL()
  const mutation = isBlocked.value ? UNBLOCK_MUTATION : BLOCK_MUTATION
  const result = await execute(mutation, { variables: { userId: props.user.id } })
  if (result) {
    isBlocked.value = !isBlocked.value
    toast.success(isBlocked.value ? 'User blocked' : 'User unblocked')
  }
  acting.value = false
}

function openMessage (): void {
  navigateTo(`/inbox/messages/${props.user.id}`)
}
</script>

<template>
  <div class="flex gap-2">
    <button
      class="button button-sm"
      :class="isFollowing ? 'primary' : 'gray'"
      :disabled="acting"
      @click="toggleFollow"
    >
      {{ isFollowing ? 'Following' : 'Follow' }}
    </button>
    <button
      class="button button-sm gray"
      :disabled="acting"
      @click="openMessage"
    >
      Message
    </button>
    <button
      class="button button-sm"
      :class="isBlocked ? 'bg-red-100 text-red-700 hover:bg-red-200' : 'gray'"
      :disabled="acting"
      @click="toggleBlock"
    >
      {{ isBlocked ? 'Unblock' : 'Block' }}
    </button>
  </div>
</template>
