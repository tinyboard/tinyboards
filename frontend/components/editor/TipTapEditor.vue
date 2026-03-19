<script setup lang="ts">
const props = defineProps<{
  modelValue: string
  placeholder?: string
  minHeight?: string
}>()

const emit = defineEmits<{
  'update:modelValue': [value: string]
}>()
</script>

<template>
  <ClientOnly>
    <EditorMarkdownEditor
      :model-value="modelValue"
      :placeholder="placeholder"
      :min-height="minHeight"
      @update:model-value="emit('update:modelValue', $event)"
    />
    <template #fallback>
      <div class="border border-gray-200 rounded p-3 min-h-[200px] bg-gray-50">
        <CommonLoadingSpinner size="sm" />
      </div>
    </template>
  </ClientOnly>
</template>
