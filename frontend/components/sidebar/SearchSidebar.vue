<script setup lang="ts">
import { useSiteStore } from '~/stores/site'

const siteStore = useSiteStore()
</script>

<template>
  <div class="space-y-5">
    <!-- Search tips -->
    <div class="bg-white rounded-lg border border-gray-200 overflow-hidden">
      <div class="h-16 bg-gradient-to-br from-amber-400 to-orange-500" />
      <div class="px-4 py-3 -mt-4">
        <div class="w-10 h-10 rounded-lg bg-white shadow-sm border border-gray-100 flex items-center justify-center mb-2">
          <svg class="w-5 h-5 text-amber-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
          </svg>
        </div>
        <h3 class="font-semibold text-sm text-gray-900">Search {{ siteStore.name }}</h3>
        <p class="text-xs text-gray-500 mt-1 leading-relaxed">
          Find posts, comments, users, and boards across the platform.
        </p>
      </div>
    </div>

    <!-- Search tips card -->
    <div class="bg-white rounded-lg border border-gray-200 p-4">
      <h4 class="text-xs font-semibold text-gray-400 uppercase tracking-wider mb-3">
        Search Tips
      </h4>
      <ul class="space-y-2.5 text-xs text-gray-600">
        <li class="flex items-start gap-2">
          <span class="inline-block w-1 h-1 rounded-full bg-gray-300 mt-1.5 shrink-0" />
          Use the tabs to filter by posts, comments, users, or boards
        </li>
        <li class="flex items-start gap-2">
          <span class="inline-block w-1 h-1 rounded-full bg-gray-300 mt-1.5 shrink-0" />
          Search is case-insensitive
        </li>
        <li class="flex items-start gap-2">
          <span class="inline-block w-1 h-1 rounded-full bg-gray-300 mt-1.5 shrink-0" />
          Minimum 2 characters required
        </li>
      </ul>
    </div>
  </div>
</template>
