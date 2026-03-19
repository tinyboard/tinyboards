import { ref } from 'vue'
import { useGraphQL } from '~/composables/useGraphQL'

interface FlairTemplate {
  id: string
  boardId: string
  flairType: 'Post' | 'User'
  templateName: string
  textDisplay: string | null
  textColor: string
  backgroundColor: string
  isModOnly: boolean
  isEditable: boolean
  isActive: boolean
  displayOrder: number
  createdAt: string
}

const FLAIRS_QUERY = `
  query GetBoardFlairs($boardId: ID!, $flairType: FlairType, $activeOnly: Boolean) {
    boardFlairs(boardId: $boardId, flairType: $flairType, activeOnly: $activeOnly) {
      id boardId flairType templateName textDisplay textColor backgroundColor
      isModOnly isEditable isActive displayOrder createdAt
    }
  }
`

const CREATE_FLAIR_MUTATION = `
  mutation CreateFlairTemplate($input: CreateFlairTemplateInput!) {
    createFlairTemplate(input: $input) {
      id boardId flairType templateName textDisplay textColor backgroundColor
      isModOnly isEditable isActive displayOrder createdAt
    }
  }
`

const UPDATE_FLAIR_MUTATION = `
  mutation UpdateFlairTemplate($templateId: ID!, $input: UpdateFlairTemplateInput!) {
    updateFlairTemplate(templateId: $templateId, input: $input) {
      id boardId flairType templateName textDisplay textColor backgroundColor
      isModOnly isEditable isActive displayOrder createdAt
    }
  }
`

const DELETE_FLAIR_MUTATION = `
  mutation DeleteFlairTemplate($templateId: ID!) {
    deleteFlairTemplate(templateId: $templateId)
  }
`

export function useFlairs () {
  const { execute, loading, error } = useGraphQL<{ boardFlairs: FlairTemplate[] }>()
  const flairs = ref<FlairTemplate[]>([])

  async function fetchFlairs (boardId: string, flairType?: 'Post' | 'User'): Promise<void> {
    const result = await execute(FLAIRS_QUERY, {
      variables: { boardId, flairType: flairType || null, activeOnly: true },
    })
    if (result?.boardFlairs) {
      flairs.value = result.boardFlairs
    }
  }

  async function createFlair (data: {
    boardId: string
    flairType: 'Post' | 'User'
    templateName: string
    textDisplay?: string
    textColor?: string
    backgroundColor?: string
    isModOnly?: boolean
    isEditable?: boolean
  }): Promise<FlairTemplate | null> {
    const { execute: exec } = useGraphQL<{ createFlairTemplate: FlairTemplate }>()
    const result = await exec(CREATE_FLAIR_MUTATION, {
      variables: { input: data },
    })
    if (result?.createFlairTemplate) {
      flairs.value.push(result.createFlairTemplate)
      return result.createFlairTemplate
    }
    return null
  }

  async function updateFlair (templateId: string, data: {
    templateName?: string
    textDisplay?: string
    textColor?: string
    backgroundColor?: string
    isModOnly?: boolean
    isEditable?: boolean
    isActive?: boolean
    displayOrder?: number
  }): Promise<FlairTemplate | null> {
    const { execute: exec } = useGraphQL<{ updateFlairTemplate: FlairTemplate }>()
    const result = await exec(UPDATE_FLAIR_MUTATION, {
      variables: { templateId, input: data },
    })
    if (result?.updateFlairTemplate) {
      const idx = flairs.value.findIndex(f => f.id === templateId)
      if (idx !== -1) {
        flairs.value[idx] = result.updateFlairTemplate
      }
      return result.updateFlairTemplate
    }
    return null
  }

  async function deleteFlair (templateId: string): Promise<void> {
    const { execute: exec } = useGraphQL()
    await exec(DELETE_FLAIR_MUTATION, { variables: { templateId } })
    flairs.value = flairs.value.filter(f => f.id !== templateId)
  }

  return {
    flairs,
    loading,
    error,
    fetchFlairs,
    createFlair,
    updateFlair,
    deleteFlair,
  }
}
