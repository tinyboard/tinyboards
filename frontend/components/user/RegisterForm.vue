<script setup lang="ts">
import { ref } from 'vue'
import { useAuth } from '~/composables/useAuth'
import { useSiteStore } from '~/stores/site'
import type { RegisterInput } from '~/types/api'

const { register, loading, error } = useAuth()
const siteStore = useSiteStore()

const form = ref({
  username: '',
  email: '',
  password: '',
  passwordVerify: '',
  inviteCode: '',
  applicationText: '',
})

const localError = ref<string | null>(null)

async function handleSubmit (): Promise<void> {
  localError.value = null

  if (form.value.password !== form.value.passwordVerify) {
    localError.value = 'Passwords do not match.'
    return
  }

  const input: RegisterInput = {
    username: form.value.username,
    email: form.value.email || undefined,
    password: form.value.password,
    inviteCode: form.value.inviteCode || undefined,
    applicationAnswer: form.value.applicationText || undefined,
  }

  const success = await register(input)
  if (success) {
    if (siteStore.registrationMode === 'email_verification') {
      await navigateTo('/register?step=verify')
    } else {
      await navigateTo('/home')
    }
  }
}
</script>

<template>
  <form class="space-y-4" @submit.prevent="handleSubmit">
    <div>
      <label for="reg-username" class="block text-sm font-medium text-gray-700 mb-1">
        Username
      </label>
      <input
        id="reg-username"
        v-model="form.username"
        type="text"
        class="form-input"
        required
        autocomplete="username"
      >
    </div>

    <div>
      <label for="reg-email" class="block text-sm font-medium text-gray-700 mb-1">
        Email
        <span v-if="siteStore.registrationMode !== 'email_verification'" class="text-gray-400 font-normal">(optional)</span>
      </label>
      <input
        id="reg-email"
        v-model="form.email"
        type="email"
        class="form-input"
        :required="siteStore.requireEmailVerification"
        autocomplete="email"
      >
    </div>

    <div>
      <label for="reg-password" class="block text-sm font-medium text-gray-700 mb-1">
        Password
      </label>
      <input
        id="reg-password"
        v-model="form.password"
        type="password"
        class="form-input"
        required
        autocomplete="new-password"
      >
    </div>

    <div>
      <label for="reg-password-verify" class="block text-sm font-medium text-gray-700 mb-1">
        Confirm password
      </label>
      <input
        id="reg-password-verify"
        v-model="form.passwordVerify"
        type="password"
        class="form-input"
        required
        autocomplete="new-password"
      >
    </div>

    <!-- Invite code field (invite_only mode) -->
    <UserInviteCodeField
      v-if="siteStore.registrationMode === 'invite_only'"
      v-model="form.inviteCode"
    />

    <!-- Application field (application mode) -->
    <UserApplicationField
      v-if="siteStore.registrationMode === 'application_required' || siteStore.registrationMode === 'application'"
      v-model="form.applicationText"
      :question="siteStore.site?.applicationQuestion ?? undefined"
    />

    <div v-if="localError || error" class="text-sm text-red-600 bg-red-50 border border-red-200 rounded px-3 py-2">
      {{ localError ?? error?.message }}
    </div>

    <button
      type="submit"
      class="button primary w-full"
      :disabled="loading"
    >
      <CommonLoadingSpinner v-if="loading" size="sm" />
      <span v-else>Create account</span>
    </button>
  </form>
</template>
