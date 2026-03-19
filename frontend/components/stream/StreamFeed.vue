<template>
  <div>
    <slot />
    <div v-if="!$slots.default" class="p-8 text-center text-sm text-gray-500">
      No posts in this stream yet.
    </div>
  </div>
</template>
