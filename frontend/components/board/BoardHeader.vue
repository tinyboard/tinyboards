<script setup lang="ts">
import type { Board } from '~/types/generated'
import { useAuthStore } from '~/stores/auth'

defineProps<{
  board: Board
  isSubscribed?: boolean
}>()

const emit = defineEmits<{
  subscribe: []
  unsubscribe: []
}>()

const authStore = useAuthStore()
</script>

<template>
  <div class="bg-white border-b border-gray-200">
    <!-- Banner -->
    <div class="h-32 sm:h-40 bg-gradient-to-br from-primary to-primary-hover overflow-hidden">
      <img
        v-if="board.banner"
        :src="board.banner"
        :alt="`${board.name} banner`"
        class="w-full h-full object-cover"
      >
    </div>

    <!-- Board info bar -->
    <div class="max-w-5xl mx-auto px-4">
      <div class="flex items-center gap-4 py-3">
        <!-- Avatar -->
        <CommonAvatar
          :src="board.icon ?? undefined"
          :name="board.name"
          size="lg"
          class="border-2 border-white shadow shrink-0"
        />

        <!-- Name + description -->
        <div class="flex-1 min-w-0">
          <h1 class="text-lg font-bold text-gray-900 truncate">{{ board.title }}</h1>
          <p class="text-sm text-gray-500">b/{{ board.name }}</p>
        </div>

        <!-- Join button -->
        <div class="shrink-0">
          <button
            v-if="authStore.isLoggedIn && isSubscribed"
            class="button button-sm white"
            @click="emit('unsubscribe')"
          >
            Joined
          </button>
          <button
            v-else-if="authStore.isLoggedIn"
            class="button button-sm primary"
            @click="emit('subscribe')"
          >
            Join
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
