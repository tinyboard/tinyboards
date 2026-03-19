<script setup lang="ts">
import { useStreams, type StreamData, type StreamFlairSubscription } from '~/composables/useStreams'
import { useGraphQL } from '~/composables/useGraphQL'
import { useToast } from '~/composables/useToast'

definePageMeta({ middleware: 'guards' })
useHead({ title: 'Edit Stream' })

const route = useRoute()
const streamId = route.params.id as string
const toast = useToast()

interface Board {
  id: string
  name: string
  title: string
  icon: string | null
}

interface FlairTemplate {
  id: string
  name: string
  textColor: string | null
  bgColor: string | null
}

const LIST_BOARDS_QUERY = `
  query ListBoards($limit: Int, $sort: SortType) {
    listBoards(limit: $limit, sort: $sort) {
      id name title icon
    }
  }
`

const BOARD_FLAIRS_QUERY = `
  query BoardFlairs($boardId: ID!, $flairType: FlairType) {
    boardFlairs(boardId: $boardId, flairType: $flairType, activeOnly: true) {
      id name textColor bgColor
    }
  }
`

const {
  stream,
  loading: loadingStream,
  fetchStream,
  updateStream,
  deleteStream,
  addBoardSubscriptions,
  removeBoardSubscription,
  addFlairSubscriptions,
  removeFlairSubscription,
  mutating,
} = useStreams()

const { execute: fetchBoards, loading: loadingBoards } = useGraphQL<{ listBoards: Board[] }>()
const { execute: fetchFlairs } = useGraphQL<{ boardFlairs: FlairTemplate[] }>()

const allBoards = ref<Board[]>([])
const selectedBoardIds = ref<Set<string>>(new Set())
const originalBoardIds = ref<Set<string>>(new Set())
const boardSearch = ref('')
const confirmDelete = ref(false)

// Flair subscriptions state
const expandedBoardId = ref<string | null>(null)
const boardFlairs = ref<FlairTemplate[]>([])
const loadingFlairs = ref(false)
const existingFlairSubs = ref<StreamFlairSubscription[]>([])

const form = reactive({
  name: '',
  description: '',
  isPublic: false,
  isDiscoverable: false,
  sortType: 'hot',
  showNsfw: false,
})

onMounted(async () => {
  const [streamData, boardsResult] = await Promise.all([
    fetchStream(streamId),
    fetchBoards(LIST_BOARDS_QUERY, { variables: { limit: 100, sort: 'active' } }),
  ])

  if (streamData) {
    form.name = streamData.name
    form.description = streamData.description ?? ''
    form.isPublic = streamData.isPublic
    form.isDiscoverable = streamData.isDiscoverable
    form.sortType = streamData.sortType
    form.showNsfw = streamData.showNsfw

    const ids = (streamData.boardSubscriptions ?? []).map(s => s.boardId)
    selectedBoardIds.value = new Set(ids)
    originalBoardIds.value = new Set(ids)
    existingFlairSubs.value = streamData.flairSubscriptions ?? []
  }

  allBoards.value = boardsResult?.listBoards ?? []
})

