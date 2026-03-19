import { ref } from 'vue'
import { useGraphQL, useGraphQLMutation } from '~/composables/useGraphQL'

// ============================================================
// Types
// ============================================================

export interface StreamBoard {
  id: string
  name: string
  title: string
  icon: string | null
}

export interface StreamBoardSubscription {
  id: string
  streamId: string
  boardId: string
  board: StreamBoard | null
  includeAllPosts: boolean
}

export interface StreamFlairSubscription {
  id: string
  streamId: string
  boardId: string
  flairId: number
}

export interface StreamData {
  id: string
  name: string
  slug: string
  description: string | null
  icon: string | null
  color: string | null
  creatorId: string
  isPublic: boolean
  isDiscoverable: boolean
  sortType: string
  showNsfw: boolean
  maxPostsPerBoard: number | null
  shareToken: string | null
  createdAt: string
  updatedAt: string
  creator: { id: string; name: string; displayName: string | null; avatar: string | null } | null
  boardSubscriptions: StreamBoardSubscription[] | null
  flairSubscriptions: StreamFlairSubscription[] | null
  followerCount: number | null
  boardSubscriptionCount: number | null
  isFollowing: boolean | null
}

// ============================================================
// GraphQL fragments and queries
// ============================================================

const STREAM_FIELDS = `
  id name slug description icon color creatorId
  isPublic isDiscoverable sortType showNsfw maxPostsPerBoard shareToken
  createdAt updatedAt
  creator { id name displayName avatar }
  boardSubscriptions {
    id streamId boardId includeAllPosts
    board { id name title icon }
  }
  flairSubscriptions { id streamId boardId flairId }
  followerCount boardSubscriptionCount isFollowing
`

const GET_STREAM_QUERY = `
  query GetStream($id: ID, $slug: String) {
    stream(id: $id, slug: $slug) { ${STREAM_FIELDS} }
  }
`

const MY_STREAMS_QUERY = `
  query MyStreams($limit: Int, $offset: Int) {
    myStreams(limit: $limit, offset: $offset) { ${STREAM_FIELDS} }
  }
`

const DISCOVER_STREAMS_QUERY = `
  query DiscoverStreams($sortBy: StreamSortType, $limit: Int, $offset: Int) {
    discoverStreams(sortBy: $sortBy, limit: $limit, offset: $offset) { ${STREAM_FIELDS} }
  }
`

const FOLLOWED_STREAMS_QUERY = `
  query FollowedStreams($limit: Int, $offset: Int) {
    followedStreams(limit: $limit, offset: $offset) { ${STREAM_FIELDS} }
  }
`

const NAVBAR_STREAMS_QUERY = `
  query NavbarStreams {
    navbarStreams { id name slug icon color }
  }
`

const SEARCH_STREAMS_QUERY = `
  query SearchStreams($query: String!, $limit: Int, $offset: Int) {
    searchStreams(query: $query, limit: $limit, offset: $offset) { ${STREAM_FIELDS} }
  }
`

const CREATE_STREAM_MUTATION = `
  mutation CreateStream($input: CreateStreamInput!) {
    createStream(input: $input) { ${STREAM_FIELDS} }
  }
`

const UPDATE_STREAM_MUTATION = `
  mutation UpdateStream($streamId: ID!, $input: UpdateStreamInput!) {
    updateStream(streamId: $streamId, input: $input) { ${STREAM_FIELDS} }
  }
`

const DELETE_STREAM_MUTATION = `
  mutation DeleteStream($streamId: ID!) {
    deleteStream(streamId: $streamId)
  }
`

const ADD_BOARDS_MUTATION = `
  mutation AddBoardSubscriptions($input: AddBoardSubscriptionsInput!) {
    addBoardSubscriptions(input: $input) { id streamId boardId board { id name title icon } includeAllPosts }
  }
`

const REMOVE_BOARD_MUTATION = `
  mutation RemoveBoardSubscription($streamId: ID!, $boardId: ID!) {
    removeBoardSubscription(streamId: $streamId, boardId: $boardId)
  }
`

const ADD_FLAIRS_MUTATION = `
  mutation AddFlairSubscriptions($input: AddFlairSubscriptionsInput!) {
    addFlairSubscriptions(input: $input) { id streamId boardId flairId }
  }
`

