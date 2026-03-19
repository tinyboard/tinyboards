<script setup lang="ts">
import { useWiki } from '~/composables/useWiki'
import { useGraphQL } from '~/composables/useGraphQL'
import { useToast } from '~/composables/useToast'

definePageMeta({ middleware: 'guards' })

const route = useRoute()
const boardName = route.params.board as string
const slug = route.params.slug as string
const toast = useToast()

useHead({ title: `Edit - ${slug}` })

const { page, loading, fetchPage, updatePage } = useWiki()
const boardId = ref<string | null>(null)
const isMod = ref(false)
const title = ref('')
const body = ref('')
const saving = ref(false)

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
    await navigateTo(`/b/${boardName}/wiki/${slug}`)
    return
  }

  await fetchPage(boardName, slug)
  if (page.value) {
    title.value = page.value.title
    body.value = page.value.body
  }
})

async function handleSubmit () {
  if (!page.value || !body.value.trim()) return

  saving.value = true
  const success = await updatePage(page.value.id, body.value)
  saving.value = false

  if (success) {
    toast.success('Wiki page updated')
    await navigateTo(`/b/${boardName}/wiki/${slug}`)
  } else {
    toast.error('Failed to update wiki page')
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
      <NuxtLink :to="`/b/${boardName}/wiki/${slug}`" class="hover:text-gray-700">{{ page?.title ?? slug }}</NuxtLink>
      <span class="mx-1">/</span>
      <span>Edit</span>
    </nav>

    <h1 class="text-lg font-semibold text-gray-900 mb-6">Edit Wiki Page</h1>

    <CommonLoadingSpinner v-if="loading" />

    <form v-else-if="page" class="space-y-5 max-w-2xl" @submit.prevent="handleSubmit">
      <div>
        <label class="block text-sm font-medium text-gray-700 mb-1">Title</label>
        <input :value="title" type="text" class="form-input w-full" disabled />
        <p class="text-xs text-gray-500 mt-1">Title cannot be changed after creation.</p>
      </div>

      <div>
        <label class="block text-sm font-medium text-gray-700 mb-1">Content</label>
        <ClientOnly>
          <textarea v-model="body" rows="16" class="form-input w-full font-mono text-sm" required />
        </ClientOnly>
      </div>

      <div class="flex gap-3">
        <button type="submit" class="button primary" :disabled="saving || !body.trim()">
          {{ saving ? 'Saving...' : 'Save Changes' }}
        </button>
        <NuxtLink :to="`/b/${boardName}/wiki/${slug}`" class="button white">Cancel</NuxtLink>
      </div>
    </form>
  </div>
</template>
