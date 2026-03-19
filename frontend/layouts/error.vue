<template>
  <div class="min-h-screen flex flex-col bg-gray-100">
    <LayoutAppHeader />

    <main class="flex-1 flex items-center justify-center">
      <slot />
    </main>

    <LayoutAppFooter />
  </div>
</template>
