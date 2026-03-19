<script setup lang="ts">
import { useStreams } from '~/composables/useStreams'
import { useGraphQL } from '~/composables/useGraphQL'
import { useToast } from '~/composables/useToast'

definePageMeta({ middleware: 'guards' })
useHead({ title: 'Create Stream' })

const toast = useToast()

interface Board {
  id: string
  name: string
  title: string
  icon: string | null
}

const LIST_BOARDS_QUERY = `
  query ListBoards($limit: Int, $sort: SortType) {
    listBoards(limit: $limit, sort: $sort) {
      id name title icon
    }
  }
`

const { createStream, addBoardSubscriptions, mutating } = useStreams()
const { execute: fetchBoards, loading: loadingBoards } = useGraphQL<{ listBoards: Board[] }>()

const boards = ref<Board[]>([])
const selectedBoardIds = ref<Set<string>>(new Set())
const boardSearch = ref('')

const form = reactive({
  name: '',
  description: '',
  isPublic: false,
  isDiscoverable: false,
  sortType: 'hot',
  showNsfw: false,
})

onMounted(async () => {
  const result = await fetchBoards(LIST_BOARDS_QUERY, { variables: { limit: 100, sort: 'active' } })
  boards.value = result?.listBoards ?? []
})

const filteredBoards = computed(() => {
  if (!boardSearch.value) return boards.value
  const q = boardSearch.value.toLowerCase()
  return boards.value.filter(b =>
    b.name.toLowerCase().includes(q) || b.title.toLowerCase().includes(q),
  )
})

function toggleBoard (boardId: string) {
  if (selectedBoardIds.value.has(boardId)) {
    selectedBoardIds.value.delete(boardId)
  } else {
    selectedBoardIds.value.add(boardId)
  }
}

async function handleSubmit () {
  if (!form.name.trim() || selectedBoardIds.value.size === 0) return

  const stream = await createStream({
    name: form.name.trim(),
    description: form.description.trim() || null,
    isPublic: form.isPublic,
    isDiscoverable: form.isDiscoverable,
    sortType: form.sortType,
    showNsfw: form.showNsfw,
  })

  if (stream) {
    await addBoardSubscriptions(stream.id, Array.from(selectedBoardIds.value))
    toast.success('Stream created')
    await navigateTo(`/streams/${stream.id}`)
  } else {
    toast.error('Failed to create stream')
  }
}
</script>

<template>
  <div class="max-w-4xl mx-auto px-4 py-4">
    <h1 class="text-lg font-semibold text-gray-900 mb-6">Create Stream</h1>

    <form class="space-y-5 max-w-2xl" @submit.prevent="handleSubmit">
      <div>
        <label class="block text-sm font-medium text-gray-700 mb-1">Stream Name</label>
        <input v-model="form.name" type="text" class="form-input w-full" placeholder="My Custom Feed" required />
      </div>

      <div>
        <label class="block text-sm font-medium text-gray-700 mb-1">Description</label>
        <textarea v-model="form.description" rows="2" class="form-input w-full" placeholder="What is this stream about?" />
      </div>

      <div>
        <label class="block text-sm font-medium text-gray-700 mb-1">Sort</label>
        <select v-model="form.sortType" class="form-input w-full">
          <option value="hot">Hot</option>
          <option value="new">New</option>
          <option value="top">Top</option>
          <option value="most_comments">Most Comments</option>
          <option value="controversial">Controversial</option>
        </select>
      </div>

      <div class="space-y-3">
        <label class="flex items-center gap-2">
          <input v-model="form.isPublic" type="checkbox" class="form-checkbox" />
          <span class="text-sm text-gray-700">Public (others can view this stream)</span>
        </label>
        <label class="flex items-center gap-2">
          <input v-model="form.isDiscoverable" type="checkbox" class="form-checkbox" />
          <span class="text-sm text-gray-700">Discoverable (appears in stream directory)</span>
        </label>
        <label class="flex items-center gap-2">
          <input v-model="form.showNsfw" type="checkbox" class="form-checkbox" />
          <span class="text-sm text-gray-700">Include NSFW content</span>
        </label>
      </div>

      <!-- Board selection -->
      <div>
        <label class="block text-sm font-medium text-gray-700 mb-1">
          Boards ({{ selectedBoardIds.size }} selected)
        </label>
        <input
          v-model="boardSearch"
          type="text"
          class="form-input w-full mb-2"
          placeholder="Search boards..."
        />
        <CommonLoadingSpinner v-if="loadingBoards" />
        <div v-else class="border rounded-lg max-h-64 overflow-y-auto">
          <label
            v-for="board in filteredBoards"
            :key="board.id"
            class="flex items-center gap-3 px-3 py-2 hover:bg-gray-50 cursor-pointer border-b last:border-b-0"
          >
            <input
              type="checkbox"
              class="form-checkbox"
              :checked="selectedBoardIds.has(board.id)"
              @change="toggleBoard(board.id)"
            />
            <div class="flex items-center gap-2">
              <div class="h-6 w-6 rounded bg-gray-200 overflow-hidden flex-shrink-0">
                <img v-if="board.icon" :src="board.icon" :alt="board.name" class="h-full w-full object-cover" />
              </div>
              <span class="text-sm text-gray-900">+{{ board.name }}</span>
              <span class="text-xs text-gray-500">{{ board.title }}</span>
            </div>
          </label>
        </div>
      </div>

      <div class="flex gap-3">
        <button
          type="submit"
          class="button primary"
          :disabled="mutating || !form.name.trim() || selectedBoardIds.size === 0"
        >
          {{ mutating ? 'Creating...' : 'Create Stream' }}
        </button>
        <NuxtLink to="/streams" class="button white">Cancel</NuxtLink>
      </div>
    </form>
  </div>
</template>
