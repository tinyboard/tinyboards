export type Maybe<T> = T | null | undefined;
export type InputMaybe<T> = T | null | undefined;
export type Exact<T extends { [key: string]: unknown }> = { [K in keyof T]: T[K] };
export type MakeOptional<T, K extends keyof T> = Omit<T, K> & { [SubKey in K]?: Maybe<T[SubKey]> };
export type MakeMaybe<T, K extends keyof T> = Omit<T, K> & { [SubKey in K]: Maybe<T[SubKey]> };
export type MakeEmpty<T extends { [key: string]: unknown }, K extends keyof T> = { [_ in K]?: never };
export type Incremental<T> = T | { [P in keyof T]?: P extends ' $fragmentName' | '__typename' ? T[P] : never };
/** All built-in and custom scalars, mapped to their actual values */
export type Scalars = {
  ID: { input: string; output: string; }
  String: { input: string; output: string; }
  Boolean: { input: boolean; output: boolean; }
  Int: { input: number; output: number; }
  Float: { input: number; output: number; }
  DateTime: { input: string; output: string; }
  JSON: { input: Record<string, unknown>; output: Record<string, unknown>; }
  Upload: { input: File; output: File; }
};

export type AddModeratorResponse = {
  __typename?: 'AddModeratorResponse';
  board: Board;
  success: Scalars['Boolean']['output'];
};

export type AddReactionInput = {
  commentId?: InputMaybe<Scalars['ID']['input']>;
  emoji: Scalars['String']['input'];
  postId?: InputMaybe<Scalars['ID']['input']>;
};

export type AddReactionResponse = {
  __typename?: 'AddReactionResponse';
  reaction: Reaction;
};

export type AssignPostFlairInput = {
  customBackgroundColor?: InputMaybe<Scalars['String']['input']>;
  customText?: InputMaybe<Scalars['String']['input']>;
  customTextColor?: InputMaybe<Scalars['String']['input']>;
  flairTemplateId: Scalars['ID']['input'];
  postId: Scalars['ID']['input'];
};

export type AssignUserFlairInput = {
  boardId: Scalars['ID']['input'];
  customBackgroundColor?: InputMaybe<Scalars['String']['input']>;
  customText?: InputMaybe<Scalars['String']['input']>;
  customTextColor?: InputMaybe<Scalars['String']['input']>;
  flairTemplateId: Scalars['ID']['input'];
  userId: Scalars['ID']['input'];
};

export type BanUserInput = {
  expiresDays?: InputMaybe<Scalars['Int']['input']>;
  reason?: InputMaybe<Scalars['String']['input']>;
  userId: Scalars['ID']['input'];
};

export type BanUserResponse = {
  __typename?: 'BanUserResponse';
  banId: Scalars['ID']['output'];
  message: Scalars['String']['output'];
  success: Scalars['Boolean']['output'];
};

export type BannedUsersResponse = {
  __typename?: 'BannedUsersResponse';
  totalCount: Scalars['Int']['output'];
  users: Array<User>;
};

export type Board = {
  __typename?: 'Board';
  banner?: Maybe<Scalars['String']['output']>;
  comments: Scalars['Int']['output'];
  createdAt: Scalars['String']['output'];
  description?: Maybe<Scalars['String']['output']>;
  excludeFromAll: Scalars['Boolean']['output'];
  hoverColor: Scalars['String']['output'];
  icon?: Maybe<Scalars['String']['output']>;
  id: Scalars['ID']['output'];
  isBanned: Scalars['Boolean']['output'];
  isHidden: Scalars['Boolean']['output'];
  isNSFW: Scalars['Boolean']['output'];
  isPostingRestrictedToMods: Scalars['Boolean']['output'];
  isRemoved: Scalars['Boolean']['output'];
  isSubscribed: Scalars['Boolean']['output'];
  name: Scalars['String']['output'];
  posts: Scalars['Int']['output'];
  primaryColor: Scalars['String']['output'];
  publicBanReason?: Maybe<Scalars['String']['output']>;
  secondaryColor: Scalars['String']['output'];
  mode: Scalars['String']['output'];
  sidebar?: Maybe<Scalars['String']['output']>;
  sidebarHTML?: Maybe<Scalars['String']['output']>;
  subscribers: Scalars['Int']['output'];
  title: Scalars['String']['output'];
  updatedAt: Scalars['String']['output'];
  usersActiveDay: Scalars['Int']['output'];
  usersActiveHalfYear: Scalars['Int']['output'];
  usersActiveMonth: Scalars['Int']['output'];
  usersActiveWeek: Scalars['Int']['output'];
  wikiEnabled: Scalars['Boolean']['output'];
  customCss?: Maybe<Scalars['String']['output']>;
};

export type BoardBanResponse = {
  __typename?: 'BoardBanResponse';
  banId: Scalars['ID']['output'];
  message: Scalars['String']['output'];
  success: Scalars['Boolean']['output'];
};

export type BoardBanUserInput = {
  boardId: Scalars['ID']['input'];
  expiresDays?: InputMaybe<Scalars['Int']['input']>;
  reason?: InputMaybe<Scalars['String']['input']>;
  userId: Scalars['ID']['input'];
};

export type BoardBannedUser = {
  __typename?: 'BoardBannedUser';
  banDate: Scalars['String']['output'];
  boardId: Scalars['ID']['output'];
  expires?: Maybe<Scalars['String']['output']>;
  id: Scalars['ID']['output'];
  user: User;
};

export type BoardModerator = {
  __typename?: 'BoardModerator';
  boardId: Scalars['ID']['output'];
  createdAt: Scalars['String']['output'];
  id: Scalars['ID']['output'];
  isInviteAccepted: Scalars['Boolean']['output'];
  permissions: Scalars['Int']['output'];
  rank: Scalars['Int']['output'];
  user: User;
};

export type BoardReactionSettings = {
  __typename?: 'BoardReactionSettings';
  boardId: Scalars['ID']['output'];
  emojiWeights: Scalars['JSON']['output'];
  id: Scalars['ID']['output'];
  reactionsEnabled: Scalars['Boolean']['output'];
};

export type BoardSettings = {
  __typename?: 'BoardSettings';
  board: Board;
  isOwner: Scalars['Boolean']['output'];
  moderatorPermissions?: Maybe<Scalars['Int']['output']>;
};

export type BoardUnbanResponse = {
  __typename?: 'BoardUnbanResponse';
  message: Scalars['String']['output'];
  success: Scalars['Boolean']['output'];
};

export type Comment = {
  __typename?: 'Comment';
  approvalStatus: Scalars['String']['output'];
  board?: Maybe<Board>;
  boardId: Scalars['ID']['output'];
  body: Scalars['String']['output'];
  bodyHTML: Scalars['String']['output'];
  createdAt: Scalars['String']['output'];
  creator?: Maybe<User>;
  creatorId: Scalars['ID']['output'];
  distinguishedAs?: Maybe<Scalars['String']['output']>;
  downvotes: Scalars['Int']['output'];
  id: Scalars['ID']['output'];
  isDeleted: Scalars['Boolean']['output'];
  isLocked: Scalars['Boolean']['output'];
  isPinned: Scalars['Boolean']['output'];
  isRemoved: Scalars['Boolean']['output'];
  isSaved: Scalars['Boolean']['output'];
  level: Scalars['Int']['output'];
  myReaction?: Maybe<Reaction>;
  myVote: Scalars['Int']['output'];
  parentId?: Maybe<Scalars['String']['output']>;
  post: Post;
  postId: Scalars['ID']['output'];
  quotedCommentId?: Maybe<Scalars['ID']['output']>;
  reactionCounts?: Maybe<Array<ReactionAggregate>>;
  replies?: Maybe<Array<Comment>>;
  replyCount: Scalars['Int']['output'];
  score: Scalars['Int']['output'];
  slug: Scalars['String']['output'];
  updatedAt: Scalars['String']['output'];
  upvotes: Scalars['Int']['output'];
};

export type CommentReportView = {
  __typename?: 'CommentReportView';
  commentId: Scalars['ID']['output'];
  createdAt: Scalars['String']['output'];
  creatorId: Scalars['ID']['output'];
  id: Scalars['ID']['output'];
  originalCommentText: Scalars['String']['output'];
  reason: Scalars['String']['output'];
  resolverId?: Maybe<Scalars['ID']['output']>;
  status: Scalars['String']['output'];
  updatedAt: Scalars['String']['output'];
};

