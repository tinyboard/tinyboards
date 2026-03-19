// TODO: These types will be superseded by types/generated.ts from GraphQL codegen.
// Manual types here serve as reference for domain concepts not in the schema.

export type SortType =
  | 'active'
  | 'hot'
  | 'new'
  | 'old'
  | 'topDay'
  | 'topWeek'
  | 'topMonth'
  | 'topYear'
  | 'topAll'
  | 'mostComments'
  | 'newComments'

export type ListingType = 'all' | 'subscribed' | 'local' | 'moderated'

export type SubscribedType = 'subscribed' | 'not_subscribed' | 'pending'

export type UserSortType = 'new' | 'old' | 'mostRep' | 'mostPosts' | 'mostComments'

export type UserListingType = 'all' | 'admins' | 'banned'

export type RegistrationMode =
  | 'open'
  | 'invite_only'
  | 'application'
  | 'email_verification'
  | 'closed'

export type PostType = 'link' | 'text' | 'image' | 'video'

export type WikiPermission = 'disabled' | 'mod_only' | 'subscriber_only' | 'anyone'
