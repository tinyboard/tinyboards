import { ref } from 'vue'
import { useGraphQL } from '~/composables/useGraphQL'

interface WikiPage {
  id: string
  title: string
  slug: string
  body: string
  bodyHTML: string | null
  createdAt: string
  updatedAt: string
  creator: { id: string; name: string } | null
}

interface WikiRevision {
  id: string
  body: string
  editSummary: string | null
  revisionNumber: number
  createdAt: string
  creator: { id: string; name: string } | null
}

const WIKI_PAGE_QUERY = `
  query GetWikiPage($boardName: String!, $slug: String!) {
    wikiPage(boardName: $boardName, slug: $slug) {
      id title slug body bodyHTML createdAt updatedAt
      creator { id name }
    }
  }
`

const WIKI_PAGES_QUERY = `
  query ListWikiPages($boardName: String!) {
    listWikiPages(boardName: $boardName) {
      id title slug updatedAt
    }
  }
`

const CREATE_WIKI_PAGE_MUTATION = `
  mutation CreateWikiPage($boardId: ID!, $input: CreateWikiPageInput!) {
    createWikiPage(boardId: $boardId, input: $input) {
      id title slug
    }
  }
`

const EDIT_WIKI_PAGE_MUTATION = `
  mutation EditWikiPage($pageId: ID!, $input: EditWikiPageInput!) {
    editWikiPage(pageId: $pageId, input: $input) {
      id title slug updatedAt
    }
  }
`

const WIKI_PAGE_HISTORY_QUERY = `
  query WikiPageHistory($pageId: ID!) {
    wikiPageHistory(pageId: $pageId) {
      id body editSummary revisionNumber createdAt
      creator { id name }
    }
  }
`

export function useWiki () {
  const { execute, loading, error } = useGraphQL<{ wikiPage: WikiPage }>()
  const page = ref<WikiPage | null>(null)
  const pages = ref<WikiPage[]>([])
  const loadingPages = ref(false)

  async function fetchPage (boardName: string, slug: string): Promise<void> {
    const result = await execute(WIKI_PAGE_QUERY, {
      variables: { boardName, slug },
    })
    if (result?.wikiPage) {
      page.value = result.wikiPage
    }
  }

  async function fetchPages (boardName: string): Promise<void> {
    loadingPages.value = true
    const { execute: exec } = useGraphQL<{ listWikiPages: WikiPage[] }>()
    const result = await exec(WIKI_PAGES_QUERY, {
      variables: { boardName },
    })
    if (result?.listWikiPages) {
      pages.value = result.listWikiPages
    }
    loadingPages.value = false
  }

  async function createPage (boardId: string, slug: string, title: string, body: string): Promise<WikiPage | null> {
    const { execute: exec } = useGraphQL<{ createWikiPage: WikiPage }>()
    const result = await exec(CREATE_WIKI_PAGE_MUTATION, {
      variables: { boardId, input: { slug, title, body } },
    })
    return result?.createWikiPage ?? null
  }

  async function updatePage (pageId: string, body: string, editSummary?: string): Promise<boolean> {
    const { execute: exec } = useGraphQL()
    const result = await exec(EDIT_WIKI_PAGE_MUTATION, {
      variables: { pageId, input: { body, editSummary: editSummary || null } },
    })
    return result != null
  }

  async function fetchRevisions (pageId: string): Promise<WikiRevision[]> {
    const { execute: exec } = useGraphQL<{ wikiPageHistory: WikiRevision[] }>()
    const result = await exec(WIKI_PAGE_HISTORY_QUERY, {
      variables: { pageId },
    })
    return result?.wikiPageHistory ?? []
  }

  return {
    page,
    pages,
    loading,
    loadingPages,
    error,
    fetchPage,
    fetchPages,
    createPage,
    updatePage,
    fetchRevisions,
  }
}