export type CommentSortType =
  | 'hot'
  | 'new'
  | 'old'
  | 'top';

export type Conversation = {
  __typename?: 'Conversation';
  lastActivity: Scalars['String']['output'];
  lastMessage: PrivateMessage;
  otherUser: User;
  unreadCount: Scalars['Int']['output'];
};

export type CreateBoardInput = {
  banner?: InputMaybe<Scalars['String']['input']>;
  description?: InputMaybe<Scalars['String']['input']>;
  hoverColor?: InputMaybe<Scalars['String']['input']>;
  icon?: InputMaybe<Scalars['String']['input']>;
  isNsfw?: InputMaybe<Scalars['Boolean']['input']>;
  name: Scalars['String']['input'];
  primaryColor?: InputMaybe<Scalars['String']['input']>;
  secondaryColor?: InputMaybe<Scalars['String']['input']>;
  title: Scalars['String']['input'];
  mode?: InputMaybe<Scalars['String']['input']>;
  wikiEnabled?: InputMaybe<Scalars['Boolean']['input']>;
};

export type CreateBoardResponse = {
  __typename?: 'CreateBoardResponse';
  board: Board;
};

export type CreateEmojiInput = {
  altText: Scalars['String']['input'];
  boardId?: InputMaybe<Scalars['ID']['input']>;
  category: Scalars['String']['input'];
  imageUrl: Scalars['String']['input'];
  keywords?: InputMaybe<Array<Scalars['String']['input']>>;
  scope?: InputMaybe<EmojiScope>;
  shortcode: Scalars['String']['input'];
};

export type CreateFlairCategoryInput = {
  boardId: Scalars['ID']['input'];
  color?: InputMaybe<Scalars['String']['input']>;
  description?: InputMaybe<Scalars['String']['input']>;
  displayOrder?: InputMaybe<Scalars['Int']['input']>;
  name: Scalars['String']['input'];
};

export type CreateFlairTemplateInput = {
  backgroundColor?: InputMaybe<Scalars['String']['input']>;
  boardId: Scalars['ID']['input'];
  categoryId?: InputMaybe<Scalars['ID']['input']>;
  displayOrder?: InputMaybe<Scalars['Int']['input']>;
  emojiIds?: InputMaybe<Array<Scalars['Int']['input']>>;
  flairType: FlairType;
  isEditable?: InputMaybe<Scalars['Boolean']['input']>;
  isModOnly?: InputMaybe<Scalars['Boolean']['input']>;
  isRequiresApproval?: InputMaybe<Scalars['Boolean']['input']>;
  maxEmojiCount?: InputMaybe<Scalars['Int']['input']>;
  maxTextLength?: InputMaybe<Scalars['Int']['input']>;
  styleConfig?: InputMaybe<FlairStyleInput>;
  templateName: Scalars['String']['input'];
  textColor?: InputMaybe<Scalars['String']['input']>;
  textDisplay?: InputMaybe<Scalars['String']['input']>;
};

export type CreateWikiPageInput = {
  body: Scalars['String']['input'];
  displayOrder?: InputMaybe<Scalars['Int']['input']>;
  editPermission?: InputMaybe<Scalars['String']['input']>;
  parentId?: InputMaybe<Scalars['ID']['input']>;
  slug: Scalars['String']['input'];
  title: Scalars['String']['input'];
  viewPermission?: InputMaybe<Scalars['String']['input']>;
};

export type DeleteNotificationResponse = {
  __typename?: 'DeleteNotificationResponse';
  success: Scalars['Boolean']['output'];
};

export type EditMessageInput = {
  body?: InputMaybe<Scalars['String']['input']>;
  messageId: Scalars['ID']['input'];
  subject?: InputMaybe<Scalars['String']['input']>;
};

export type EditMessageResponse = {
  __typename?: 'EditMessageResponse';
  message: PrivateMessage;
};

export type EditWikiPageInput = {
  body?: InputMaybe<Scalars['String']['input']>;
  displayOrder?: InputMaybe<Scalars['Int']['input']>;
  editPermission?: InputMaybe<Scalars['String']['input']>;
  editSummary?: InputMaybe<Scalars['String']['input']>;
  isLocked?: InputMaybe<Scalars['Boolean']['input']>;
  title?: InputMaybe<Scalars['String']['input']>;
  viewPermission?: InputMaybe<Scalars['String']['input']>;
};

export type EmojiObject = {
  __typename?: 'EmojiObject';
  altText: Scalars['String']['output'];
  boardId?: Maybe<Scalars['ID']['output']>;
  category: Scalars['String']['output'];
  createdAt: Scalars['String']['output'];
  createdBy: Scalars['ID']['output'];
  id: Scalars['ID']['output'];
  imageUrl: Scalars['String']['output'];
  isActive: Scalars['Boolean']['output'];
  scope: Scalars['String']['output'];
  shortcode: Scalars['String']['output'];
  updatedAt: Scalars['String']['output'];
  usageCount: Scalars['Int']['output'];
};

export type EmojiScope =
  | 'Board'
  | 'Site';

export type FilterMode =
  | 'exclude'
  | 'include';

export type FlairCategory = {
  __typename?: 'FlairCategory';
  boardId: Scalars['ID']['output'];
  color?: Maybe<Scalars['String']['output']>;
  createdAt: Scalars['String']['output'];
  createdBy: Scalars['ID']['output'];
  description?: Maybe<Scalars['String']['output']>;
  displayOrder: Scalars['Int']['output'];
  id: Scalars['ID']['output'];
  name: Scalars['String']['output'];
  updatedAt: Scalars['String']['output'];
};

export type FlairStyle = {
  __typename?: 'FlairStyle';
  animationDuration?: Maybe<Scalars['Int']['output']>;
  animationType?: Maybe<Scalars['String']['output']>;
  backgroundColor?: Maybe<Scalars['String']['output']>;
  borderColor?: Maybe<Scalars['String']['output']>;
  borderRadius?: Maybe<Scalars['Int']['output']>;
  borderStyle?: Maybe<Scalars['String']['output']>;
  borderWidth?: Maybe<Scalars['Int']['output']>;
  customCss?: Maybe<Scalars['String']['output']>;
  fontSize?: Maybe<Scalars['String']['output']>;
  fontWeight?: Maybe<Scalars['String']['output']>;
  gradientDirection?: Maybe<Scalars['String']['output']>;
  gradientEnd?: Maybe<Scalars['String']['output']>;
  gradientStart?: Maybe<Scalars['String']['output']>;
  iconUrl?: Maybe<Scalars['String']['output']>;
  margin?: Maybe<Scalars['String']['output']>;
  padding?: Maybe<Scalars['String']['output']>;
  shadowBlur?: Maybe<Scalars['Int']['output']>;
  shadowColor?: Maybe<Scalars['String']['output']>;
  shadowOffsetX?: Maybe<Scalars['Int']['output']>;
  shadowOffsetY?: Maybe<Scalars['Int']['output']>;
  textColor?: Maybe<Scalars['String']['output']>;
};

export type FlairStyleInput = {
  animationDuration?: InputMaybe<Scalars['Int']['input']>;
  animationType?: InputMaybe<Scalars['String']['input']>;
  backgroundColor?: InputMaybe<Scalars['String']['input']>;
  borderColor?: InputMaybe<Scalars['String']['input']>;
  borderRadius?: InputMaybe<Scalars['Int']['input']>;
  borderStyle?: InputMaybe<Scalars['String']['input']>;
  borderWidth?: InputMaybe<Scalars['Int']['input']>;
  customCss?: InputMaybe<Scalars['String']['input']>;
  fontSize?: InputMaybe<Scalars['String']['input']>;
  fontWeight?: InputMaybe<Scalars['String']['input']>;
  gradientDirection?: InputMaybe<Scalars['String']['input']>;
  gradientEnd?: InputMaybe<Scalars['String']['input']>;
  gradientStart?: InputMaybe<Scalars['String']['input']>;
  margin?: InputMaybe<Scalars['String']['input']>;
  padding?: InputMaybe<Scalars['String']['input']>;
  shadowBlur?: InputMaybe<Scalars['Int']['input']>;
  shadowColor?: InputMaybe<Scalars['String']['input']>;
  shadowOffsetX?: InputMaybe<Scalars['Int']['input']>;
  shadowOffsetY?: InputMaybe<Scalars['Int']['input']>;
  textColor?: InputMaybe<Scalars['String']['input']>;
};

