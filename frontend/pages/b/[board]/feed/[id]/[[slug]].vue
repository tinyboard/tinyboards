<script setup lang="ts">
const route = useRoute()
const boardName = route.params.board as string
const postId = route.params.id as string
const slug = route.params.slug as string | undefined

const slugPart = slug ? `/${slug}` : ''
await navigateTo(`/b/${boardName}/${postId}${slugPart}`, { replace: true, redirectCode: 301 })
</script>
