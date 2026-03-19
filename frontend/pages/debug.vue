<script setup lang="ts">
import { useSiteStore } from '~/stores/site'
import { useAuthStore } from '~/stores/auth'
import { useUIStore } from '~/stores/ui'

useHead({ title: 'Debug' })

const siteStore = useSiteStore()
const authStore = useAuthStore()
const uiStore = useUIStore()
</script>

<template>
  <div class="max-w-4xl mx-auto px-4 py-4">
    <h1 class="text-lg font-semibold text-gray-900 mb-4">
      Debug
    </h1>
    <div class="space-y-4 text-sm">
      <div class="bg-white border rounded p-4">
        <h2 class="font-semibold mb-2">
          Site Store
        </h2>
        <pre class="text-xs bg-gray-50 p-2 rounded overflow-auto">{{ JSON.stringify(siteStore.$state, null, 2) }}</pre>
      </div>
      <div class="bg-white border rounded p-4">
        <h2 class="font-semibold mb-2">
          Auth Store
        </h2>
        <pre class="text-xs bg-gray-50 p-2 rounded overflow-auto">{{ JSON.stringify({ isLoggedIn: authStore.isLoggedIn, isAdmin: authStore.isAdmin, user: authStore.user?.name }, null, 2) }}</pre>
      </div>
      <div class="bg-white border rounded p-4">
        <h2 class="font-semibold mb-2">
          UI Store
        </h2>
        <pre class="text-xs bg-gray-50 p-2 rounded overflow-auto">{{ JSON.stringify(uiStore.$state, null, 2) }}</pre>
      </div>
    </div>
  </div>
</template>