export type FlairTemplate = {
  __typename?: 'FlairTemplate';
  backgroundColor: Scalars['String']['output'];
  boardId: Scalars['ID']['output'];
  categoryId?: Maybe<Scalars['ID']['output']>;
  createdAt: Scalars['String']['output'];
  createdBy: Scalars['ID']['output'];
  displayOrder: Scalars['Int']['output'];
  emojiIds: Array<Maybe<Scalars['Int']['output']>>;
  flairType: Scalars['String']['output'];
  id: Scalars['ID']['output'];
  isActive: Scalars['Boolean']['output'];
  isEditable: Scalars['Boolean']['output'];
  isModOnly: Scalars['Boolean']['output'];
  isRequiresApproval: Scalars['Boolean']['output'];
  maxEmojiCount: Scalars['Int']['output'];
  maxTextLength: Scalars['Int']['output'];
  styleConfig: Scalars['String']['output'];
  templateKey?: Maybe<Scalars['String']['output']>;
  templateName: Scalars['String']['output'];
  textColor: Scalars['String']['output'];
  textDisplay: Scalars['String']['output'];
  updatedAt: Scalars['String']['output'];
  usageCount: Scalars['Int']['output'];
};

export type FlairType =
  | 'Post'
  | 'User';

export type ListEmojisInput = {
  boardId?: InputMaybe<Scalars['ID']['input']>;
  category?: InputMaybe<Scalars['String']['input']>;
  limit?: InputMaybe<Scalars['Int']['input']>;
  offset?: InputMaybe<Scalars['Int']['input']>;
  scope?: InputMaybe<EmojiScope>;
  search?: InputMaybe<Scalars['String']['input']>;
};

export type ListingType =
  | 'all'
  | 'local'
  | 'moderated'
  | 'subscribed';

export type LocalSite = {
  __typename?: 'LocalSite';
  allowedPostTypes?: Maybe<Scalars['String']['output']>;
  applicationQuestion?: Maybe<Scalars['String']['output']>;
  boardCreationAdminOnly: Scalars['Boolean']['output'];
  boardCreationMode: Scalars['String']['output'];
  boardsEnabled: Scalars['Boolean']['output'];
  captchaDifficulty: Scalars['String']['output'];
  captchaEnabled: Scalars['Boolean']['output'];
  createdAt: Scalars['String']['output'];
  defaultAvatar?: Maybe<Scalars['String']['output']>;
  defaultPostListingType: Scalars['String']['output'];
  defaultTheme: Scalars['String']['output'];
  description?: Maybe<Scalars['String']['output']>;
  enableDownvotes: Scalars['Boolean']['output'];
  enableNSFW: Scalars['Boolean']['output'];
  enableNSFWTagging: Scalars['Boolean']['output'];
  filteredWords?: Maybe<Scalars['String']['output']>;
  hideModlogModNames: Scalars['Boolean']['output'];
  homepageBanner?: Maybe<Scalars['String']['output']>;
  hoverColor: Scalars['String']['output'];
  icon?: Maybe<Scalars['String']['output']>;
  id: Scalars['ID']['output'];
  isPrivate: Scalars['Boolean']['output'];
  isSiteSetup: Scalars['Boolean']['output'];
  legalInformation?: Maybe<Scalars['String']['output']>;
  name: Scalars['String']['output'];
  primaryColor: Scalars['String']['output'];
  registrationMode: Scalars['String']['output'];
  requireEmailVerification: Scalars['Boolean']['output'];
  secondaryColor: Scalars['String']['output'];
  updatedAt: Scalars['String']['output'];
  welcomeMessage?: Maybe<Scalars['String']['output']>;
  wordFilterEnabled: Scalars['Boolean']['output'];
  defaultBoardMode: Scalars['String']['output'];
  customCss?: Maybe<Scalars['String']['output']>;
  customCssEnabled: Scalars['Boolean']['output'];
};

export type MarkNotificationsReadResponse = {
  __typename?: 'MarkNotificationsReadResponse';
  markedCount: Scalars['Int']['output'];
  success: Scalars['Boolean']['output'];
};

export type MeResponse = {
  __typename?: 'MeResponse';
  unreadNotificationsCount: Scalars['Int']['output'];
  user: User;
};

export type ModerationLogEntry = {
  __typename?: 'ModerationLogEntry';
  actionType: Scalars['String']['output'];
  boardId?: Maybe<Scalars['ID']['output']>;
  createdAt: Scalars['String']['output'];
  expiresAt?: Maybe<Scalars['String']['output']>;
  id: Scalars['ID']['output'];
  metadata?: Maybe<Scalars['String']['output']>;
  moderatorId: Scalars['ID']['output'];
  moderatorName: Scalars['String']['output'];
  reason?: Maybe<Scalars['String']['output']>;
  targetId: Scalars['ID']['output'];
  targetType: Scalars['String']['output'];
};

export type ModerationLogResponse = {
  __typename?: 'ModerationLogResponse';
  entries: Array<ModerationLogEntry>;
  totalCount: Scalars['Int']['output'];
};

export type Mutation = {
  __typename?: 'Mutation';
  addModerator: AddModeratorResponse;
  addReaction: AddReactionResponse;
  adminAddSelfAsMod: Board;
  adminBanBoard: Board;
  adminBannedBoards: Array<Board>;
  adminRemoveSelfAsMod: Board;
  adminUnbanBoard: Board;
  approveApplication: Scalars['Boolean']['output'];
  approveUserFlair: UserFlair;
  assignPostFlair: PostFlair;
  assignUserFlair: UserFlair;
  banUserFromBoard: BoardBanResponse;
  banUserFromSite: BanUserResponse;
  blockBoard: Scalars['Boolean']['output'];
  blockUser: Scalars['Boolean']['output'];
  createBoard: CreateBoardResponse;
  createComment: Comment;
  createEmoji: EmojiObject;
  createFlairCategory: FlairCategory;
  createFlairTemplate: FlairTemplate;
  createInvite: Scalars['String']['output'];
  createPost: Post;
  createWikiPage: WikiPage;
  deleteAccount: Scalars['Boolean']['output'];
  deleteEmoji: Scalars['Boolean']['output'];
  deleteFlairCategory: Scalars['Boolean']['output'];
  deleteFlairTemplate: Scalars['Boolean']['output'];
  deleteInvite: Scalars['Boolean']['output'];
  deleteMessage: Scalars['Boolean']['output'];
  deleteNotification: DeleteNotificationResponse;
  deleteWikiPage: Scalars['Boolean']['output'];
  denyApplication: Scalars['Boolean']['output'];
  dismissReport: ResolveReportResponse;
  editComment: Comment;
  editMessage: EditMessageResponse;
  editPost: Post;
  editWikiPage: WikiPage;
  excludeBoardFromAll: Board;
  featurePost: Post;
  followUser: Scalars['Boolean']['output'];
  hidePost: Post;
  lockPost: Post;
  markAllNotificationsAsRead: MarkNotificationsReadResponse;
  markNotificationsRead: MarkNotificationsReadResponse;
  pinComment: Comment;
  removeBoardModerator: RemoveModeratorResponse;
  removeComment: Comment;
  removePost: Post;
  removePostFlair: Scalars['Boolean']['output'];
  removeReaction: RemoveReactionResponse;
  removeUserFlair: Scalars['Boolean']['output'];
  reportComment: ReportResponse;
  reportPost: ReportResponse;
  resolveReport: ResolveReportResponse;
  restoreComment: Comment;
  restorePost: Post;
  revertWikiPage: WikiPage;
  saveComment: Comment;
  savePost: Post;
  sendMessage: SendMessageResponse;
  subscribeToBoard: Scalars['Boolean']['output'];
  transferBoardOwnership: TransferOwnershipResponse;
  unbanUserFromBoard: BoardUnbanResponse;
  unbanUserFromSite: UnbanUserResponse;
  unblockBoard: Scalars['Boolean']['output'];
  unblockUser: Scalars['Boolean']['output'];
  unfollowUser: Scalars['Boolean']['output'];
  unhidePost: Post;
  unlockPost: Post;
  unsaveComment: Comment;
  unsavePost: Post;
  unsubscribeFromBoard: Scalars['Boolean']['output'];
  updateBoardReactionSettings: UpdateBoardReactionSettingsResponse;
  updateBoardSettings: UpdateBoardSettingsResponse;
  updateEmoji: EmojiObject;
  updateFlairCategory: FlairCategory;
  updateFlairTemplate: FlairTemplate;
  updateNotificationSettings: UpdateNotificationSettingsResponse;
  updateProfile: User;
  updateSettings: UserSettings;
  updateSiteConfig: LocalSite;
  updateUserBoardCreationApproval: User;
  uploadFile: Scalars['String']['output'];
  voteOnComment: Comment;
  voteOnPost: Post;
};


