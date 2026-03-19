<script setup lang="ts">
import { computed } from 'vue'
import type { Post } from '~/types/generated'
import { timeAgo } from '~/utils/date'
import { postUrl } from '~/utils/slug'

const props = defineProps<{
  post: Post
  compact?: boolean
}>()

const isThread = computed(() => props.post.isThread)

const linkHostname = computed(() => {
  if (!props.post.url) { return '' }
  try {
    return new globalThis.URL(props.post.url).hostname
  } catch {
    return props.post.url
  }
})

const isYouTubeEmbed = computed(() => {
  const url = props.post.embedVideoUrl
  if (!url) return false
  try {
    const hostname = new globalThis.URL(url).hostname
    return hostname.includes('youtube.com') || hostname.includes('youtube-nocookie.com')
  } catch {
    return false
  }
})

const isDirectVideo = computed(() => {
  const url = props.post.embedVideoUrl
  if (!url || isYouTubeEmbed.value) return false
  return /\.(mp4|webm|ogg)$/i.test(url)
})

const hasLinkPreview = computed(() => {
  return !!(props.post.embedTitle || props.post.embedDescription) && !props.post.embedVideoUrl
})
</script>

<template>
  <article class="bg-white border border-gray-200 rounded-lg hover:border-gray-300 hover:shadow-xs transition-all duration-150 group">
    <!-- Compact mode: tight single row, no vote column -->
    <template v-if="compact">
      <div class="flex items-center gap-2 px-3 py-2">
        <!-- Score (not for threads) -->
        <span v-if="!isThread" class="text-xs font-semibold text-gray-500 tabular-nums w-6 text-center shrink-0">
          {{ post.score }}
        </span>

        <!-- Pinned -->
        <svg v-if="post.isFeaturedBoard" class="w-3.5 h-3.5 text-green-500 shrink-0" fill="currentColor" viewBox="0 0 20 20">
          <path d="M5.5 16.5a.75.75 0 01-.75-.75v-5.5a.75.75 0 011.5 0v5.5a.75.75 0 01-.75.75zM10 16.5a.75.75 0 01-.75-.75V9.75a.75.75 0 011.5 0v6a.75.75 0 01-.75.75zM14.5 16.5a.75.75 0 01-.75-.75v-3.5a.75.75 0 011.5 0v3.5a.75.75 0 01-.75.75z" />
        </svg>

        <!-- Title -->
        <h3 class="text-sm leading-snug flex-1 min-w-0 truncate">
          <a
            v-if="post.url"
            :href="post.url"
            class="text-gray-900 no-underline group-hover:text-primary transition-colors font-normal"
            target="_blank"
            rel="noopener noreferrer"
          >
            {{ post.title }}
          </a>
          <NuxtLink
            v-else
            :to="postUrl(post)"
            class="text-gray-900 no-underline group-hover:text-primary transition-colors font-normal"
          >
            {{ post.title }}
          </NuxtLink>
          <span v-if="post.url" class="text-xs text-gray-400 ml-1">({{ linkHostname }})</span>
        </h3>

        <!-- Badges -->
        <span v-if="post.isNSFW" class="badge badge-red shrink-0">NSFW</span>
        <span
          v-if="isThread"
          class="shrink-0 inline-flex items-center px-1.5 py-0.5 rounded text-[10px] font-medium bg-primary/10 text-primary"
        >
          Thread
        </span>

        <!-- Compact thumbnail -->
        <img
          v-if="post.thumbnailUrl"
          :src="post.thumbnailUrl"
          :alt="post.title"
          class="w-10 h-10 rounded object-cover shrink-0"
          loading="lazy"
        />

        <!-- Meta -->
        <NuxtLink
          :to="postUrl(post)"
          class="text-xs text-gray-400 no-underline hover:text-gray-600 shrink-0 hidden sm:inline"
        >
          {{ post.commentCount }} {{ post.commentCount === 1 ? 'comment' : 'comments' }}
        </NuxtLink>
        <span class="text-gray-300 hidden sm:inline">&middot;</span>
        <NuxtLink
          v-if="post.creator"
          :to="`/@${post.creator.name}`"
          class="text-xs text-gray-400 no-underline hover:text-primary shrink-0 hidden sm:inline"
        >
          {{ post.creator.name }}
        </NuxtLink>
        <time :datetime="post.createdAt" class="text-xs text-gray-400 shrink-0">{{ timeAgo(post.createdAt) }}</time>

        <!-- Board name (in combined feeds) -->
        <NuxtLink
          v-if="post.board"
          :to="`/b/${post.board.name}`"
          class="text-xs font-medium text-gray-500 no-underline hover:text-primary shrink-0 hidden sm:inline"
        >
          b/{{ post.board.name }}
        </NuxtLink>
      </div>
    </template>

    <!-- Expanded mode -->
    <template v-else>
      <div class="flex">
        <!-- Vote column (only for non-thread posts) -->
        <PostActions v-if="!isThread" :post="post" layout="vertical" class="px-2 py-3 border-r border-gray-100 bg-gray-50/50 rounded-l-lg" />

        <!-- Content -->
        <div class="flex-1 min-w-0 p-3">
          <!-- Author line with avatar -->
          <div class="flex items-center gap-2 mb-1.5">
            <CommonAvatar
              v-if="post.creator"
              :src="post.creator.avatar ?? undefined"
              :name="post.creator.name"
              size="xs"
            />
            <NuxtLink
              v-if="post.creator"
              :to="`/@${post.creator.name}`"
              class="text-xs font-medium text-primary no-underline hover:underline"
            >
              {{ post.creator.displayName ?? post.creator.name }}
            </NuxtLink>
            <span v-if="post.creator?.isAdmin" class="inline-flex items-center px-1 py-0 rounded text-[10px] font-medium bg-green-100 text-green-700 border border-green-200 leading-4">
              Admin
            </span>
            <span class="text-gray-300">&middot;</span>
            <time :datetime="post.createdAt" class="text-xs text-gray-400">{{ timeAgo(post.createdAt) }}</time>

            <!-- Thread badge (right-aligned) -->
            <span
              v-if="isThread"
              class="ml-auto inline-flex items-center gap-1 px-1.5 py-0.5 rounded text-[10px] font-medium bg-primary/10 text-primary"
            >
              <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 8h10M7 12h4m1 8l-4-4H5a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v8a2 2 0 01-2 2h-3l-4 4z" />
              </svg>
              Thread
            </span>
          </div>

          <!-- Pinned indicator -->
          <div v-if="post.isFeaturedBoard" class="flex items-center gap-1 text-green-600 text-xs font-medium mb-1">
            <svg class="w-3.5 h-3.5" fill="currentColor" viewBox="0 0 20 20">
              <path d="M5.5 16.5a.75.75 0 01-.75-.75v-5.5a.75.75 0 011.5 0v5.5a.75.75 0 01-.75.75zM10 16.5a.75.75 0 01-.75-.75V9.75a.75.75 0 011.5 0v6a.75.75 0 01-.75.75zM14.5 16.5a.75.75 0 01-.75-.75v-3.5a.75.75 0 011.5 0v3.5a.75.75 0 01-.75.75z" />
            </svg>
            Pinned
          </div>

          <!-- Title -->
          <h3 class="text-base leading-snug mb-0.5">
            <a
              v-if="post.url"
              :href="post.url"
              class="text-gray-900 no-underline group-hover:text-primary transition-colors font-normal"
              target="_blank"
              rel="noopener noreferrer"
            >
              {{ post.title }}
            </a>
            <NuxtLink
              v-else
              :to="postUrl(post)"
              class="text-gray-900 no-underline group-hover:text-primary transition-colors font-normal"
            >
              {{ post.title }}
            </NuxtLink>
            <span v-if="post.url" class="text-xs text-gray-400 ml-1 font-normal">
              ({{ linkHostname }})
            </span>
          </h3>

          <!-- Video embed (YouTube or direct video) -->
          <ClientOnly>
            <div v-if="isYouTubeEmbed" class="mt-2 rounded-lg overflow-hidden aspect-video max-w-lg">
              <iframe
                :src="post.embedVideoUrl!"
                class="w-full h-full"
                frameborder="0"
                allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture"
                allowfullscreen
                loading="lazy"
              />
            </div>
            <div v-else-if="isDirectVideo" class="mt-2 max-w-lg">
              <video
                :src="post.embedVideoUrl!"
                class="w-full rounded-lg"
                controls
                preload="metadata"
              />
            </div>
          </ClientOnly>

          <!-- Thumbnail image (when no video but thumbnail exists) -->
          <img
            v-if="!post.embedVideoUrl && post.thumbnailUrl"
            :src="post.thumbnailUrl"
            :alt="post.title"
            class="mt-2 max-w-sm max-h-64 rounded-lg object-cover"
            loading="lazy"
          />

          <!-- Post image -->
          <img
            v-if="post.image && !post.thumbnailUrl"
            :src="post.image"
            :alt="post.altText || post.title"
            class="mt-2 max-w-sm max-h-64 rounded-lg object-cover"
            loading="lazy"
          />

          <!-- Link preview card (for link posts without video) -->
          <a
            v-if="hasLinkPreview && post.url"
            :href="post.url"
            target="_blank"
            rel="noopener noreferrer"
            class="mt-2 block border border-gray-200 rounded-lg overflow-hidden hover:border-gray-300 transition-colors no-underline max-w-lg"
          >
            <div class="px-3 py-2">
              <p v-if="post.embedTitle" class="text-sm font-medium text-gray-800 line-clamp-1">
                {{ post.embedTitle }}
              </p>
              <p v-if="post.embedDescription" class="text-xs text-gray-500 line-clamp-2 mt-0.5">
                {{ post.embedDescription }}
              </p>
              <p class="text-xs text-gray-400 mt-1">{{ linkHostname }}</p>
            </div>
          </a>

          <!-- Body preview (if text post) -->
          <p
            v-if="post.body && !post.url"
            class="text-sm text-gray-500 line-clamp-2 mt-0.5 leading-relaxed"
          >
            {{ post.body }}
          </p>

          <!-- Flags -->
          <div v-if="post.isNSFW || post.isLocked" class="flex items-center gap-2 mt-1">
            <span v-if="post.isNSFW" class="badge badge-red">NSFW</span>
            <span v-if="post.isLocked" class="inline-flex items-center gap-0.5 text-yellow-500 text-xs">
              <svg class="w-3.5 h-3.5" fill="currentColor" viewBox="0 0 20 20">
                <path fill-rule="evenodd" d="M5 9V7a5 5 0 0110 0v2a2 2 0 012 2v5a2 2 0 01-2 2H5a2 2 0 01-2-2v-5a2 2 0 012-2zm8-2v2H7V7a3 3 0 016 0z" clip-rule="evenodd" />
              </svg>
              Locked
            </span>
          </div>

          <!-- Action bar -->
          <div class="mt-2 flex items-center gap-3 text-xs text-gray-500">
            <NuxtLink
              :to="postUrl(post)"
              class="inline-flex items-center gap-1 no-underline text-gray-500 hover:text-gray-700 transition-colors"
            >
              <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z" />
              </svg>
              {{ post.commentCount }} {{ post.commentCount === 1 ? 'Comment' : 'Comments' }}
            </NuxtLink>

            <button class="inline-flex items-center gap-1 text-gray-500 hover:text-gray-700 transition-colors">
              <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 5a2 2 0 012-2h10a2 2 0 012 2v16l-7-3.5L5 21V5z" />
              </svg>
              Save
            </button>

            <!-- Board name (shown in combined feed views) -->
            <NuxtLink
              v-if="post.board"
              :to="`/b/${post.board.name}`"
              class="ml-auto inline-flex items-center gap-1 font-medium text-gray-500 no-underline hover:text-primary transition-colors"
            >
              b/{{ post.board.name }}
            </NuxtLink>
          </div>
        </div>
      </div>
    </template>
  </article>
</template>
