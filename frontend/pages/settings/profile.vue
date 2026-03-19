<script setup lang="ts">
import { useGraphQL } from '~/composables/useGraphQL'
import { useFileUpload } from '~/composables/useFileUpload'
import { useAuth } from '~/composables/useAuth'

definePageMeta({ layout: 'settings', middleware: 'guards' })
useHead({ title: 'Profile Settings' })

const { user, fetchMe } = useAuth()
const { uploadFile, uploading } = useFileUpload()

const displayName = ref(user.value?.displayName ?? '')
const bio = ref(user.value?.bio ?? '')
const avatarUrl = ref(user.value?.avatar ?? '')
const bannerUrl = ref(user.value?.banner ?? '')
const profileBackground = ref(user.value?.profileBackground ?? '')
const avatarFrame = ref(user.value?.avatarFrame ?? '')
const signature = ref(user.value?.signature ?? '')
const avatarPreview = ref<string | null>(null)
const bannerPreview = ref<string | null>(null)
const pendingAvatarFile = ref<File | null>(null)
const pendingBannerFile = ref<File | null>(null)
const saving = ref(false)
const success = ref(false)
const saveError = ref<string | null>(null)

// Sync form fields when user data loads asynchronously
watch(() => user.value, (u) => {
  if (u) {
    if (!displayName.value) displayName.value = u.displayName ?? ''
    if (!bio.value) bio.value = u.bio ?? ''
    if (!avatarUrl.value) {
      avatarUrl.value = u.avatar ?? ''
      avatarPreview.value = u.avatar ?? null
    }
    if (!bannerUrl.value) {
      bannerUrl.value = u.banner ?? ''
      bannerPreview.value = u.banner ?? null
    }
    if (!profileBackground.value) profileBackground.value = u.profileBackground ?? ''
    if (!avatarFrame.value) avatarFrame.value = u.avatarFrame ?? ''
    if (!signature.value) signature.value = u.signature ?? ''
  }
}, { immediate: true })

const UPDATE_PROFILE_MUTATION = `
  mutation UpdateProfile($input: UpdateProfileInput!) {
    updateProfile(input: $input) {
      id
      name
      displayName
      bio
      avatar
      banner
      profileBackground
      avatarFrame
      signature
    }
  }
`

function handleAvatarSelect (event: Event): void {
  const input = event.target as HTMLInputElement
  const file = input.files?.[0]
  if (!file) return

  pendingAvatarFile.value = file
  const reader = new FileReader()
  reader.onload = () => {
    avatarPreview.value = reader.result as string
  }
  reader.readAsDataURL(file)
}

function handleBannerSelect (event: Event): void {
  const input = event.target as HTMLInputElement
  const file = input.files?.[0]
  if (!file) return

  pendingBannerFile.value = file
  const reader = new FileReader()
  reader.onload = () => {
    bannerPreview.value = reader.result as string
  }
  reader.readAsDataURL(file)
}

function removeAvatar (): void {
  avatarUrl.value = ''
  avatarPreview.value = null
  pendingAvatarFile.value = null
}

function removeBanner (): void {
  bannerUrl.value = ''
  bannerPreview.value = null
  pendingBannerFile.value = null
}

async function saveProfile (): Promise<void> {
  saving.value = true
  success.value = false
  saveError.value = null

  // Upload pending files first
  if (pendingAvatarFile.value) {
    const url = await uploadFile(pendingAvatarFile.value)
    if (url) {
      avatarUrl.value = url
      pendingAvatarFile.value = null
    } else {
      saveError.value = 'Failed to upload avatar'
      saving.value = false
      return
    }
  }

  if (pendingBannerFile.value) {
    const url = await uploadFile(pendingBannerFile.value)
    if (url) {
      bannerUrl.value = url
      pendingBannerFile.value = null
    } else {
      saveError.value = 'Failed to upload banner'
      saving.value = false
      return
    }
  }

  const { execute, error: mutError } = useGraphQL()
  await execute(UPDATE_PROFILE_MUTATION, {
    variables: {
      input: {
        displayName: displayName.value.trim() || null,
        bio: bio.value.trim() || null,
        avatar: avatarUrl.value || null,
        banner: bannerUrl.value || null,
        profileBackground: profileBackground.value.trim() || null,
        avatarFrame: avatarFrame.value.trim() || null,
        signature: signature.value.trim() || null,
      },
    },
  })

  if (mutError.value) {
    saveError.value = mutError.value.message
  } else {
    success.value = true
    await fetchMe()
    setTimeout(() => { success.value = false }, 3000)
  }

  saving.value = false
}
</script>

