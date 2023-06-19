use crate::{
    activities::{
      deletion::{receive_delete_action, verify_delete_activity, DeletableObjects},
      generate_activity_id,
    },
    insert_activity,
    objects::person::ApubPerson,
    protocol::{activities::deletion::delete::Delete, IdOrNestedObject},
  };
  use activitypub_federation::{config::Data, kinds::activity::DeleteType, traits::ActivityHandler};
  use lemmy_api_common::context::LemmyContext;
  use lemmy_db_schema::{
    source::{
      comment::{Comment, CommentUpdateForm},
      community::{Community, CommunityUpdateForm},
      moderator::{
        ModRemoveComment,
        ModRemoveCommentForm,
        ModRemoveCommunity,
        ModRemoveCommunityForm,
        ModRemovePost,
        ModRemovePostForm,
      },
      post::{Post, PostUpdateForm},
    },
    traits::Crud,
  };
use lemmy_utils::error::LemmyError;
use url::Url;