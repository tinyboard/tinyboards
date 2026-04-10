<script setup lang="ts">
definePageMeta({ layout: 'auth' })
useHead({ title: 'Verify Email' })

const route = useRoute()
const token = route.query.token as string | undefined

const loading = ref(false)
const success = ref(false)
const error = ref<string | null>(null)

async function verifyToken (): Promise<void> {
  if (!token) return

  loading.value = true
  error.value = null

  try {
    await $fetch('/api/auth/email-verify', {
      method: 'POST',
      body: { token },
    })
    success.value = true
  } catch (err: unknown) {
    const fetchError = err as { data?: { error?: string }; statusMessage?: string }
    error.value = fetchError.data?.error ?? fetchError.statusMessage ?? 'Verification failed'
  } finally {
    loading.value = false
  }
}

// Auto-verify on page load if token is present
onMounted(() => {
  if (token) {
    verifyToken()
  }
})
</script>

<template>
  <div class="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
    <h1 class="text-xl font-bold text-gray-900 mb-2 text-center">
      Email Verification
    </h1>

    <template v-if="success">
      <div class="text-center py-4">
        <svg class="w-12 h-12 text-green-500 mx-auto mb-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
        </svg>
        <p class="text-sm text-gray-600 mb-4">
          Your email has been verified successfully.
        </p>
        <NuxtLink to="/home" class="button primary no-underline">
          Go to home
        </NuxtLink>
      </div>
    </template>

    <template v-else-if="!token">
      <div class="text-center py-4">
        <p class="text-sm text-red-600 mb-4">
          Invalid or missing verification token. Please request a new verification email from your account settings.
        </p>
        <NuxtLink to="/settings/account" class="text-sm text-primary font-medium hover:underline">
          Account settings
        </NuxtLink>
      </div>
    </template>

    <template v-else-if="loading">
      <div class="text-center py-8">
        <CommonLoadingSpinner size="md" />
        <p class="text-sm text-gray-500 mt-3">Verifying your email...</p>
      </div>
    </template>

    <template v-else-if="error">
      <div class="text-center py-4">
        <svg class="w-12 h-12 text-red-400 mx-auto mb-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
        </svg>
        <p class="text-sm text-red-600 mb-4">{{ error }}</p>
        <div class="flex flex-col items-center gap-2">
          <button class="text-sm text-primary font-medium hover:underline" @click="verifyToken">
            Try again
          </button>
          <NuxtLink to="/settings/account" class="text-sm text-gray-500 hover:underline">
            Account settings
          </NuxtLink>
        </div>
      </div>
    </template>
  </div>
</template>
