<template>
  <div class="bg-white border border-gray-200 rounded divide-y divide-gray-100">
    <slot />
    <div v-if="!$slots.default" class="p-8 text-center text-sm text-gray-500">
      No conversations.
    </div>
  </div>
</template>