export type MutationAddModeratorArgs = {
  boardId: Scalars['ID']['input'];
  permissions?: InputMaybe<Scalars['Int']['input']>;
  userId: Scalars['ID']['input'];
};


export type MutationAddReactionArgs = {
  input: AddReactionInput;
};


export type MutationAdminAddSelfAsModArgs = {
  boardId: Scalars['ID']['input'];
  modPerms?: InputMaybe<Scalars['Int']['input']>;
};


export type MutationAdminBanBoardArgs = {
  adminNotes?: InputMaybe<Scalars['String']['input']>;
  boardId: Scalars['ID']['input'];
  publicReason: Scalars['String']['input'];
};


export type MutationAdminRemoveSelfAsModArgs = {
  boardId: Scalars['ID']['input'];
};


export type MutationAdminUnbanBoardArgs = {
  boardId: Scalars['ID']['input'];
};


export type MutationApproveApplicationArgs = {
  applicationId: Scalars['ID']['input'];
};


export type MutationApproveUserFlairArgs = {
  approved: Scalars['Boolean']['input'];
  flairId: Scalars['ID']['input'];
};


export type MutationAssignPostFlairArgs = {
  input: AssignPostFlairInput;
};


export type MutationAssignUserFlairArgs = {
  input: AssignUserFlairInput;
};


export type MutationBanUserFromBoardArgs = {
  input: BoardBanUserInput;
};


export type MutationBanUserFromSiteArgs = {
  input: BanUserInput;
};


export type MutationBlockBoardArgs = {
  boardId: Scalars['ID']['input'];
};


export type MutationBlockUserArgs = {
  userId: Scalars['ID']['input'];
};


export type MutationCreateBoardArgs = {
  bannerFile?: InputMaybe<Scalars['Upload']['input']>;
  iconFile?: InputMaybe<Scalars['Upload']['input']>;
  input: CreateBoardInput;
};


export type MutationCreateCommentArgs = {
  body: Scalars['String']['input'];
  parentId?: InputMaybe<Scalars['ID']['input']>;
  postId: Scalars['ID']['input'];
};


export type MutationCreateEmojiArgs = {
  input: CreateEmojiInput;
};


export type MutationCreateFlairCategoryArgs = {
  input: CreateFlairCategoryInput;
};


export type MutationCreateFlairTemplateArgs = {
  input: CreateFlairTemplateInput;
};


export type MutationCreatePostArgs = {
  altText?: InputMaybe<Scalars['String']['input']>;
  board?: InputMaybe<Scalars['String']['input']>;
  body?: InputMaybe<Scalars['String']['input']>;
  file?: InputMaybe<Scalars['Upload']['input']>;
  isNSFW?: InputMaybe<Scalars['Boolean']['input']>;
  link?: InputMaybe<Scalars['String']['input']>;
  postType?: InputMaybe<Scalars['String']['input']>;
  title: Scalars['String']['input'];
};


export type MutationCreateWikiPageArgs = {
  boardId: Scalars['ID']['input'];
  input: CreateWikiPageInput;
};


export type MutationDeleteEmojiArgs = {
  emojiId: Scalars['ID']['input'];
};


export type MutationDeleteFlairCategoryArgs = {
  categoryId: Scalars['ID']['input'];
};


export type MutationDeleteFlairTemplateArgs = {
  templateId: Scalars['ID']['input'];
};


export type MutationDeleteInviteArgs = {
  inviteId: Scalars['ID']['input'];
};


export type MutationDeleteMessageArgs = {
  messageId: Scalars['ID']['input'];
};


export type MutationDeleteNotificationArgs = {
  notificationId: Scalars['ID']['input'];
};


export type MutationDeleteWikiPageArgs = {
  pageId: Scalars['ID']['input'];
};


export type MutationDenyApplicationArgs = {
  applicationId: Scalars['ID']['input'];
  reason?: InputMaybe<Scalars['String']['input']>;
};


export type MutationDismissReportArgs = {
  reportId: Scalars['ID']['input'];
  reportType: Scalars['String']['input'];
};


export type MutationEditCommentArgs = {
  body: Scalars['String']['input'];
  id: Scalars['ID']['input'];
};


export type MutationEditMessageArgs = {
  input: EditMessageInput;
};


export type MutationEditPostArgs = {
  altText?: InputMaybe<Scalars['String']['input']>;
  body: Scalars['String']['input'];
  id: Scalars['ID']['input'];
};


export type MutationEditWikiPageArgs = {
  input: EditWikiPageInput;
  pageId: Scalars['ID']['input'];
};


export type MutationExcludeBoardFromAllArgs = {
  boardId: Scalars['ID']['input'];
  exclude: Scalars['Boolean']['input'];
};


export type MutationFeaturePostArgs = {
  featureType?: InputMaybe<Scalars['String']['input']>;
  featured: Scalars['Boolean']['input'];
  postId: Scalars['ID']['input'];
};


export type MutationFollowUserArgs = {
  userId: Scalars['ID']['input'];
};


export type MutationHidePostArgs = {
  postId: Scalars['ID']['input'];
};


export type MutationLockPostArgs = {
  postId: Scalars['ID']['input'];
};


export type MutationMarkNotificationsReadArgs = {
  notificationIds: Array<Scalars['ID']['input']>;
};


export type MutationPinCommentArgs = {
  commentId: Scalars['ID']['input'];
};


export type MutationRemoveBoardModeratorArgs = {
  boardId: Scalars['ID']['input'];
  userId: Scalars['ID']['input'];
};


export type MutationRemoveCommentArgs = {
  commentId: Scalars['ID']['input'];
  reason?: InputMaybe<Scalars['String']['input']>;
};


export type MutationRemovePostArgs = {
  postId: Scalars['ID']['input'];
  reason?: InputMaybe<Scalars['String']['input']>;
};


export type MutationRemovePostFlairArgs = {
  postId: Scalars['ID']['input'];
};


export type MutationRemoveReactionArgs = {
  input: RemoveReactionInput;
};


export type MutationRemoveUserFlairArgs = {
  boardId: Scalars['ID']['input'];
  userId: Scalars['ID']['input'];
};


export type MutationReportCommentArgs = {
  commentId: Scalars['ID']['input'];
  reason: Scalars['String']['input'];
};


export type MutationReportPostArgs = {
  postId: Scalars['ID']['input'];
  reason: Scalars['String']['input'];
};


export type MutationResolveReportArgs = {
  reportId: Scalars['ID']['input'];
  reportType: Scalars['String']['input'];
};


export type MutationRestoreCommentArgs = {
  commentId: Scalars['ID']['input'];
};


export type MutationRestorePostArgs = {
  postId: Scalars['ID']['input'];
};


export type MutationRevertWikiPageArgs = {
  pageId: Scalars['ID']['input'];
  revisionId: Scalars['ID']['input'];
};


export type MutationSaveCommentArgs = {
  commentId: Scalars['ID']['input'];
};


export type MutationSavePostArgs = {
  postId: Scalars['ID']['input'];
};


export type MutationSendMessageArgs = {
  input: SendMessageInput;
};


export type MutationSubscribeToBoardArgs = {
  boardId: Scalars['ID']['input'];
};


export type MutationTransferBoardOwnershipArgs = {
  boardId: Scalars['ID']['input'];
  newOwnerId: Scalars['ID']['input'];
};


export type MutationUnbanUserFromBoardArgs = {
  boardId: Scalars['ID']['input'];
  userId: Scalars['ID']['input'];
};


export type MutationUnbanUserFromSiteArgs = {
  reason?: InputMaybe<Scalars['String']['input']>;
  userId: Scalars['ID']['input'];
};


export type MutationUnblockBoardArgs = {
  boardId: Scalars['ID']['input'];
};


export type MutationUnblockUserArgs = {
  userId: Scalars['ID']['input'];
};


export type MutationUnfollowUserArgs = {
  userId: Scalars['ID']['input'];
};


export type MutationUnhidePostArgs = {
  postId: Scalars['ID']['input'];
};


export type MutationUnlockPostArgs = {
  postId: Scalars['ID']['input'];
};