const REMOVE_FLAIR_MUTATION = `
  mutation RemoveFlairSubscription($streamId: ID!, $boardId: ID!, $flairId: Int!) {
    removeFlairSubscription(streamId: $streamId, boardId: $boardId, flairId: $flairId)
  }
`

const CLEAR_SUBS_MUTATION = `
  mutation ClearStreamSubscriptions($streamId: ID!) {
    clearStreamSubscriptions(streamId: $streamId)
  }
`

const FOLLOW_MUTATION = `
  mutation FollowStream($streamId: ID!) {
    followStream(streamId: $streamId)
  }
`

const UNFOLLOW_MUTATION = `
  mutation UnfollowStream($streamId: ID!) {
    unfollowStream(streamId: $streamId)
  }
`

const NAVBAR_SETTINGS_MUTATION = `
  mutation UpdateStreamNavbarSettings($streamId: ID!, $addedToNavbar: Boolean!, $navbarPosition: Int) {
    updateStreamNavbarSettings(streamId: $streamId, addedToNavbar: $addedToNavbar, navbarPosition: $navbarPosition)
  }
`

const REGENERATE_TOKEN_MUTATION = `
  mutation RegenerateShareToken($streamId: ID!) {
    regenerateShareToken(streamId: $streamId)
  }
`

// ============================================================
// Composable
// ============================================================

