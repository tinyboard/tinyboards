import { ref } from 'vue'
import type { Ref } from 'vue'
import { useGraphQL } from '~/composables/useGraphQL'
import type { Board } from '~/types/generated'
import type { ApiError } from '~/types/api'

const BOARD_QUERY = `
  query GetBoard($name: String!) {
    board(name: $name) {
      id
      name
      title
      description
      createdAt
      isNSFW
      isHidden
      icon
      banner
      primaryColor
      secondaryColor
      hoverColor
      sidebar
      sidebarHTML
      subscribers
      posts
      comments
      usersActiveDay
      usersActiveWeek
      isSubscribed
      mode
      wikiEnabled
      customCss
    }
  }
`

interface BoardResponse {
  board: Board & { isSubscribed: boolean; mode: string; wikiEnabled: boolean }
}

interface UseBoardReturn {
  board: Ref<(Board & { mode?: string; wikiEnabled?: boolean }) | null>
  isSubscribed: Ref<boolean>
  loading: Ref<boolean>
  error: Ref<ApiError | null>
  fetchBoard: (name: string) => Promise<void>
  subscribe: () => Promise<void>
  unsubscribe: () => Promise<void>
}

export function useBoard (): UseBoardReturn {
  const { execute, loading, error } = useGraphQL<BoardResponse>()
  const board = ref<(Board & { mode?: string; wikiEnabled?: boolean }) | null>(null)
  const isSubscribed = ref(false)

  async function fetchBoard (name: string): Promise<void> {
    const result = await execute(BOARD_QUERY, {
      variables: { name },
    })
    if (result?.board) {
      board.value = result.board
      isSubscribed.value = result.board.isSubscribed ?? false
    }
  }

  const SUBSCRIBE_MUTATION = `
    mutation SubscribeBoard($boardId: ID!) {
      subscribeToBoard(boardId: $boardId)
    }
  `

  const UNSUBSCRIBE_MUTATION = `
    mutation UnsubscribeBoard($boardId: ID!) {
      unsubscribeFromBoard(boardId: $boardId)
    }
  `

  async function subscribe (): Promise<void> {
    if (!board.value) { return }
    const { execute: exec } = useGraphQL<{ subscribeToBoard: boolean }>()
    const result = await exec(SUBSCRIBE_MUTATION, { variables: { boardId: board.value.id } })
    if (result?.subscribeToBoard) {
      isSubscribed.value = true
      if (board.value) {
        board.value = { ...board.value, subscribers: board.value.subscribers + 1 }
      }
    }
  }

  async function unsubscribe (): Promise<void> {
    if (!board.value) { return }
    const { execute: exec } = useGraphQL<{ unsubscribeFromBoard: boolean }>()
    const result = await exec(UNSUBSCRIBE_MUTATION, { variables: { boardId: board.value.id } })
    if (result?.unsubscribeFromBoard) {
      isSubscribed.value = false
      if (board.value) {
        board.value = { ...board.value, subscribers: Math.max(0, board.value.subscribers - 1) }
      }
    }
  }

  return { board, isSubscribed, loading, error, fetchBoard, subscribe, unsubscribe }
}