export type MutationUnsaveCommentArgs = {
  commentId: Scalars['ID']['input'];
};


export type MutationUnsavePostArgs = {
  postId: Scalars['ID']['input'];
};


export type MutationUnsubscribeFromBoardArgs = {
  boardId: Scalars['ID']['input'];
};


export type MutationUpdateBoardReactionSettingsArgs = {
  input: UpdateBoardReactionSettingsInput;
};


export type MutationUpdateBoardSettingsArgs = {
  bannerFile?: InputMaybe<Scalars['Upload']['input']>;
  iconFile?: InputMaybe<Scalars['Upload']['input']>;
  input: UpdateBoardSettingsInput;
};


export type MutationUpdateEmojiArgs = {
  emojiId: Scalars['ID']['input'];
  input: UpdateEmojiInput;
};


export type MutationUpdateFlairCategoryArgs = {
  categoryId: Scalars['ID']['input'];
  input: UpdateFlairCategoryInput;
};


export type MutationUpdateFlairTemplateArgs = {
  input: UpdateFlairTemplateInput;
  templateId: Scalars['ID']['input'];
};


export type MutationUpdateNotificationSettingsArgs = {
  input: UpdateNotificationSettingsInput;
};


export type MutationUpdateProfileArgs = {
  input: UpdateProfileInput;
};


export type MutationUpdateSettingsArgs = {
  input: UpdateSettingsInput;
};


export type MutationUpdateSiteConfigArgs = {
  input: UpdateSiteConfigInput;
};


export type MutationUpdateUserBoardCreationApprovalArgs = {
  approved: Scalars['Boolean']['input'];
  userId: Scalars['ID']['input'];
};


export type MutationUploadFileArgs = {
  file: Scalars['Upload']['input'];
};


export type MutationVoteOnCommentArgs = {
  commentId: Scalars['ID']['input'];
  direction: Scalars['Int']['input'];
};


export type MutationVoteOnPostArgs = {
  direction: Scalars['Int']['input'];
  postId: Scalars['ID']['input'];
};

export type Notification = {
  __typename?: 'Notification';
  actor?: Maybe<NotificationActor>;
  comment?: Maybe<NotificationCommentContext>;
  commentId?: Maybe<Scalars['ID']['output']>;
  createdAt: Scalars['String']['output'];
  id: Scalars['ID']['output'];
  isRead: Scalars['Boolean']['output'];
  message?: Maybe<NotificationMessageContext>;
  messageId?: Maybe<Scalars['ID']['output']>;
  post?: Maybe<NotificationPostContext>;
  postId?: Maybe<Scalars['ID']['output']>;
  type: Scalars['String']['output'];
};

export type NotificationActor = {
  __typename?: 'NotificationActor';
  avatar?: Maybe<Scalars['String']['output']>;
  displayName?: Maybe<Scalars['String']['output']>;
  id: Scalars['ID']['output'];
  name: Scalars['String']['output'];
};

export type NotificationCommentContext = {
  __typename?: 'NotificationCommentContext';
  boardName: Scalars['String']['output'];
  body: Scalars['String']['output'];
  id: Scalars['ID']['output'];
  postId: Scalars['ID']['output'];
  postTitle: Scalars['String']['output'];
};

export type NotificationMessageContext = {
  __typename?: 'NotificationMessageContext';
  body: Scalars['String']['output'];
  id: Scalars['ID']['output'];
};

export type NotificationPostContext = {
  __typename?: 'NotificationPostContext';
  boardId: Scalars['ID']['output'];
  boardName: Scalars['String']['output'];
  id: Scalars['ID']['output'];
  title: Scalars['String']['output'];
};

export type NotificationSettings = {
  __typename?: 'NotificationSettings';
  boardInvitesEnabled: Scalars['Boolean']['output'];
  commentRepliesEnabled: Scalars['Boolean']['output'];
  emailEnabled: Scalars['Boolean']['output'];
  mentionsEnabled: Scalars['Boolean']['output'];
  moderatorActionsEnabled: Scalars['Boolean']['output'];
  postRepliesEnabled: Scalars['Boolean']['output'];
  privateMessagesEnabled: Scalars['Boolean']['output'];
  systemNotificationsEnabled: Scalars['Boolean']['output'];
};

export type Post = {
  __typename?: 'Post';
  altText?: Maybe<Scalars['String']['output']>;
  approvalStatus: Scalars['String']['output'];
  board?: Maybe<Board>;
  boardId: Scalars['ID']['output'];
  body: Scalars['String']['output'];
  bodyHTML: Scalars['String']['output'];
  commentCount: Scalars['Int']['output'];
  controversyRank: Scalars['Float']['output'];
  distinguishedAs?: Maybe<Scalars['String']['output']>;
  createdAt: Scalars['String']['output'];
  creator?: Maybe<User>;
  creatorId: Scalars['ID']['output'];
  downvotes: Scalars['Int']['output'];
  embedDescription?: Maybe<Scalars['String']['output']>;
  embedTitle?: Maybe<Scalars['String']['output']>;
  embedVideoUrl?: Maybe<Scalars['String']['output']>;
  flairs?: Maybe<Array<PostFlair>>;
  hotRank: Scalars['Int']['output'];
  hotRankActive: Scalars['Int']['output'];
  id: Scalars['ID']['output'];
  image?: Maybe<Scalars['String']['output']>;
  isDeleted: Scalars['Boolean']['output'];
  isFeaturedBoard: Scalars['Boolean']['output'];
  isFeaturedLocal: Scalars['Boolean']['output'];
  isLocked: Scalars['Boolean']['output'];
  isNSFW: Scalars['Boolean']['output'];
  isRemoved: Scalars['Boolean']['output'];
  isSaved: Scalars['Boolean']['output'];
  isThread: Scalars['Boolean']['output'];
  lastCrawlDate?: Maybe<Scalars['String']['output']>;
  myModPermissions?: Maybe<Scalars['Int']['output']>;
  myReaction?: Maybe<Reaction>;
  myVote: Scalars['Int']['output'];
  newestCommentTime: Scalars['String']['output'];
  postType: Scalars['String']['output'];
  reactionCounts?: Maybe<Array<ReactionAggregate>>;
  score: Scalars['Int']['output'];
  slug: Scalars['String']['output'];
  sourceUrl?: Maybe<Scalars['String']['output']>;
  thumbnailUrl?: Maybe<Scalars['String']['output']>;
  title: Scalars['String']['output'];
  updatedAt: Scalars['String']['output'];
  upvotes: Scalars['Int']['output'];
  url?: Maybe<Scalars['String']['output']>;
  urlPath: Scalars['String']['output'];
};

export type PostFlair = {
  __typename?: 'PostFlair';
  customBackgroundColor?: Maybe<Scalars['String']['output']>;
  customText?: Maybe<Scalars['String']['output']>;
  customTextColor?: Maybe<Scalars['String']['output']>;
  flairTemplateId: Scalars['ID']['output'];
  id: Scalars['ID']['output'];
  postId: Scalars['ID']['output'];
};

export type PostReportView = {
  __typename?: 'PostReportView';
  createdAt: Scalars['String']['output'];
  creatorId: Scalars['ID']['output'];
  id: Scalars['ID']['output'];
  originalPostBody?: Maybe<Scalars['String']['output']>;
  originalPostTitle: Scalars['String']['output'];
  originalPostUrl?: Maybe<Scalars['String']['output']>;
  postId: Scalars['ID']['output'];
  reason: Scalars['String']['output'];
  resolverId?: Maybe<Scalars['ID']['output']>;
  status: Scalars['String']['output'];
  updatedAt: Scalars['String']['output'];
};

export type PrivateMessage = {
  __typename?: 'PrivateMessage';
  body: Scalars['String']['output'];
  bodyHTML: Scalars['String']['output'];
  createdAt: Scalars['String']['output'];
  creator?: Maybe<User>;
  creatorId: Scalars['ID']['output'];
  id: Scalars['ID']['output'];
  isDeleted: Scalars['Boolean']['output'];
  isRead: Scalars['Boolean']['output'];
  isSenderHidden: Scalars['Boolean']['output'];
  recipient?: Maybe<User>;
  recipientId?: Maybe<Scalars['ID']['output']>;
  subject?: Maybe<Scalars['String']['output']>;
  updatedAt: Scalars['String']['output'];
};

