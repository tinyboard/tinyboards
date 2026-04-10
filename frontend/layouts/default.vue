<template>
  <div class="min-h-screen flex flex-col bg-gray-100">
    <LayoutAppHeader />

    <div class="flex-1 flex gap-6 lg:gap-8 max-w-8xl mx-auto w-full px-3 sm:px-4 lg:px-8 2xl:px-12">
      <main class="flex-1 min-w-0 pb-20 lg:pb-0">
        <slot />
      </main>
      <LayoutAppSidebar />
    </div>

    <LayoutAppFooter class="hidden lg:block" />
    <LayoutMobileBottomNav class="lg:hidden" />
  </div>
</template>
