<template>
  <div class="bg-white border border-gray-200 rounded">
    <slot />
    <div v-if="!$slots.default" class="p-8 text-center text-sm text-gray-500">
      No notifications.
    </div>
  </div>
</template>
