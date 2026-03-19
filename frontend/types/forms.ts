// TODO: Define form data types used by form components

export interface LoginForm {
  usernameOrEmail: string
  password: string
  captchaUuid?: string
  captchaAnswer?: string
}

export interface RegisterForm {
  username: string
  email: string
  password: string
  passwordVerify: string
  showNSFW?: boolean
  captchaUuid?: string
  captchaAnswer?: string
  inviteCode?: string
  applicationText?: string
}

export interface CreatePostForm {
  title: string
  body?: string
  url?: string
  boardId: number
  isNSFW?: boolean
}

export interface CreateCommentForm {
  body: string
  postId: number
  parentId?: number
}

export interface EditBoardForm {
  id: number
  title?: string
  description?: string
  icon?: string
  banner?: string
  isNSFW?: boolean
  sidebar?: string
}
