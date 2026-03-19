<template>
  <div class="min-h-screen flex flex-col bg-gray-100">
    <LayoutAppHeader />

    <main class="flex-1 flex items-center justify-center px-4 py-12">
      <div class="w-full max-w-md">
        <slot />
      </div>
    </main>

    <LayoutAppFooter />
  </div>
</template>
