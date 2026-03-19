<template>
  <ClientOnly>
    <div class="border border-gray-200 rounded">
      <EditorToolbar />
      <textarea class="w-full min-h-[300px] p-3 text-sm border-0 focus:ring-0 resize-y" placeholder="Write wiki content..." />
    </div>
    <template #fallback>
      <div class="border border-gray-200 rounded p-3 min-h-[300px] bg-gray-50">
        <CommonLoadingSpinner size="sm" />
      </div>
    </template>
  </ClientOnly>
</template>
