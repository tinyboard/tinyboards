import { Extension } from '@tiptap/core'
import { Plugin, PluginKey } from '@tiptap/pm/state'

export interface ImageUploadOptions {
  /** Upload function that takes a File and returns a URL string */
  uploadFn: (file: File) => Promise<string | null>
  /** Accepted file types */
  acceptedTypes: string[]
  /** Max file size in bytes (default 10MB) */
  maxSize: number
  /** Callback when upload starts */
  onUploadStart?: () => void
  /** Callback when upload ends */
  onUploadEnd?: () => void
}

export const ImageUpload = Extension.create<ImageUploadOptions>({
  name: 'imageUpload',

  addOptions () {
    return {
      uploadFn: async () => null,
      acceptedTypes: ['image/jpeg', 'image/png', 'image/gif', 'image/webp', 'image/svg+xml'],
      maxSize: 10 * 1024 * 1024,
      onUploadStart: undefined,
      onUploadEnd: undefined,
    }
  },

  addProseMirrorPlugins () {
    const options = this.options
    const editor = this.editor

    return [
      new Plugin({
        key: new PluginKey('imageUpload'),
        props: {
          handleDrop (view, event) {
            const files = event.dataTransfer?.files
            if (!files || files.length === 0) return false

            const imageFiles = Array.from(files).filter(
              f => options.acceptedTypes.includes(f.type) && f.size <= options.maxSize,
            )
            if (imageFiles.length === 0) return false

            event.preventDefault()

            for (const file of imageFiles) {
              handleUpload(file)
            }
            return true
          },

          handlePaste (_view, event) {
            const items = event.clipboardData?.items
            if (!items) return false

            const imageItems = Array.from(items).filter(
              item => item.kind === 'file' && options.acceptedTypes.includes(item.type),
            )
            if (imageItems.length === 0) return false

            event.preventDefault()

            for (const item of imageItems) {
              const file = item.getAsFile()
              if (file && file.size <= options.maxSize) {
                handleUpload(file)
              }
            }
            return true
          },
        },
      }),
    ]

    async function handleUpload (file: File): Promise<void> {
      options.onUploadStart?.()
      try {
        const url = await options.uploadFn(file)
        if (url) {
          editor.chain().focus().setImage({ src: url }).run()
        }
      } finally {
        options.onUploadEnd?.()
      }
    }
  },
})

export default ImageUpload
