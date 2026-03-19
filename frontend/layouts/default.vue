<template>
  <div class="min-h-screen flex flex-col bg-gray-100">
    <LayoutAppHeader />

    <div class="flex-1 flex gap-4 max-w-8xl mx-auto w-full">
      <main class="flex-1 min-w-0">
        <slot />
      </main>
      <LayoutAppSidebar />
    </div>

    <LayoutAppFooter />
  </div>
</template>
