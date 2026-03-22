<template>
  <div class="min-h-screen flex flex-col bg-gray-100">
    <LayoutAppHeader />

    <div class="flex-1 flex gap-6 lg:gap-8 max-w-8xl mx-auto w-full px-4 lg:px-8">
      <main class="flex-1 min-w-0">
        <slot />
      </main>
      <LayoutAppSidebar />
    </div>

    <LayoutAppFooter />
  </div>
</template>
