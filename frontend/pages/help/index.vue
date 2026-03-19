<script setup lang="ts">
useHead({ title: 'Help' })
</script>

<template>
  <div class="max-w-2xl mx-auto px-4 py-8">
    <h1 class="text-xl font-bold text-gray-900 mb-4">
      Help Center
    </h1>
    <ul class="space-y-2">
      <li>
        <NuxtLink to="/help/about" class="text-primary">
          About
        </NuxtLink>
      </li>
      <li>
        <NuxtLink to="/help/rules" class="text-primary">
          Rules
        </NuxtLink>
      </li>
      <li>
        <NuxtLink to="/help/terms" class="text-primary">
          Terms of Service
        </NuxtLink>
      </li>
      <li>
        <NuxtLink to="/help/privacy" class="text-primary">
          Privacy Policy
        </NuxtLink>
      </li>
    </ul>
  </div>
</template>