export type Query = {
  __typename?: 'Query';
  board: Board;
  boardFlairs: Array<FlairTemplate>;
  comment: Comment;
  comments: Array<Comment>;
  flairTemplate?: Maybe<FlairTemplate>;
  getAllEmojisAdmin: Array<EmojiObject>;
  getBoardBannedUsers: Array<BoardBannedUser>;
  getBoardModerators: Array<BoardModerator>;
  getBoardSettings: BoardSettings;
  getCommentReports: Array<CommentReportView>;
  getConversation: Array<PrivateMessage>;
  getModeratedBoards: Array<Board>;
  getModerationLog: ModerationLogResponse;
  getNotificationSettings: NotificationSettings;
  getNotifications: Array<Notification>;
  getPostReports: Array<PostReportView>;
  getUnreadMessageCount: Scalars['Int']['output'];
  getUnreadNotificationCount: UnreadNotificationCount;
  getUserSettings: UserSettings;
  isFollowingUser: Scalars['Boolean']['output'];
  listBannedUsers: BannedUsersResponse;
  listBoards: Array<Board>;
  listConversations: Array<Conversation>;
  listEmojis: Array<EmojiObject>;
  listInvites: Array<SiteInviteGql>;
  listPosts: Array<Post>;
  listRegistrationApplications: Array<RegistrationApplication>;
  listUsers: Array<User>;
  listWikiPages: Array<WikiPage>;
  manageBoardFlairs: Array<FlairTemplate>;
  me: MeResponse;
  post: Post;
  searchContent: SearchResult;
  searchUsernames: Array<Scalars['String']['output']>;
  site: LocalSite;
  siteStats: SiteStats;
  user: User;
  userFollowers: Array<User>;
  userFollowing: Array<User>;
  wikiPage?: Maybe<WikiPage>;
  wikiPageHistory: Array<WikiRevision>;
};


export type QueryBoardArgs = {
  name: Scalars['String']['input'];
};


export type QueryBoardFlairsArgs = {
  activeOnly?: InputMaybe<Scalars['Boolean']['input']>;
  boardId: Scalars['ID']['input'];
  flairType?: InputMaybe<FlairType>;
};


export type QueryCommentArgs = {
  id: Scalars['ID']['input'];
};


export type QueryCommentsArgs = {
  boardId?: InputMaybe<Scalars['ID']['input']>;
  boardName?: InputMaybe<Scalars['String']['input']>;
  includeRemoved?: InputMaybe<Scalars['Boolean']['input']>;
  limit?: InputMaybe<Scalars['Int']['input']>;
  page?: InputMaybe<Scalars['Int']['input']>;
  postId?: InputMaybe<Scalars['ID']['input']>;
  removedOnly?: InputMaybe<Scalars['Boolean']['input']>;
  sort?: InputMaybe<CommentSortType>;
  userId?: InputMaybe<Scalars['ID']['input']>;
  userName?: InputMaybe<Scalars['String']['input']>;
};


export type QueryFlairTemplateArgs = {
  id: Scalars['ID']['input'];
};


export type QueryGetAllEmojisAdminArgs = {
  boardId?: InputMaybe<Scalars['ID']['input']>;
};


export type QueryGetBoardBannedUsersArgs = {
  boardId: Scalars['ID']['input'];
  limit?: InputMaybe<Scalars['Int']['input']>;
  page?: InputMaybe<Scalars['Int']['input']>;
};


export type QueryGetBoardModeratorsArgs = {
  boardId: Scalars['ID']['input'];
};


export type QueryGetBoardSettingsArgs = {
  boardId: Scalars['ID']['input'];
};


export type QueryGetCommentReportsArgs = {
  boardId?: InputMaybe<Scalars['ID']['input']>;
  limit?: InputMaybe<Scalars['Int']['input']>;
  offset?: InputMaybe<Scalars['Int']['input']>;
  statusFilter?: InputMaybe<Scalars['String']['input']>;
};


export type QueryGetConversationArgs = {
  limit?: InputMaybe<Scalars['Int']['input']>;
  offset?: InputMaybe<Scalars['Int']['input']>;
  userId: Scalars['ID']['input'];
};


export type QueryGetModeratedBoardsArgs = {
  limit?: InputMaybe<Scalars['Int']['input']>;
  page?: InputMaybe<Scalars['Int']['input']>;
};


export type QueryGetModerationLogArgs = {
  actionType?: InputMaybe<Scalars['String']['input']>;
  boardId?: InputMaybe<Scalars['ID']['input']>;
  limit?: InputMaybe<Scalars['Int']['input']>;
  moderatorId?: InputMaybe<Scalars['ID']['input']>;
  offset?: InputMaybe<Scalars['Int']['input']>;
};


export type QueryGetNotificationsArgs = {
  kindFilter?: InputMaybe<Scalars['String']['input']>;
  limit?: InputMaybe<Scalars['Int']['input']>;
  page?: InputMaybe<Scalars['Int']['input']>;
  unreadOnly?: InputMaybe<Scalars['Boolean']['input']>;
};


export type QueryGetPostReportsArgs = {
  boardId?: InputMaybe<Scalars['ID']['input']>;
  limit?: InputMaybe<Scalars['Int']['input']>;
  offset?: InputMaybe<Scalars['Int']['input']>;
  statusFilter?: InputMaybe<Scalars['String']['input']>;
};


export type QueryIsFollowingUserArgs = {
  userId: Scalars['ID']['input'];
};


export type QueryListBannedUsersArgs = {
  limit?: InputMaybe<Scalars['Int']['input']>;
  page?: InputMaybe<Scalars['Int']['input']>;
};


export type QueryListBoardsArgs = {
  bannedBoards?: InputMaybe<Scalars['Boolean']['input']>;
  limit?: InputMaybe<Scalars['Int']['input']>;
  listingType?: InputMaybe<ListingType>;
  page?: InputMaybe<Scalars['Int']['input']>;
  searchTerm?: InputMaybe<Scalars['String']['input']>;
  searchTitleAndDesc?: InputMaybe<Scalars['Boolean']['input']>;
  sort?: InputMaybe<SortType>;
};


export type QueryListEmojisArgs = {
  input?: InputMaybe<ListEmojisInput>;
};


export type QueryListPostsArgs = {
  boardId?: InputMaybe<Scalars['ID']['input']>;
  boardName?: InputMaybe<Scalars['String']['input']>;
  includeRemoved?: InputMaybe<Scalars['Boolean']['input']>;
  limit?: InputMaybe<Scalars['Int']['input']>;
  listingType?: InputMaybe<ListingType>;
  page?: InputMaybe<Scalars['Int']['input']>;
  removedOnly?: InputMaybe<Scalars['Boolean']['input']>;
  savedOnly?: InputMaybe<Scalars['Boolean']['input']>;
  sort?: InputMaybe<SortType>;
  userId?: InputMaybe<Scalars['ID']['input']>;
  userName?: InputMaybe<Scalars['String']['input']>;
};


export type QueryListRegistrationApplicationsArgs = {
  limit?: InputMaybe<Scalars['Int']['input']>;
  offset?: InputMaybe<Scalars['Int']['input']>;
};


export type QueryListUsersArgs = {
  limit?: InputMaybe<Scalars['Int']['input']>;
  page?: InputMaybe<Scalars['Int']['input']>;
  searchTerm?: InputMaybe<Scalars['String']['input']>;
};


export type QueryListWikiPagesArgs = {
  boardName: Scalars['String']['input'];
  includeDeleted?: InputMaybe<Scalars['Boolean']['input']>;
};


export type QueryManageBoardFlairsArgs = {
  boardId: Scalars['ID']['input'];
  flairType?: InputMaybe<FlairType>;
};


export type QueryPostArgs = {
  id: Scalars['ID']['input'];
};


export type QuerySearchContentArgs = {
  boardId?: InputMaybe<Scalars['ID']['input']>;
  creatorId?: InputMaybe<Scalars['ID']['input']>;
  limit?: InputMaybe<Scalars['Int']['input']>;
  page?: InputMaybe<Scalars['Int']['input']>;
  q: Scalars['String']['input'];
  searchType?: InputMaybe<SearchType>;
  sort?: InputMaybe<SortType>;
};


export type QuerySearchUsernamesArgs = {
  limit?: InputMaybe<Scalars['Int']['input']>;
  query: Scalars['String']['input'];
};


export type QueryUserArgs = {
  username: Scalars['String']['input'];
};


export type QueryUserFollowersArgs = {
  userId: Scalars['ID']['input'];
};


