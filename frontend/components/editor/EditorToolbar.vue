<script setup lang="ts">
const emit = defineEmits<{
  action: [type: string]
}>()
</script>

<template>
  <div class="bg-gray-50 border-b border-gray-200 px-2 py-1 flex items-center gap-1">
    <button type="button" class="p-1 rounded hover:bg-gray-200 text-gray-600" title="Bold" @click="emit('action', 'bold')">
      <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="3" d="M6 4h8a4 4 0 014 4 4 4 0 01-4 4H6z" />
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="3" d="M6 12h9a4 4 0 014 4 4 4 0 01-4 4H6z" />
      </svg>
    </button>
    <button type="button" class="p-1 rounded hover:bg-gray-200 text-gray-600 italic" title="Italic" @click="emit('action', 'italic')">
      <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 4h4m-2 0l-4 16m0 0h4" />
      </svg>
    </button>
    <button type="button" class="p-1 rounded hover:bg-gray-200 text-gray-600" title="Strikethrough" @click="emit('action', 'strikethrough')">
      <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M16 4H9.5a3.5 3.5 0 000 7h5a3.5 3.5 0 010 7H6M4 12h16" />
      </svg>
    </button>
    <button type="button" class="p-1 rounded hover:bg-gray-200 text-gray-600" title="Link" @click="emit('action', 'link')">
      <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1" />
      </svg>
    </button>
    <div class="w-px h-4 bg-gray-300 mx-1" />
    <button type="button" class="p-1 rounded hover:bg-gray-200 text-gray-600" title="Heading" @click="emit('action', 'heading')">
      <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h8" />
      </svg>
    </button>
    <button type="button" class="p-1 rounded hover:bg-gray-200 text-gray-600" title="Quote" @click="emit('action', 'quote')">
      <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 10h.01M12 10h.01M16 10h.01M9 16H5a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v8a2 2 0 01-2 2h-5l-5 5v-5z" />
      </svg>
    </button>
    <button type="button" class="p-1 rounded hover:bg-gray-200 text-gray-600" title="Code" @click="emit('action', 'code')">
      <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 20l4-16m4 4l4 4-4 4M6 16l-4-4 4-4" />
      </svg>
    </button>
    <button type="button" class="p-1 rounded hover:bg-gray-200 text-gray-600" title="Bullet list" @click="emit('action', 'list')">
      <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 6h13M8 12h13M8 18h13M3 6h.01M3 12h.01M3 18h.01" />
      </svg>
    </button>
  </div>
</template>