const filteredBoards = computed(() => {
  if (!boardSearch.value) return allBoards.value
  const q = boardSearch.value.toLowerCase()
  return allBoards.value.filter(b =>
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

function flairSubsForBoard (boardId: string): StreamFlairSubscription[] {
  return existingFlairSubs.value.filter(f => f.boardId === boardId)
}

async function toggleFlairPanel (boardId: string) {
  if (expandedBoardId.value === boardId) {
    expandedBoardId.value = null
    return
  }
  expandedBoardId.value = boardId
  loadingFlairs.value = true
  const result = await fetchFlairs(BOARD_FLAIRS_QUERY, {
    variables: { boardId, flairType: 'post' },
  })
  boardFlairs.value = result?.boardFlairs ?? []
  loadingFlairs.value = false
}

function isFlairSubscribed (flairId: string): boolean {
  return existingFlairSubs.value.some(
    f => f.boardId === expandedBoardId.value && f.flairId === Number(flairId),
  )
}

async function handleToggleFlair (flairId: string) {
  if (!expandedBoardId.value) return
  const boardId = expandedBoardId.value

  if (isFlairSubscribed(flairId)) {
    await removeFlairSubscription(streamId, boardId, Number(flairId))
    existingFlairSubs.value = existingFlairSubs.value.filter(
      f => !(f.boardId === boardId && f.flairId === Number(flairId)),
    )
    toast.success('Flair filter removed')
  } else {
    const result = await addFlairSubscriptions(streamId, boardId, [Number(flairId)])
    existingFlairSubs.value.push(...result)
    toast.success('Flair filter added')
  }
}

async function handleSubmit () {
  // Update stream settings
  await updateStream(streamId, {
    name: form.name.trim(),
    description: form.description.trim() || null,
    isPublic: form.isPublic,
    isDiscoverable: form.isDiscoverable,
    sortType: form.sortType,
    showNsfw: form.showNsfw,
  })

  // Handle board subscription changes
  const added = [...selectedBoardIds.value].filter(id => !originalBoardIds.value.has(id))
  const removed = [...originalBoardIds.value].filter(id => !selectedBoardIds.value.has(id))

  if (added.length > 0) {
    await addBoardSubscriptions(streamId, added)
  }

  for (const boardId of removed) {
    await removeBoardSubscription(streamId, boardId)
  }

  toast.success('Stream updated')
  await navigateTo(`/streams/${streamId}`)
}

async function handleDelete () {
  await deleteStream(streamId)
  toast.success('Stream deleted')
  await navigateTo('/streams')
}
</script>

<template>
  <div class="max-w-4xl mx-auto px-4 py-4">
    <h1 class="text-lg font-semibold text-gray-900 mb-6">Edit Stream</h1>

    <CommonLoadingSpinner v-if="loadingStream" />

    <form v-else-if="stream" class="space-y-5 max-w-2xl" @submit.prevent="handleSubmit">
      <div>
        <label class="block text-sm font-medium text-gray-700 mb-1">Stream Name</label>
        <input v-model="form.name" type="text" class="form-input w-full" required />
      </div>

      <div>
        <label class="block text-sm font-medium text-gray-700 mb-1">Description</label>
        <textarea v-model="form.description" rows="2" class="form-input w-full" />
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
          <span class="text-sm text-gray-700">Public</span>
        </label>
        <label class="flex items-center gap-2">
          <input v-model="form.isDiscoverable" type="checkbox" class="form-checkbox" />
          <span class="text-sm text-gray-700">Discoverable</span>
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
          <div
            v-for="board in filteredBoards"
            :key="board.id"
            class="border-b last:border-b-0"
          >
            <label class="flex items-center gap-3 px-3 py-2 hover:bg-gray-50 cursor-pointer">
              <input
                type="checkbox"
                class="form-checkbox"
                :checked="selectedBoardIds.has(board.id)"
                @change="toggleBoard(board.id)"
              />
              <span class="text-sm text-gray-900 flex-1">+{{ board.name }}</span>
              <span class="text-xs text-gray-500">{{ board.title }}</span>
              <button
                v-if="selectedBoardIds.has(board.id)"
                type="button"
                class="text-xs text-primary hover:underline"
                @click.prevent="toggleFlairPanel(board.id)"
              >
                {{ expandedBoardId === board.id ? 'Hide' : 'Flairs' }}
                <span v-if="flairSubsForBoard(board.id).length > 0" class="text-gray-400">
                  ({{ flairSubsForBoard(board.id).length }})
                </span>
              </button>
            </label>

            <!-- Flair subscriptions panel -->
            <div
              v-if="expandedBoardId === board.id"
              class="bg-gray-50 px-4 py-2 border-t"
            >
              <p class="text-xs text-gray-500 mb-2">
                Filter posts by flair. If no flairs are selected, all posts from this board are included.
              </p>
              <CommonLoadingSpinner v-if="loadingFlairs" size="sm" />
              <div v-else-if="boardFlairs.length === 0" class="text-xs text-gray-400">
                No flairs available for this board.
              </div>
              <div v-else class="flex flex-wrap gap-2">
                <button
                  v-for="flair in boardFlairs"
                  :key="flair.id"
                  type="button"
                  class="inline-flex items-center px-2 py-1 rounded text-xs transition-colors"
                  :class="isFlairSubscribed(flair.id)
                    ? 'bg-primary text-white'
                    : 'bg-white border border-gray-300 text-gray-700 hover:border-gray-400'"
                  :style="isFlairSubscribed(flair.id) ? {} : {
                    backgroundColor: flair.bgColor ?? undefined,
                    color: flair.textColor ?? undefined,
                  }"
                  @click="handleToggleFlair(flair.id)"
                >
                  {{ flair.name }}
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>

      <div class="flex gap-3">
        <button type="submit" class="button primary" :disabled="mutating || !form.name.trim()">
          {{ mutating ? 'Saving...' : 'Save Changes' }}
        </button>
        <NuxtLink :to="`/streams/${streamId}`" class="button white">Cancel</NuxtLink>
      </div>

      <!-- Delete stream -->
      <div class="border-t pt-5 mt-8">
        <h3 class="text-sm font-medium text-red-600 mb-2">Danger Zone</h3>
        <template v-if="confirmDelete">
          <p class="text-sm text-gray-600 mb-3">Are you sure? This cannot be undone.</p>
          <div class="flex gap-3">
            <button type="button" class="button button-sm bg-red-600 text-white hover:bg-red-700" @click="handleDelete">
              Delete Stream
            </button>
            <button type="button" class="button white button-sm" @click="confirmDelete = false">Cancel</button>
          </div>
        </template>
        <button v-else type="button" class="button white button-sm text-red-600" @click="confirmDelete = true">
          Delete this stream
        </button>
      </div>
    </form>
  </div>
</template>