<template>
  <div>
    <h2 class="text-lg font-semibold text-gray-900 mb-4">
      Profile
    </h2>

    <form @submit.prevent="saveProfile" class="space-y-6 max-w-lg">
      <!-- Avatar -->
      <div>
        <label class="block text-sm font-medium text-gray-700 mb-2">Avatar</label>
        <div class="flex items-center gap-4">
          <div class="h-20 w-20 rounded-full bg-gray-100 flex items-center justify-center overflow-hidden border border-gray-200">
            <img v-if="avatarPreview" :src="avatarPreview" alt="Avatar" class="h-full w-full object-cover" />
            <svg v-else xmlns="http://www.w3.org/2000/svg" class="h-8 w-8 text-gray-300" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z" />
            </svg>
          </div>
          <div class="flex gap-2">
            <label class="button white button-sm cursor-pointer">
              {{ avatarPreview ? 'Change' : 'Upload' }}
              <input type="file" accept="image/*" class="hidden" @change="handleAvatarSelect" />
            </label>
            <button v-if="avatarPreview" type="button" class="button white button-sm text-red-600" @click="removeAvatar">
              Remove
            </button>
          </div>
        </div>
      </div>

      <!-- Banner -->
      <div>
        <label class="block text-sm font-medium text-gray-700 mb-2">Profile Banner</label>
        <div
          class="w-full h-28 rounded-lg bg-gray-100 flex items-center justify-center overflow-hidden border border-gray-200"
          :style="bannerPreview ? { backgroundImage: `url(${bannerPreview})`, backgroundSize: 'cover', backgroundPosition: 'center' } : {}"
        >
          <span v-if="!bannerPreview" class="text-gray-400 text-xs">No banner (recommended 4:1 aspect ratio)</span>
        </div>
        <div class="flex gap-2 mt-2">
          <label class="button white button-sm cursor-pointer">
            {{ bannerPreview ? 'Change' : 'Upload banner' }}
            <input type="file" accept="image/*" class="hidden" @change="handleBannerSelect" />
          </label>
          <button v-if="bannerPreview" type="button" class="button white button-sm text-red-600" @click="removeBanner">
            Remove
          </button>
        </div>
      </div>

      <!-- Profile Background -->
      <div>
        <label class="block text-sm font-medium text-gray-700 mb-1">Profile Background URL</label>
        <input
          v-model="profileBackground"
          type="url"
          class="form-input"
          placeholder="https://example.com/background.jpg"
        >
        <p class="text-xs text-gray-400 mt-1">Image shown behind your profile header. Leave empty for the default gradient.</p>
      </div>

      <!-- Avatar Frame -->
      <div>
        <label class="block text-sm font-medium text-gray-700 mb-1">Avatar Frame URL</label>
        <input
          v-model="avatarFrame"
          type="url"
          class="form-input"
          placeholder="https://example.com/frame.png"
        >
        <p class="text-xs text-gray-400 mt-1">Decorative frame layered over your avatar.</p>
      </div>

      <!-- Display Name -->
      <div>
        <label class="block text-sm font-medium text-gray-700 mb-1">Display Name</label>
        <input
          v-model="displayName"
          type="text"
          class="form-input"
          placeholder="Your display name"
        >
      </div>

      <!-- Bio -->
      <div>
        <label class="block text-sm font-medium text-gray-700 mb-1">Bio</label>
        <textarea
          v-model="bio"
          class="form-input"
          rows="4"
          placeholder="Tell others about yourself..."
        />
      </div>

      <!-- Signature -->
      <div>
        <label class="block text-sm font-medium text-gray-700 mb-1">Signature</label>
        <textarea
          v-model="signature"
          class="form-input"
          rows="3"
          placeholder="Appears below your comments. Basic HTML allowed."
        />
        <p class="text-xs text-gray-400 mt-1">Shown below your comments to users who have signatures enabled.</p>
      </div>

      <div class="flex items-center gap-3">
        <button type="submit" class="button primary" :disabled="saving || uploading">
          {{ uploading ? 'Uploading...' : saving ? 'Saving...' : 'Save' }}
        </button>
        <span v-if="success" class="text-sm text-green-600">Saved successfully.</span>
        <span v-if="saveError" class="text-sm text-red-600">{{ saveError }}</span>
      </div>
    </form>
  </div>
</template>
