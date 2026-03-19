<script setup lang="ts">
defineProps<{
  modelValue: string
}>()

const emit = defineEmits<{
  'update:modelValue': [value: string]
}>()
</script>

<template>
  <div>
    <label for="invite-code" class="block text-sm font-medium text-gray-700 mb-1">
      Invite code
    </label>
    <input
      id="invite-code"
      :value="modelValue"
      type="text"
      class="form-input"
      required
      placeholder="Enter your invite code"
      @input="emit('update:modelValue', ($event.target as HTMLInputElement).value)"
    >
    <p class="mt-1 text-xs text-gray-500">
      Registration requires an invite code. Contact an existing member to get one.
    </p>
  </div>
</template>
