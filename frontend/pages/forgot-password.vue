<script setup lang="ts">
definePageMeta({ layout: 'auth' })
useHead({ title: 'Reset Password' })

const email = ref('')
const loading = ref(false)
const submitted = ref(false)
const error = ref<string | null>(null)

async function handleSubmit (): Promise<void> {
  loading.value = true
  error.value = null

  try {
    await $fetch('/api/auth/password-reset-request', {
      method: 'POST',
      body: { email: email.value },
    })
    submitted.value = true
  } catch (err: unknown) {
    const fetchError = err as { data?: { error?: string }; statusMessage?: string }
    error.value = fetchError.data?.error ?? fetchError.statusMessage ?? 'Failed to send reset email'
  } finally {
    loading.value = false
  }
}
</script>

<template>
  <div class="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
    <h1 class="text-xl font-bold text-gray-900 mb-2 text-center">
      Reset your password
    </h1>

    <template v-if="submitted">
      <div class="text-center py-4">
        <svg class="w-12 h-12 text-green-500 mx-auto mb-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 8l7.89 5.26a2 2 0 002.22 0L21 8M5 19h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" />
        </svg>
        <p class="text-sm text-gray-600 mb-4">
          If an account with that email exists, we've sent a password reset link.
          Check your inbox and follow the instructions.
        </p>
        <NuxtLink to="/login" class="text-sm text-primary font-medium hover:underline">
          Back to login
        </NuxtLink>
      </div>
    </template>

    <template v-else>
      <p class="text-sm text-gray-500 text-center mb-6">
        Enter the email address associated with your account and we'll send you a link to reset your password.
      </p>

      <form class="space-y-4" @submit.prevent="handleSubmit">
        <div>
          <label for="reset-email" class="block text-sm font-medium text-gray-700 mb-1">
            Email address
          </label>
          <input
            id="reset-email"
            v-model="email"
            type="email"
            class="form-input"
            required
            autocomplete="email"
            placeholder="you@example.com"
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
          <span v-else>Send reset link</span>
        </button>
      </form>

      <p class="mt-4 text-sm text-center text-gray-500">
        Remember your password?
        <NuxtLink to="/login" class="text-primary font-medium">
          Log in
        </NuxtLink>
      </p>
    </template>
  </div>
</template>
