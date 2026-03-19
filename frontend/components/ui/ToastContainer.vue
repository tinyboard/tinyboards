<script setup lang="ts">
import { useUIStore } from '~/stores/ui'

const ui = useUIStore()

const typeConfig: Record<string, { bg: string; icon: string }> = {
  success: { bg: 'bg-green-50 border-green-300 text-green-800', icon: '&#10003;' },
  error: { bg: 'bg-red-50 border-red-300 text-red-800', icon: '&#10007;' },
  warning: { bg: 'bg-yellow-50 border-yellow-300 text-yellow-800', icon: '&#9888;' },
  info: { bg: 'bg-blue-50 border-blue-300 text-blue-800', icon: '&#8505;' },
}
</script>

<template>
  <Teleport to="body">
    <div class="fixed bottom-4 right-4 z-[9999] flex flex-col gap-2 max-w-sm">
      <TransitionGroup
        enter-active-class="transition ease-out duration-200"
        enter-from-class="translate-y-2 opacity-0"
        enter-to-class="translate-y-0 opacity-100"
        leave-active-class="transition ease-in duration-150"
        leave-from-class="translate-y-0 opacity-100"
        leave-to-class="translate-y-2 opacity-0"
        move-class="transition-all duration-200"
      >
        <div
          v-for="toast in ui.toasts"
          :key="toast.id"
          class="flex items-start gap-2 px-4 py-3 rounded-lg border shadow-lg text-sm"
          :class="typeConfig[toast.type]?.bg"
          role="alert"
        >
          <span class="shrink-0 mt-0.5 font-bold" v-html="typeConfig[toast.type]?.icon" />
          <span class="flex-1">{{ toast.message }}</span>
          <button
            class="shrink-0 opacity-60 hover:opacity-100 ml-2"
            aria-label="Dismiss"
            @click="ui.removeToast(toast.id)"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>
      </TransitionGroup>
    </div>
  </Teleport>
</template>
