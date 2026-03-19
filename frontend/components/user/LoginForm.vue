<script setup lang="ts">
import { ref } from 'vue'
import { useAuth } from '~/composables/useAuth'
import type { LoginInput } from '~/types/api'

const { login, loading, error } = useAuth()
const route = useRoute()

const form = ref<LoginInput>({
  usernameOrEmail: '',
  password: '',
})

async function handleSubmit (): Promise<void> {
  const success = await login(form.value)
  if (success) {
    const redirect = route.query.redirect as string | undefined
    await navigateTo(redirect ?? '/home')
  }
}
</script>

<template>
  <form class="space-y-4" @submit.prevent="handleSubmit">
    <div>
      <label for="login-username" class="block text-sm font-medium text-gray-700 mb-1">
        Username or email
      </label>
      <input
        id="login-username"
        v-model="form.usernameOrEmail"
        type="text"
        class="form-input"
        required
        autocomplete="username"
      >
    </div>

    <div>
      <div class="flex items-center justify-between mb-1">
        <label for="login-password" class="block text-sm font-medium text-gray-700">
          Password
        </label>
        <NuxtLink to="/forgot-password" class="text-xs text-primary hover:underline">
          Forgot password?
        </NuxtLink>
      </div>
      <input
        id="login-password"
        v-model="form.password"
        type="password"
        class="form-input"
        required
        autocomplete="current-password"
      >
    </div>

    <div v-if="error" class="text-sm text-red-600 bg-red-50 border border-red-200 rounded px-3 py-2">
      {{ error.message }}
    </div>

    <button
      type="submit"
      class="button primary w-full"
      :disabled="loading"
    >
      <CommonLoadingSpinner v-if="loading" size="sm" />
      <span v-else>Log in</span>
    </button>
  </form>
</template>
