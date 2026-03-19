<script setup lang="ts">
import { useGraphQL } from '~/composables/useGraphQL'
import { useToast } from '~/composables/useToast'

definePageMeta({ middleware: 'guards' })
useHead({ title: 'Create Board' })

const toast = useToast()
const loading = ref(false)
const error = ref<string | null>(null)

const form = ref({
  name: '',
  title: '',
  description: '',
  isNsfw: false,
})

const CREATE_BOARD_MUTATION = `
  mutation CreateBoard($input: CreateBoardInput!) {
    createBoard(input: $input) {
      board {
        id
        name
      }
    }
  }
`

function validateName (name: string): string | null {
  if (!name) return 'Board name is required'
  if (name.length > 50) return 'Board name must be 50 characters or less'
  if (name.includes(' ')) return 'Board name cannot contain spaces'
  if (!/^[a-zA-Z0-9_]+$/.test(name)) return 'Board name can only contain letters, numbers, and underscores'
  return null
}

async function handleSubmit (): Promise<void> {
  error.value = null

  const nameError = validateName(form.value.name)
  if (nameError) {
    error.value = nameError
    return
  }

  if (!form.value.title.trim()) {
    error.value = 'Board title is required'
    return
  }

  loading.value = true

  try {
    const { execute } = useGraphQL<{ createBoard: { board: { id: string; name: string } } }>()
    const result = await execute(CREATE_BOARD_MUTATION, {
      variables: {
        input: {
          name: form.value.name,
          title: form.value.title,
          description: form.value.description || null,
          isNsfw: form.value.isNsfw,
        },
      },
    })

    if (result?.createBoard?.board) {
      toast.success('Board created')
      await navigateTo(`/b/${result.createBoard.board.name}`)
    }
  } catch (err: unknown) {
    const gqlError = err as { message?: string }
    error.value = gqlError.message ?? 'Failed to create board'
  } finally {
    loading.value = false
  }
}
</script>

<template>
  <div class="max-w-2xl mx-auto px-4 py-6">
    <h1 class="text-xl font-bold text-gray-900 mb-6">
      Create a Board
    </h1>

    <form class="space-y-5" @submit.prevent="handleSubmit">
      <div>
        <label for="board-name" class="block text-sm font-medium text-gray-700 mb-1">
          Board name
        </label>
        <div class="flex items-center">
          <span class="text-sm text-gray-400 mr-1">+</span>
          <input
            id="board-name"
            v-model="form.name"
            type="text"
            class="form-input flex-1"
            placeholder="myboard"
            required
            maxlength="50"
            pattern="[a-zA-Z0-9_]+"
          >
        </div>
        <p class="text-xs text-gray-400 mt-1">Letters, numbers, and underscores only. Cannot be changed later.</p>
      </div>

      <div>
        <label for="board-title" class="block text-sm font-medium text-gray-700 mb-1">
          Display title
        </label>
        <input
          id="board-title"
          v-model="form.title"
          type="text"
          class="form-input"
          placeholder="My Board"
          required
        >
      </div>

      <div>
        <label for="board-description" class="block text-sm font-medium text-gray-700 mb-1">
          Description
          <span class="text-gray-400 font-normal">(optional)</span>
        </label>
        <textarea
          id="board-description"
          v-model="form.description"
          class="form-input"
          rows="3"
          placeholder="What is this board about?"
        />
      </div>

      <div class="flex items-center gap-2">
        <input
          id="board-nsfw"
          v-model="form.isNsfw"
          type="checkbox"
          class="form-checkbox"
        >
        <label for="board-nsfw" class="text-sm text-gray-700">
          Mark as NSFW
        </label>
      </div>

      <div v-if="error" class="text-sm text-red-600 bg-red-50 border border-red-200 rounded px-3 py-2">
        {{ error }}
      </div>

      <div class="flex items-center gap-3">
        <button
          type="submit"
          class="button primary"
          :disabled="loading"
        >
          <CommonLoadingSpinner v-if="loading" size="sm" />
          <span v-else>Create Board</span>
        </button>
        <NuxtLink to="/boards" class="button white no-underline">
          Cancel
        </NuxtLink>
      </div>
    </form>
  </div>
</template>
