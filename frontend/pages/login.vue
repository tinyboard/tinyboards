<script setup lang="ts">
definePageMeta({
  layout: 'auth',
})

useHead({
  title: 'Log in',
})
</script>

<template>
  <div class="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
    <h1 class="text-xl font-bold text-gray-900 mb-6 text-center">
      Log in
    </h1>

    <UserLoginForm />

    <p class="mt-4 text-sm text-center text-gray-500">
      Don't have an account?
      <NuxtLink to="/register" class="text-primary font-medium">
        Sign up
      </NuxtLink>
    </p>
  </div>
</template>
