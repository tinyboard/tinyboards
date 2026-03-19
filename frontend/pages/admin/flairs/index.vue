<script setup lang="ts">
import { useGraphQL, useGraphQLMutation } from '~/composables/useGraphQL'

definePageMeta({ layout: 'admin' })
useHead({ title: 'Admin - Flairs' })

interface FlairTemplate {
  id: string
  boardId: string
  flairType: string
  templateName: string
  textDisplay: string
  textColor: string
  backgroundColor: string
  isModOnly: boolean
  isActive: boolean
  usageCount: number
}

interface Board {
  id: string
  name: string
  title: string
}

const { execute: fetchBoards, data: boardData } = useGraphQL<{ listBoards: Board[] }>()
const { execute: fetchFlairs, loading, error, data: flairData } = useGraphQL<{ manageBoardFlairs: FlairTemplate[] }>()
const { execute: executeDelete, loading: deleting } = useGraphQLMutation<{ deleteFlairTemplate: boolean }>()

const selectedBoardId = ref<string | null>(null)

const BOARDS_QUERY = `
  query { listBoards(limit: 100) { id name title } }
`

const FLAIRS_QUERY = `
  query ManageBoardFlairs($boardId: ID!) {
    manageBoardFlairs(boardId: $boardId) {
      id boardId flairType templateName textDisplay textColor backgroundColor
      isModOnly isActive usageCount
    }
  }
`

const DELETE_FLAIR = `
  mutation DeleteFlairTemplate($templateId: ID!) {
    deleteFlairTemplate(templateId: $templateId)
  }
`

async function loadFlairs () {
  if (!selectedBoardId.value) return
  await fetchFlairs(FLAIRS_QUERY, { variables: { boardId: selectedBoardId.value } })
}

async function deleteFlair (id: string) {
  await executeDelete(DELETE_FLAIR, { variables: { templateId: id } })
  await loadFlairs()
}

async function selectBoard (boardId: string) {
  selectedBoardId.value = boardId
  await loadFlairs()
}

onMounted(async () => {
  await fetchBoards(BOARDS_QUERY)
})

const boards = computed(() => boardData.value?.listBoards ?? [])
const flairs = computed(() => flairData.value?.manageBoardFlairs ?? [])
</script>

<template>
  <div>
    <div class="flex items-center justify-between mb-6">
      <h2 class="text-lg font-semibold text-gray-900">
        Flair Management
      </h2>
      <NuxtLink
        v-if="selectedBoardId"
        :to="`/admin/flairs/create?boardId=${selectedBoardId}`"
        class="button primary button-sm"
      >
        Create Flair
      </NuxtLink>
    </div>

    <p class="text-sm text-gray-500 mb-4">
      Flairs are managed per-board. Select a board to view and manage its flairs.
    </p>

    <div class="mb-6">
      <label class="block text-sm font-medium text-gray-700 mb-1">Board</label>
      <select
        class="form-input w-full max-w-sm"
        :value="selectedBoardId ?? ''"
        @change="selectBoard(($event.target as HTMLSelectElement).value)"
      >
        <option value="" disabled>
          Select a board...
        </option>
        <option v-for="board in boards" :key="board.id" :value="board.id">
          +{{ board.name }} ({{ board.title }})
        </option>
      </select>
    </div>

    <template v-if="selectedBoardId">
      <CommonLoadingSpinner v-if="loading" />
      <CommonErrorDisplay v-else-if="error" :message="error.message" />

      <div v-else-if="flairs.length === 0" class="text-sm text-gray-500">
        No flairs found for this board.
      </div>

      <div v-else class="space-y-3">
        <div
          v-for="flair in flairs"
          :key="flair.id"
          class="bg-white rounded-lg border border-gray-200 p-4 flex items-center justify-between"
        >
          <div class="flex items-center gap-3">
            <span
              class="inline-flex items-center px-2.5 py-0.5 rounded text-xs font-medium"
              :style="{ color: flair.textColor, backgroundColor: flair.backgroundColor }"
            >
              {{ flair.textDisplay || flair.templateName }}
            </span>
            <div>
              <span class="text-sm text-gray-900">{{ flair.templateName }}</span>
              <span class="ml-2 text-xs text-gray-500">
                {{ flair.flairType }} &middot; {{ flair.usageCount }} uses
              </span>
              <span v-if="flair.isModOnly" class="ml-2 text-xs text-orange-600">
                Mod only
              </span>
              <span v-if="!flair.isActive" class="ml-2 text-xs text-red-600">
                Inactive
              </span>
            </div>
          </div>
          <div class="flex items-center gap-2 shrink-0">
            <NuxtLink :to="`/admin/flairs/${flair.id}/edit`" class="button button-sm white">
              Edit
            </NuxtLink>
            <button
              class="button button-sm red"
              :disabled="deleting"
              @click="deleteFlair(flair.id)"
            >
              Delete
            </button>
          </div>
        </div>
      </div>
    </template>
  </div>
</template>
