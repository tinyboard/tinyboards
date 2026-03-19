<script setup lang="ts">
definePageMeta({ layout: 'settings', middleware: 'guards' })
useHead({ title: 'Security Settings' })

const oldPassword = ref('')
const newPassword = ref('')
const confirmPassword = ref('')
const saving = ref(false)
const success = ref(false)
const errorMsg = ref('')

async function changePassword (): Promise<void> {
  errorMsg.value = ''
  success.value = false

  if (newPassword.value !== confirmPassword.value) {
    errorMsg.value = 'Passwords do not match.'
    return
  }

  if (newPassword.value.length < 8) {
    errorMsg.value = 'Password must be at least 8 characters.'
    return
  }

  saving.value = true

  try {
    await $fetch('/api/auth/change-password', {
      method: 'POST',
      body: {
        old_password: oldPassword.value,
        new_password: newPassword.value,
      },
    })

    success.value = true
    oldPassword.value = ''
    newPassword.value = ''
    confirmPassword.value = ''
    setTimeout(() => { success.value = false }, 3000)
  } catch (err: unknown) {
    const fetchError = err as { data?: { error?: string }; statusMessage?: string }
    errorMsg.value = fetchError.data?.error ?? fetchError.statusMessage ?? 'Failed to change password'
  }

  saving.value = false
}
</script>

<template>
  <div>
    <h2 class="text-lg font-semibold text-gray-900 mb-4">
      Security
    </h2>

    <form @submit.prevent="changePassword" class="space-y-4 max-w-md">
      <div>
        <label class="block text-sm font-medium text-gray-700 mb-1">Current Password</label>
        <input v-model="oldPassword" type="password" class="form-input" required autocomplete="current-password" />
      </div>

      <div>
        <label class="block text-sm font-medium text-gray-700 mb-1">New Password</label>
        <input v-model="newPassword" type="password" class="form-input" required autocomplete="new-password" minlength="8" />
      </div>

      <div>
        <label class="block text-sm font-medium text-gray-700 mb-1">Confirm New Password</label>
        <input v-model="confirmPassword" type="password" class="form-input" required autocomplete="new-password" />
      </div>

      <p v-if="errorMsg" class="text-sm text-red-600">{{ errorMsg }}</p>

      <div class="flex items-center gap-3">
        <button type="submit" class="button primary" :disabled="saving">
          {{ saving ? 'Saving...' : 'Change Password' }}
        </button>
        <span v-if="success" class="text-sm text-green-600">Password changed.</span>
      </div>
    </form>
  </div>
</template>