export type QueryUserFollowingArgs = {
  userId: Scalars['ID']['input'];
};


export type QueryWikiPageArgs = {
  boardName: Scalars['String']['input'];
  slug: Scalars['String']['input'];
};


export type QueryWikiPageHistoryArgs = {
  pageId: Scalars['ID']['input'];
};

export type Reaction = {
  __typename?: 'Reaction';
  commentId?: Maybe<Scalars['ID']['output']>;
  createdAt: Scalars['String']['output'];
  emoji: Scalars['String']['output'];
  id: Scalars['ID']['output'];
  postId?: Maybe<Scalars['ID']['output']>;
  score: Scalars['Int']['output'];
  userId: Scalars['ID']['output'];
};

export type ReactionAggregate = {
  __typename?: 'ReactionAggregate';
  commentId?: Maybe<Scalars['ID']['output']>;
  count: Scalars['Int']['output'];
  emoji: Scalars['String']['output'];
  id: Scalars['ID']['output'];
  postId?: Maybe<Scalars['ID']['output']>;
};

export type RegistrationApplication = {
  __typename?: 'RegistrationApplication';
  adminId?: Maybe<Scalars['ID']['output']>;
  answer: Scalars['String']['output'];
  createdAt: Scalars['String']['output'];
  denyReason?: Maybe<Scalars['String']['output']>;
  id: Scalars['ID']['output'];
  userId: Scalars['ID']['output'];
};

export type RemoveModeratorResponse = {
  __typename?: 'RemoveModeratorResponse';
  message: Scalars['String']['output'];
  success: Scalars['Boolean']['output'];
};

export type RemoveReactionInput = {
  commentId?: InputMaybe<Scalars['ID']['input']>;
  emoji: Scalars['String']['input'];
  postId?: InputMaybe<Scalars['ID']['input']>;
};

export type RemoveReactionResponse = {
  __typename?: 'RemoveReactionResponse';
  success: Scalars['Boolean']['output'];
};

export type ReportResponse = {
  __typename?: 'ReportResponse';
  reportId: Scalars['ID']['output'];
  success: Scalars['Boolean']['output'];
};

export type ResolveReportResponse = {
  __typename?: 'ResolveReportResponse';
  success: Scalars['Boolean']['output'];
};

export type SearchResult = {
  __typename?: 'SearchResult';
  boards: Array<Board>;
  comments: Array<Comment>;
  posts: Array<Post>;
  users: Array<User>;
};

export type SearchType =
  | 'all'
  | 'boards'
  | 'comments'
  | 'posts'
  | 'users';

export type SendMessageInput = {
  body: Scalars['String']['input'];
  recipientId: Scalars['ID']['input'];
  subject?: InputMaybe<Scalars['String']['input']>;
};

export type SendMessageResponse = {
  __typename?: 'SendMessageResponse';
  message: PrivateMessage;
};

export type SiteInviteGql = {
  __typename?: 'SiteInviteGql';
  createdAt: Scalars['String']['output'];
  id: Scalars['ID']['output'];
  verificationCode: Scalars['String']['output'];
};

export type SiteStats = {
  __typename?: 'SiteStats';
  boards: Scalars['Int']['output'];
  comments: Scalars['Int']['output'];
  posts: Scalars['Int']['output'];
  users: Scalars['Int']['output'];
  usersActiveDay: Scalars['Int']['output'];
  usersActiveHalfYear: Scalars['Int']['output'];
  usersActiveMonth: Scalars['Int']['output'];
  usersActiveWeek: Scalars['Int']['output'];
};

export type SortType =
  | 'active'
  | 'controversial'
  | 'hot'
  | 'mostComments'
  | 'new'
  | 'newComments'
  | 'old'
  | 'topAll'
  | 'topDay'
  | 'topMonth'
  | 'topWeek'
  | 'topYear';

export type TransferOwnershipResponse = {
  __typename?: 'TransferOwnershipResponse';
  message: Scalars['String']['output'];
  success: Scalars['Boolean']['output'];
};

export type UnbanUserResponse = {
  __typename?: 'UnbanUserResponse';
  message: Scalars['String']['output'];
  success: Scalars['Boolean']['output'];
};

export type UnreadNotificationCount = {
  __typename?: 'UnreadNotificationCount';
  activity: Scalars['Int']['output'];
  mentions: Scalars['Int']['output'];
  privateMessages: Scalars['Int']['output'];
  replies: Scalars['Int']['output'];
  total: Scalars['Int']['output'];
};

export type UpdateBoardReactionSettingsInput = {
  boardId: Scalars['ID']['input'];
  emojiWeights?: InputMaybe<Scalars['JSON']['input']>;
  isReactionsEnabled?: InputMaybe<Scalars['Boolean']['input']>;
};

export type UpdateBoardReactionSettingsResponse = {
  __typename?: 'UpdateBoardReactionSettingsResponse';
  settings: BoardReactionSettings;
};

export type UpdateBoardSettingsInput = {
  banner?: InputMaybe<Scalars['String']['input']>;
  boardId: Scalars['ID']['input'];
  description?: InputMaybe<Scalars['String']['input']>;
  excludeFromAll?: InputMaybe<Scalars['Boolean']['input']>;
  hoverColor?: InputMaybe<Scalars['String']['input']>;
  icon?: InputMaybe<Scalars['String']['input']>;
  isHidden?: InputMaybe<Scalars['Boolean']['input']>;
  isNsfw?: InputMaybe<Scalars['Boolean']['input']>;
  mode?: InputMaybe<Scalars['String']['input']>;
  postingRestrictedToMods?: InputMaybe<Scalars['Boolean']['input']>;
  primaryColor?: InputMaybe<Scalars['String']['input']>;
  secondaryColor?: InputMaybe<Scalars['String']['input']>;
  sidebar?: InputMaybe<Scalars['String']['input']>;
  title?: InputMaybe<Scalars['String']['input']>;
  wikiEnabled?: InputMaybe<Scalars['Boolean']['input']>;
  customCss?: InputMaybe<Scalars['String']['input']>;
};

export type UpdateBoardSettingsResponse = {
  __typename?: 'UpdateBoardSettingsResponse';
  board: Board;
};

export type UpdateEmojiInput = {
  altText?: InputMaybe<Scalars['String']['input']>;
  category?: InputMaybe<Scalars['String']['input']>;
  imageUrl?: InputMaybe<Scalars['String']['input']>;
  isActive?: InputMaybe<Scalars['Boolean']['input']>;
  keywords?: InputMaybe<Array<Scalars['String']['input']>>;
  shortcode?: InputMaybe<Scalars['String']['input']>;
};

export type UpdateFlairCategoryInput = {
  color?: InputMaybe<Scalars['String']['input']>;
  description?: InputMaybe<Scalars['String']['input']>;
  displayOrder?: InputMaybe<Scalars['Int']['input']>;
  name?: InputMaybe<Scalars['String']['input']>;
};

export type UpdateFlairTemplateInput = {
  backgroundColor?: InputMaybe<Scalars['String']['input']>;
  categoryId?: InputMaybe<Scalars['ID']['input']>;
  displayOrder?: InputMaybe<Scalars['Int']['input']>;
  emojiIds?: InputMaybe<Array<Scalars['Int']['input']>>;
  isActive?: InputMaybe<Scalars['Boolean']['input']>;
  isEditable?: InputMaybe<Scalars['Boolean']['input']>;
  isModOnly?: InputMaybe<Scalars['Boolean']['input']>;
  isRequiresApproval?: InputMaybe<Scalars['Boolean']['input']>;
  maxEmojiCount?: InputMaybe<Scalars['Int']['input']>;
  maxTextLength?: InputMaybe<Scalars['Int']['input']>;
  styleConfig?: InputMaybe<FlairStyleInput>;
  templateName?: InputMaybe<Scalars['String']['input']>;
  textColor?: InputMaybe<Scalars['String']['input']>;
  textDisplay?: InputMaybe<Scalars['String']['input']>;
};

export type UpdateNotificationSettingsInput = {
  boardInvitesEnabled?: InputMaybe<Scalars['Boolean']['input']>;
  commentRepliesEnabled?: InputMaybe<Scalars['Boolean']['input']>;
  emailEnabled?: InputMaybe<Scalars['Boolean']['input']>;
  mentionsEnabled?: InputMaybe<Scalars['Boolean']['input']>;
  moderatorActionsEnabled?: InputMaybe<Scalars['Boolean']['input']>;
  postRepliesEnabled?: InputMaybe<Scalars['Boolean']['input']>;
  privateMessagesEnabled?: InputMaybe<Scalars['Boolean']['input']>;
  systemNotificationsEnabled?: InputMaybe<Scalars['Boolean']['input']>;
};

