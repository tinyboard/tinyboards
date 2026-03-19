<script setup lang="ts">
import { useReactions } from '~/composables/useReactions'
import { useAuthStore } from '~/stores/auth'

const props = defineProps<{
  targetType: 'post' | 'comment'
  targetId: string
}>()

const authStore = useAuthStore()
const { reactions, acting, toggleReaction, addReaction } = useReactions(props.targetType, props.targetId)

const showPicker = ref(false)

const defaultEmojis = ['👍', '❤️', '😂', '😮', '😢', '🔥']

async function handleToggle (emoji: string): Promise<void> {
  if (!authStore.isLoggedIn) {
    navigateTo('/login')
    return
  }
  await toggleReaction(emoji)
}

async function handlePickerSelect (emoji: string): Promise<void> {
  showPicker.value = false
  if (!authStore.isLoggedIn) {
    navigateTo('/login')
    return
  }
  const existing = reactions.value.find(r => r.emoji === emoji)
  if (existing?.reacted) return
  await addReaction(emoji)
}
</script>

<template>
  <div class="flex items-center gap-1.5 flex-wrap">
    <!-- Existing reactions -->
    <button
      v-for="r in reactions"
      :key="r.emoji"
      class="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-xs border transition-colors"
      :class="r.reacted
        ? 'border-primary/30 bg-primary/5 text-primary'
        : 'border-gray-200 bg-white text-gray-600 hover:border-gray-300'"
      :disabled="acting"
      @click="handleToggle(r.emoji)"
    >
      <span>{{ r.emoji }}</span>
      <span class="font-medium">{{ r.count }}</span>
    </button>

    <!-- Add reaction button -->
    <div v-if="authStore.isLoggedIn" class="relative">
      <button
        class="inline-flex items-center px-1.5 py-0.5 rounded-full text-xs border border-dashed border-gray-300 text-gray-400 hover:text-gray-600 hover:border-gray-400 transition-colors"
        :disabled="acting"
        @click="showPicker = !showPicker"
      >
        +
      </button>

      <!-- Simple emoji picker -->
      <div
        v-if="showPicker"
        class="absolute bottom-full left-0 mb-1 bg-white border border-gray-200 rounded-lg shadow-lg p-2 z-20 flex gap-1 dropdown-enter"
      >
        <button
          v-for="emoji in defaultEmojis"
          :key="emoji"
          class="w-8 h-8 flex items-center justify-center rounded hover:bg-gray-100 text-base"
          @click="handlePickerSelect(emoji)"
        >
          {{ emoji }}
        </button>
      </div>
    </div>
  </div>
</template>
