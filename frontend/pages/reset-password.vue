<script setup lang="ts">
definePageMeta({ layout: 'auth' })
useHead({ title: 'Set New Password' })

const route = useRoute()
const token = route.query.token as string | undefined

const password = ref('')
const confirmPassword = ref('')
const loading = ref(false)
const success = ref(false)
const error = ref<string | null>(null)

async function handleSubmit (): Promise<void> {
  error.value = null

  if (password.value.length < 10) {
    error.value = 'Password must be at least 10 characters'
    return
  }

  if (password.value !== confirmPassword.value) {
    error.value = 'Passwords do not match'
    return
  }

  if (!token) {
    error.value = 'Invalid or missing reset token'
    return
  }

  loading.value = true

  try {
    await $fetch('/api/auth/password-reset-complete', {
      method: 'POST',
      body: { token, new_password: password.value },
    })
    success.value = true
  } catch (err: unknown) {
    const fetchError = err as { data?: { error?: string }; statusMessage?: string }
    error.value = fetchError.data?.error ?? fetchError.statusMessage ?? 'Failed to reset password'
  } finally {
    loading.value = false
  }
}
</script>

<template>
  <div class="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
    <h1 class="text-xl font-bold text-gray-900 mb-2 text-center">
      Set new password
    </h1>

    <template v-if="success">
      <div class="text-center py-4">
        <svg class="w-12 h-12 text-green-500 mx-auto mb-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
        </svg>
        <p class="text-sm text-gray-600 mb-4">
          Your password has been reset successfully.
        </p>
        <NuxtLink to="/login" class="button primary no-underline">
          Log in with new password
        </NuxtLink>
      </div>
    </template>

    <template v-else-if="!token">
      <div class="text-center py-4">
        <p class="text-sm text-red-600 mb-4">
          Invalid or missing reset token. Please request a new password reset link.
        </p>
        <NuxtLink to="/forgot-password" class="text-sm text-primary font-medium hover:underline">
          Request new link
        </NuxtLink>
      </div>
    </template>

    <template v-else>
      <p class="text-sm text-gray-500 text-center mb-6">
        Choose a new password for your account.
      </p>

      <form class="space-y-4" @submit.prevent="handleSubmit">
        <div>
          <label for="new-password" class="block text-sm font-medium text-gray-700 mb-1">
            New password
          </label>
          <input
            id="new-password"
            v-model="password"
            type="password"
            class="form-input"
            required
            minlength="10"
            autocomplete="new-password"
          >
          <p class="text-xs text-gray-400 mt-1">Must be at least 10 characters</p>
        </div>

        <div>
          <label for="confirm-password" class="block text-sm font-medium text-gray-700 mb-1">
            Confirm new password
          </label>
          <input
            id="confirm-password"
            v-model="confirmPassword"
            type="password"
            class="form-input"
            required
            minlength="10"
            autocomplete="new-password"
          >
        </div>

        <div v-if="error" class="text-sm text-red-600 bg-red-50 border border-red-200 rounded px-3 py-2">
          {{ error }}
        </div>

        <button
          type="submit"
          class="button primary w-full"
          :disabled="loading"
        >
          <CommonLoadingSpinner v-if="loading" size="sm" />
          <span v-else>Reset password</span>
        </button>
      </form>
    </template>
  </div>
</template>
