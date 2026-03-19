<script setup lang="ts">
defineProps<{
  currentAvatar?: string
}>()

const emit = defineEmits<{
  upload: [file: File]
}>()

function handleFileChange (event: Event): void {
  const input = event.target as HTMLInputElement
  if (input.files?.[0]) {
    emit('upload', input.files[0])
  }
}
</script>

<template>
  <div class="flex items-center gap-4">
    <CommonAvatar :src="currentAvatar" name="User" size="lg" />
    <label class="button button-sm gray cursor-pointer">
      Change Avatar
      <input type="file" class="hidden" accept="image/*" @change="handleFileChange">
    </label>
  </div>
</template>
