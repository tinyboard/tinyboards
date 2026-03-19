<script setup lang="ts">
import { useGraphQL, useGraphQLMutation } from '~/composables/useGraphQL'

definePageMeta({ layout: 'admin' })
useHead({ title: 'Admin - Create Flair' })

const route = useRoute()
const router = useRouter()

interface FlairTemplate {
  id: string
  templateName: string
}

interface Board {
  id: string
  name: string
  title: string
}

const { execute: fetchBoards, data: boardData } = useGraphQL<{ listBoards: Board[] }>()
const { execute: executeCreate, loading: creating, error: createError } = useGraphQLMutation<{ createFlairTemplate: FlairTemplate }>()

const selectedBoardId = ref((route.query.boardId as string) || '')
const templateName = ref('')
const textDisplay = ref('')
const flairType = ref('Post')
const textColor = ref('#000000')
const backgroundColor = ref('#e0e0e0')
const isModOnly = ref(false)

const BOARDS_QUERY = `
  query { listBoards(limit: 100) { id name title } }
`

const CREATE_FLAIR = `
  mutation CreateFlairTemplate($input: CreateFlairTemplateInput!) {
    createFlairTemplate(input: $input) { id templateName }
  }
`

async function createFlair () {
  if (!selectedBoardId.value || !templateName.value.trim()) return

  const result = await executeCreate(CREATE_FLAIR, {
    variables: {
      input: {
        boardId: selectedBoardId.value,
        flairType: flairType.value,
        templateName: templateName.value.trim(),
        textDisplay: textDisplay.value.trim() || templateName.value.trim(),
        textColor: textColor.value,
        backgroundColor: backgroundColor.value,
        isModOnly: isModOnly.value,
      },
    },
  })

  if (result?.createFlairTemplate) {
    await router.push(`/admin/flairs?boardId=${selectedBoardId.value}`)
  }
}

onMounted(async () => {
  await fetchBoards(BOARDS_QUERY)
})

const boards = computed(() => boardData.value?.listBoards ?? [])
</script>

<template>
  <div>
    <h2 class="text-lg font-semibold text-gray-900 mb-6">
      Create Flair
    </h2>

    <form class="space-y-4 max-w-lg" @submit.prevent="createFlair">
      <div>
        <label class="block text-sm font-medium text-gray-700 mb-1">Board</label>
        <select v-model="selectedBoardId" class="form-input w-full" required>
          <option value="" disabled>
            Select a board...
          </option>
          <option v-for="board in boards" :key="board.id" :value="board.id">
            +{{ board.name }} ({{ board.title }})
          </option>
        </select>
      </div>

      <div>
        <label class="block text-sm font-medium text-gray-700 mb-1">Flair Type</label>
        <select v-model="flairType" class="form-input w-full">
          <option value="Post">Post Flair</option>
          <option value="User">User Flair</option>
        </select>
      </div>

      <div>
        <label class="block text-sm font-medium text-gray-700 mb-1">Name</label>
        <input v-model="templateName" type="text" class="form-input w-full" placeholder="e.g. Discussion" required />
      </div>

      <div>
        <label class="block text-sm font-medium text-gray-700 mb-1">Display Text</label>
        <input v-model="textDisplay" type="text" class="form-input w-full" placeholder="Leave blank to use name" />
      </div>

      <div class="flex gap-4">
        <div>
          <label class="block text-sm font-medium text-gray-700 mb-1">Text Color</label>
          <input v-model="textColor" type="color" class="h-10 w-16 p-1 border border-gray-300 rounded" />
        </div>
        <div>
          <label class="block text-sm font-medium text-gray-700 mb-1">Background</label>
          <input v-model="backgroundColor" type="color" class="h-10 w-16 p-1 border border-gray-300 rounded" />
        </div>
        <div class="flex items-end">
          <span
            class="inline-flex items-center px-2.5 py-1 rounded text-xs font-medium"
            :style="{ color: textColor, backgroundColor: backgroundColor }"
          >
            {{ textDisplay || templateName || 'Preview' }}
          </span>
        </div>
      </div>

      <div>
        <label class="flex items-center gap-2">
          <input v-model="isModOnly" type="checkbox" class="form-checkbox" />
          <span class="text-sm text-gray-700">Mod-only flair</span>
        </label>
      </div>

      <CommonErrorDisplay v-if="createError" :message="createError.message" />

      <div class="flex gap-3">
        <button
          type="submit"
          class="button primary"
          :disabled="creating || !selectedBoardId || !templateName.trim()"
        >
          {{ creating ? 'Creating...' : 'Create Flair' }}
        </button>
        <NuxtLink to="/admin/flairs" class="button white">
          Cancel
        </NuxtLink>
      </div>
    </form>
  </div>
</template>
