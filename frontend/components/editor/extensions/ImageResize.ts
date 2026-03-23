import Image from '@tiptap/extension-image'
import { VueNodeViewRenderer } from '@tiptap/vue-3'
import ImageResizeComponent from '../ImageResizeComponent.vue'

export const ImageResize = Image.extend({
  addAttributes () {
    return {
      ...this.parent?.(),
      width: {
        default: null,
        parseHTML: element => element.getAttribute('width') || element.style.width || null,
        renderHTML: (attributes) => {
          if (!attributes.width) return {}
          return { width: attributes.width }
        },
      },
      height: {
        default: null,
        parseHTML: element => element.getAttribute('height') || null,
        renderHTML: (attributes) => {
          if (!attributes.height) return {}
          return { height: attributes.height }
        },
      },
    }
  },

  addNodeView () {
    return VueNodeViewRenderer(ImageResizeComponent)
  },
})

export default ImageResize
