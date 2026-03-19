<template>
  <div class="bg-white border border-gray-200 rounded">
    <div class="p-4 border-b border-gray-200">
      <h3 class="font-semibold text-gray-900 text-sm">
        Moderation Log
      </h3>
    </div>
    <slot />
    <div v-if="!$slots.default" class="p-8 text-center text-sm text-gray-500">
      No moderation actions yet.
    </div>
  </div>
</template>