export function useStreams () {
  const { execute, loading, error } = useGraphQL<Record<string, unknown>>()
  const { execute: execMutation, loading: mutating, error: mutationError } = useGraphQLMutation<Record<string, unknown>>()

  const stream = ref<StreamData | null>(null)
  const streams = ref<StreamData[]>([])
  const navbarStreamsList = ref<{ id: string; name: string; slug: string; icon: string | null; color: string | null }[]>([])

  // ---- Queries ----

  async function fetchStream (id?: string, slug?: string): Promise<StreamData | null> {
    const result = await execute(GET_STREAM_QUERY, { variables: { id, slug } })
    const s = (result as { stream: StreamData } | null)?.stream ?? null
    stream.value = s
    return s
  }

  async function fetchMyStreams (limit = 50, offset = 0): Promise<StreamData[]> {
    const result = await execute(MY_STREAMS_QUERY, { variables: { limit, offset } })
    const list = (result as { myStreams: StreamData[] } | null)?.myStreams ?? []
    streams.value = list
    return list
  }

  async function fetchDiscoverStreams (sortBy = 'Popular', limit = 50, offset = 0): Promise<StreamData[]> {
    const result = await execute(DISCOVER_STREAMS_QUERY, { variables: { sortBy, limit, offset } })
    const list = (result as { discoverStreams: StreamData[] } | null)?.discoverStreams ?? []
    streams.value = list
    return list
  }

  async function fetchFollowedStreams (limit = 50, offset = 0): Promise<StreamData[]> {
    const result = await execute(FOLLOWED_STREAMS_QUERY, { variables: { limit, offset } })
    const list = (result as { followedStreams: StreamData[] } | null)?.followedStreams ?? []
    streams.value = list
    return list
  }

  async function fetchNavbarStreams (): Promise<typeof navbarStreamsList.value> {
    const result = await execute(NAVBAR_STREAMS_QUERY)
    const list = (result as { navbarStreams: typeof navbarStreamsList.value } | null)?.navbarStreams ?? []
    navbarStreamsList.value = list
    return list
  }

  async function searchStreams (query: string, limit = 50, offset = 0): Promise<StreamData[]> {
    const result = await execute(SEARCH_STREAMS_QUERY, { variables: { query, limit, offset } })
    const list = (result as { searchStreams: StreamData[] } | null)?.searchStreams ?? []
    streams.value = list
    return list
  }

  // ---- Mutations ----

  async function createStream (input: {
    name: string
    description?: string | null
    isPublic?: boolean
    isDiscoverable?: boolean
    sortType?: string
    showNsfw?: boolean
    maxPostsPerBoard?: number | null
  }): Promise<StreamData | null> {
    const result = await execMutation(CREATE_STREAM_MUTATION, { variables: { input } })
    return (result as { createStream: StreamData } | null)?.createStream ?? null
  }

  async function updateStream (streamId: string, input: {
    name?: string
    description?: string | null
    isPublic?: boolean
    isDiscoverable?: boolean
    sortType?: string
    showNsfw?: boolean
    maxPostsPerBoard?: number | null
  }): Promise<StreamData | null> {
    const result = await execMutation(UPDATE_STREAM_MUTATION, { variables: { streamId, input } })
    return (result as { updateStream: StreamData } | null)?.updateStream ?? null
  }

  async function deleteStream (streamId: string): Promise<boolean> {
    const result = await execMutation(DELETE_STREAM_MUTATION, { variables: { streamId } })
    return (result as { deleteStream: boolean } | null)?.deleteStream ?? false
  }

  // ---- Board subscriptions ----

  async function addBoardSubscriptions (streamId: string, boardIds: string[]): Promise<StreamBoardSubscription[]> {
    const result = await execMutation(ADD_BOARDS_MUTATION, {
      variables: { input: { streamId, boardIds } },
    })
    return (result as { addBoardSubscriptions: StreamBoardSubscription[] } | null)?.addBoardSubscriptions ?? []
  }

  async function removeBoardSubscription (streamId: string, boardId: string): Promise<boolean> {
    const result = await execMutation(REMOVE_BOARD_MUTATION, { variables: { streamId, boardId } })
    return (result as { removeBoardSubscription: boolean } | null)?.removeBoardSubscription ?? false
  }

  // ---- Flair subscriptions ----

  async function addFlairSubscriptions (streamId: string, boardId: string, flairIds: number[]): Promise<StreamFlairSubscription[]> {
    const result = await execMutation(ADD_FLAIRS_MUTATION, {
      variables: { input: { streamId, boardId, flairIds } },
    })
    return (result as { addFlairSubscriptions: StreamFlairSubscription[] } | null)?.addFlairSubscriptions ?? []
  }

  async function removeFlairSubscription (streamId: string, boardId: string, flairId: number): Promise<boolean> {
    const result = await execMutation(REMOVE_FLAIR_MUTATION, { variables: { streamId, boardId, flairId } })
    return (result as { removeFlairSubscription: boolean } | null)?.removeFlairSubscription ?? false
  }

  async function clearSubscriptions (streamId: string): Promise<boolean> {
    const result = await execMutation(CLEAR_SUBS_MUTATION, { variables: { streamId } })
    return (result as { clearStreamSubscriptions: boolean } | null)?.clearStreamSubscriptions ?? false
  }

  // ---- Follow / unfollow ----

  async function followStream (streamId: string): Promise<boolean> {
    const result = await execMutation(FOLLOW_MUTATION, { variables: { streamId } })
    return (result as { followStream: boolean } | null)?.followStream ?? false
  }

  async function unfollowStream (streamId: string): Promise<boolean> {
    const result = await execMutation(UNFOLLOW_MUTATION, { variables: { streamId } })
    return (result as { unfollowStream: boolean } | null)?.unfollowStream ?? false
  }

  // ---- Navbar settings ----

  async function updateNavbarSettings (streamId: string, addedToNavbar: boolean, navbarPosition?: number): Promise<boolean> {
    const result = await execMutation(NAVBAR_SETTINGS_MUTATION, {
      variables: { streamId, addedToNavbar, navbarPosition: navbarPosition ?? null },
    })
    return (result as { updateStreamNavbarSettings: boolean } | null)?.updateStreamNavbarSettings ?? false
  }

  // ---- Share token ----

  async function regenerateShareToken (streamId: string): Promise<string | null> {
    const result = await execMutation(REGENERATE_TOKEN_MUTATION, { variables: { streamId } })
    return (result as { regenerateShareToken: string } | null)?.regenerateShareToken ?? null
  }

  return {
    // State
    stream,
    streams,
    navbarStreamsList,
    loading,
    error,
    mutating,
    mutationError,

    // Queries
    fetchStream,
    fetchMyStreams,
    fetchDiscoverStreams,
    fetchFollowedStreams,
    fetchNavbarStreams,
    searchStreams,

    // Mutations
    createStream,
    updateStream,
    deleteStream,

    // Board subscriptions
    addBoardSubscriptions,
    removeBoardSubscription,

    // Flair subscriptions
    addFlairSubscriptions,
    removeFlairSubscription,
    clearSubscriptions,

    // Follow
    followStream,
    unfollowStream,

    // Navbar
    updateNavbarSettings,

    // Share
    regenerateShareToken,
  }
}
