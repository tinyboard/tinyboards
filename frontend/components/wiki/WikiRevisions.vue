<template>
  <div class="space-y-2">
    <slot />
    <div v-if="!$slots.default" class="text-sm text-gray-500">
      No revisions.
    </div>
  </div>
</template>
