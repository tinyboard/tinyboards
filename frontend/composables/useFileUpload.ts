import { ref } from 'vue'
import type { Ref } from 'vue'
import type { ApiError } from '~/types/api'

interface UseFileUploadReturn {
  uploading: Ref<boolean>
  error: Ref<ApiError | null>
  uploadFile: (file: File) => Promise<string | null>
  executeWithFile: (query: string, variables: Record<string, unknown>, fileField: string, file: File) => Promise<Record<string, unknown> | null>
}

/**
 * Composable for uploading files via the GraphQL multipart upload spec.
 *
 * - `uploadFile(file)` calls the `uploadFile` mutation and returns the URL
 * - `executeWithFile(query, variables, fileField, file)` runs any GraphQL mutation
 *   with a file variable mapped to the given field path
 *
 * All requests go through `/api/graphql-upload` (the Nuxt BFF multipart proxy).
 */
export function useFileUpload (): UseFileUploadReturn {
  const uploading = ref(false)
  const error: Ref<ApiError | null> = ref(null)

  async function sendMultipart (
    query: string,
    variables: Record<string, unknown>,
    fileMap: Record<string, string[]>,
    files: Record<string, File>,
  ): Promise<Record<string, unknown> | null> {
    uploading.value = true
    error.value = null

    try {
      const formData = new FormData()

      // Set file placeholders to null in variables for the operations field
      const ops = { query, variables }
      formData.append('operations', JSON.stringify(ops))
      formData.append('map', JSON.stringify(fileMap))

      // Append file fields
      for (const [key, file] of Object.entries(files)) {
        formData.append(key, file, file.name)
      }

      const response = await $fetch<{ data?: Record<string, unknown>; errors?: Array<{ message: string }> }>(
        '/api/graphql-upload',
        {
          method: 'POST',
          headers: {
            'X-Requested-With': 'XMLHttpRequest',
          },
          body: formData,
        },
      )

      if (response.errors && response.errors.length > 0) {
        error.value = response.errors[0]
        return null
      }

      return response.data ?? null
    } catch (err: unknown) {
      const message = err instanceof Error ? err.message : 'Upload failed'
      error.value = { message }
      return null
    } finally {
      uploading.value = false
    }
  }

  async function uploadFile (file: File): Promise<string | null> {
    const query = `
      mutation UploadFile($file: Upload!) {
        uploadFile(file: $file)
      }
    `
    const variables = { file: null } as Record<string, unknown>
    const result = await sendMultipart(
      query,
      variables,
      { file: ['variables.file'] },
      { file },
    )
    return (result?.uploadFile as string) ?? null
  }

  async function executeWithFile (
    query: string,
    variables: Record<string, unknown>,
    fileField: string,
    file: File,
  ): Promise<Record<string, unknown> | null> {
    // Set the file variable to null (placeholder for multipart spec)
    const vars = { ...variables, [fileField]: null }
    return sendMultipart(
      query,
      vars,
      { [fileField]: [`variables.${fileField}`] },
      { [fileField]: file },
    )
  }

  return { uploading, error, uploadFile, executeWithFile }
}