export type UpdateNotificationSettingsResponse = {
  __typename?: 'UpdateNotificationSettingsResponse';
  settings: NotificationSettings;
  success: Scalars['Boolean']['output'];
};

export type UpdateProfileInput = {
  avatar?: InputMaybe<Scalars['String']['input']>;
  avatarFrame?: InputMaybe<Scalars['String']['input']>;
  banner?: InputMaybe<Scalars['String']['input']>;
  bio?: InputMaybe<Scalars['String']['input']>;
  displayName?: InputMaybe<Scalars['String']['input']>;
  profileBackground?: InputMaybe<Scalars['String']['input']>;
  profileMusic?: InputMaybe<Scalars['String']['input']>;
  profileMusicYoutube?: InputMaybe<Scalars['String']['input']>;
  signature?: InputMaybe<Scalars['String']['input']>;
};

export type UpdateSettingsInput = {
  defaultListingType?: InputMaybe<Scalars['String']['input']>;
  defaultSortType?: InputMaybe<Scalars['String']['input']>;
  editorMode?: InputMaybe<Scalars['String']['input']>;
  email?: InputMaybe<Scalars['String']['input']>;
  interfaceLanguage?: InputMaybe<Scalars['String']['input']>;
  isEmailNotificationsEnabled?: InputMaybe<Scalars['Boolean']['input']>;
  showBots?: InputMaybe<Scalars['Boolean']['input']>;
  showNsfw?: InputMaybe<Scalars['Boolean']['input']>;
  theme?: InputMaybe<Scalars['String']['input']>;
};

export type UpdateSiteConfigInput = {
  applicationEmailAdmins?: InputMaybe<Scalars['Boolean']['input']>;
  applicationQuestion?: InputMaybe<Scalars['String']['input']>;
  bannedDomains?: InputMaybe<Scalars['String']['input']>;
  boardCreationAdminOnly?: InputMaybe<Scalars['Boolean']['input']>;
  boardCreationMode?: InputMaybe<Scalars['String']['input']>;
  boardEmojisEnabled?: InputMaybe<Scalars['Boolean']['input']>;
  boardsEnabled?: InputMaybe<Scalars['Boolean']['input']>;
  captchaDifficulty?: InputMaybe<Scalars['String']['input']>;
  captchaEnabled?: InputMaybe<Scalars['Boolean']['input']>;
  defaultTheme?: InputMaybe<Scalars['String']['input']>;
  description?: InputMaybe<Scalars['String']['input']>;
  emojiEnabled?: InputMaybe<Scalars['Boolean']['input']>;
  enableDownvotes?: InputMaybe<Scalars['Boolean']['input']>;
  enableNsfw?: InputMaybe<Scalars['Boolean']['input']>;
  enableNsfwTagging?: InputMaybe<Scalars['Boolean']['input']>;
  filteredWords?: InputMaybe<Scalars['String']['input']>;
  hideModlogModNames?: InputMaybe<Scalars['Boolean']['input']>;
  homepageBanner?: InputMaybe<Scalars['String']['input']>;
  hoverColor?: InputMaybe<Scalars['String']['input']>;
  icon?: InputMaybe<Scalars['String']['input']>;
  isPrivate?: InputMaybe<Scalars['Boolean']['input']>;
  legalInformation?: InputMaybe<Scalars['String']['input']>;
  linkFilterEnabled?: InputMaybe<Scalars['Boolean']['input']>;
  name?: InputMaybe<Scalars['String']['input']>;
  primaryColor?: InputMaybe<Scalars['String']['input']>;
  registrationMode?: InputMaybe<Scalars['String']['input']>;
  reportsEmailAdmins?: InputMaybe<Scalars['Boolean']['input']>;
  requireEmailVerification?: InputMaybe<Scalars['Boolean']['input']>;
  secondaryColor?: InputMaybe<Scalars['String']['input']>;
  welcomeMessage?: InputMaybe<Scalars['String']['input']>;
  wordFilterEnabled?: InputMaybe<Scalars['Boolean']['input']>;
  defaultBoardMode?: InputMaybe<Scalars['String']['input']>;
  customCss?: InputMaybe<Scalars['String']['input']>;
  customCssEnabled?: InputMaybe<Scalars['Boolean']['input']>;
};

export type User = {
  __typename?: 'User';
  adminLevel: Scalars['Int']['output'];
  avatar?: Maybe<Scalars['String']['output']>;
  avatarFrame?: Maybe<Scalars['String']['output']>;
  banner?: Maybe<Scalars['String']['output']>;
  bio?: Maybe<Scalars['String']['output']>;
  bioHTML?: Maybe<Scalars['String']['output']>;
  commentCount: Scalars['Int']['output'];
  commentScore: Scalars['Int']['output'];
  createdAt: Scalars['String']['output'];
  displayName?: Maybe<Scalars['String']['output']>;
  id: Scalars['ID']['output'];
  isAdmin: Scalars['Boolean']['output'];
  isBanned: Scalars['Boolean']['output'];
  isBotAccount: Scalars['Boolean']['output'];
  lastSeenAt: Scalars['String']['output'];
  name: Scalars['String']['output'];
  postCount: Scalars['Int']['output'];
  postScore: Scalars['Int']['output'];
  profileBackground?: Maybe<Scalars['String']['output']>;
  profileMusic?: Maybe<Scalars['String']['output']>;
  profileMusicYoutube?: Maybe<Scalars['String']['output']>;
  signature?: Maybe<Scalars['String']['output']>;
  unbanDate?: Maybe<Scalars['String']['output']>;
};

export type UserFlair = {
  __typename?: 'UserFlair';
  approvedAt?: Maybe<Scalars['String']['output']>;
  approvedBy?: Maybe<Scalars['ID']['output']>;
  boardId: Scalars['ID']['output'];
  createdAt: Scalars['String']['output'];
  customBackgroundColor?: Maybe<Scalars['String']['output']>;
  customText?: Maybe<Scalars['String']['output']>;
  customTextColor?: Maybe<Scalars['String']['output']>;
  flairTemplateId: Scalars['ID']['output'];
  id: Scalars['ID']['output'];
  isApproved: Scalars['Boolean']['output'];
  isSelfAssigned: Scalars['Boolean']['output'];
  userId: Scalars['ID']['output'];
};

export type UserSettings = {
  __typename?: 'UserSettings';
  defaultListingType: Scalars['String']['output'];
  defaultSortType: Scalars['String']['output'];
  editorMode: Scalars['String']['output'];
  email?: Maybe<Scalars['String']['output']>;
  id: Scalars['ID']['output'];
  interfaceLanguage: Scalars['String']['output'];
  isEmailNotificationsEnabled: Scalars['Boolean']['output'];
  name: Scalars['String']['output'];
  showBots: Scalars['Boolean']['output'];
  showNSFW: Scalars['Boolean']['output'];
  theme: Scalars['String']['output'];
  updatedAt: Scalars['String']['output'];
};

export type WikiPage = {
  __typename?: 'WikiPage';
  boardId: Scalars['ID']['output'];
  body: Scalars['String']['output'];
  bodyHTML?: Maybe<Scalars['String']['output']>;
  createdAt: Scalars['String']['output'];
  creatorId: Scalars['ID']['output'];
  displayOrder: Scalars['Int']['output'];
  editPermission: Scalars['String']['output'];
  id: Scalars['ID']['output'];
  isDeleted: Scalars['Boolean']['output'];
  isLocked: Scalars['Boolean']['output'];
  lastEditedBy?: Maybe<Scalars['ID']['output']>;
  parentId?: Maybe<Scalars['ID']['output']>;
  slug: Scalars['String']['output'];
  title: Scalars['String']['output'];
  updatedAt: Scalars['String']['output'];
  viewPermission: Scalars['String']['output'];
};

export type WikiRevision = {
  __typename?: 'WikiRevision';
  body: Scalars['String']['output'];
  createdAt: Scalars['String']['output'];
  creator?: Maybe<User>;
  editSummary?: Maybe<Scalars['String']['output']>;
  id: Scalars['ID']['output'];
  revisionNumber: Scalars['Int']['output'];
};
