<script setup lang="ts">
import { useWiki } from '~/composables/useWiki'
import { useGraphQL } from '~/composables/useGraphQL'
import { useToast } from '~/composables/useToast'

definePageMeta({ middleware: 'guards' })

const route = useRoute()
const boardName = route.params.board as string
const toast = useToast()

useHead({ title: `New Wiki Page - b/${boardName}` })

const boardId = ref<string | null>(null)
const isMod = ref(false)
const title = ref('')
const slug = ref('')
const body = ref('')
const saving = ref(false)

const { createPage } = useWiki()

const BOARD_QUERY = `
  query GetBoard($name: String!) {
    board(name: $name) { id }
  }
`

const BOARD_SETTINGS_QUERY = `
  query GetBoardSettings($boardId: ID!) {
    getBoardSettings(boardId: $boardId) {
      moderatorPermissions
    }
  }
`

onMounted(async () => {
  const { execute: execBoard } = useGraphQL<{ board: { id: string } }>()
  const boardResult = await execBoard(BOARD_QUERY, { variables: { name: boardName } })
  if (!boardResult?.board) return

  boardId.value = boardResult.board.id

  const { execute: execSettings } = useGraphQL<{ getBoardSettings: { moderatorPermissions: number | null } }>()
  const settingsResult = await execSettings(BOARD_SETTINGS_QUERY, { variables: { boardId: boardId.value } })
  if (settingsResult?.getBoardSettings?.moderatorPermissions != null) {
    isMod.value = true
  } else {
    await navigateTo(`/b/${boardName}/wiki`)
  }
})

function generateSlug (text: string): string {
  return text
    .toLowerCase()
    .replace(/[^a-z0-9\s-]/g, '')
    .replace(/\s+/g, '-')
    .replace(/-+/g, '-')
    .replace(/^-|-$/g, '')
}

watch(title, (val) => {
  slug.value = generateSlug(val)
})

async function handleSubmit () {
  if (!boardId.value || !title.value.trim() || !slug.value.trim() || !body.value.trim()) return

  saving.value = true
  const result = await createPage(boardId.value, slug.value.trim(), title.value.trim(), body.value)
  saving.value = false

  if (result) {
    toast.success('Wiki page created')
    await navigateTo(`/b/${boardName}/wiki/${result.slug}`)
  } else {
    toast.error('Failed to create wiki page')
  }
}
</script>

<template>
  <div class="max-w-4xl mx-auto px-4 py-4">
    <nav class="text-sm text-gray-500 mb-4">
      <NuxtLink :to="`/b/${boardName}`" class="hover:text-gray-700">b/{{ boardName }}</NuxtLink>
      <span class="mx-1">/</span>
      <NuxtLink :to="`/b/${boardName}/wiki`" class="hover:text-gray-700">Wiki</NuxtLink>
      <span class="mx-1">/</span>
      <span>New Page</span>
    </nav>

    <h1 class="text-lg font-semibold text-gray-900 mb-6">Create Wiki Page</h1>

    <form class="space-y-5 max-w-2xl" @submit.prevent="handleSubmit">
      <div>
        <label class="block text-sm font-medium text-gray-700 mb-1">Title</label>
        <input v-model="title" type="text" class="form-input w-full" placeholder="Page title" required />
      </div>

      <div>
        <label class="block text-sm font-medium text-gray-700 mb-1">Slug</label>
        <input v-model="slug" type="text" class="form-input w-full font-mono text-sm" placeholder="page-slug" required />
        <p class="text-xs text-gray-500 mt-1">Auto-generated from title. Used in the URL.</p>
      </div>

      <div>
        <label class="block text-sm font-medium text-gray-700 mb-1">Content</label>
        <ClientOnly>
          <textarea v-model="body" rows="16" class="form-input w-full font-mono text-sm" placeholder="Write your wiki page content..." required />
        </ClientOnly>
      </div>

      <div class="flex gap-3">
        <button type="submit" class="button primary" :disabled="saving || !title.trim() || !body.trim()">
          {{ saving ? 'Creating...' : 'Create Page' }}
        </button>
        <NuxtLink :to="`/b/${boardName}/wiki`" class="button white">Cancel</NuxtLink>
      </div>
    </form>
  </div>
</template>
