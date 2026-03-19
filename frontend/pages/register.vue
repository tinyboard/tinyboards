<script setup lang="ts">
import { useSiteStore } from '~/stores/site'

definePageMeta({
  layout: 'auth',
})

useHead({
  title: 'Sign up',
})

const siteStore = useSiteStore()
const route = useRoute()

const isVerifyStep = computed(() => route.query.step === 'verify')
</script>

<template>
  <div class="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
    <!-- Closed registration -->
    <template v-if="siteStore.registrationMode === 'closed'">
      <h1 class="text-xl font-bold text-gray-900 mb-4 text-center">
        Registration closed
      </h1>
      <p class="text-sm text-gray-500 text-center">
        This site is not accepting new registrations at this time.
      </p>
    </template>

    <!-- Email verification step -->
    <template v-else-if="isVerifyStep">
      <h1 class="text-xl font-bold text-gray-900 mb-4 text-center">
        Check your email
      </h1>
      <p class="text-sm text-gray-500 text-center">
        We sent a verification link to your email address.
        Please click the link to activate your account.
      </p>
    </template>

    <!-- Registration form -->
    <template v-else>
      <h1 class="text-xl font-bold text-gray-900 mb-6 text-center">
        Create an account
      </h1>

      <UserRegisterForm />

      <p class="mt-4 text-sm text-center text-gray-500">
        Already have an account?
        <NuxtLink to="/login" class="text-primary font-medium">
          Log in
        </NuxtLink>
      </p>
    </template>
  </div>
</template>
