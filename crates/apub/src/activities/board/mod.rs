use tinyboards_api_common::data::TinyBoardsContext;
use tinyboards_federation::{traits::Actor, config::Data};
use tinyboards_utils::TinyBoardsError;
use url::Url;

use crate::{
    activities::send_tinyboards_activity,
    activity_lists::AnnouncableActivities,
    objects::{board::ApubBoard, person::ApubPerson},
    protocol::activities::board::announce::AnnounceActivity,
};

pub mod announce;
pub mod collection_add;
pub mod collection_remove;
pub mod lock_page;
pub mod report;
pub mod update;

/// This function sends all activities which are happening in a board
/// to the correct inboxes.
/// 
/// For example Create/Page, Add/Mod etc.
/// 
/// Activities are sent to the board itself if it lives on another instance. If the board
/// is local, the activity is directly wrapped into Announce and sent to board subscribers.
/// Activities are also sent to those who subscribe to the actor (with exception of mod activities)
/// 
/// * `activity` - The activity that is being sent
/// * `actor` - The user who is sending said activity
/// * `board` - board in which activity is being sent
/// * `inboxes` - Any additional inboxes in which activity should be sent to
/// * `is_mod_activity` - True for things like Add/Mod, these are not sent to user followers
pub(crate) async fn send_activity_in_board(
    activity: AnnouncableActivities,
    actor: &ApubPerson,
    board: &ApubBoard,
    extra_inboxes: Vec<Url>,
    is_mod_action: bool,
    context: &Data<TinyBoardsContext>,
) -> Result<(), TinyBoardsError> {
    let mut inboxes = extra_inboxes;

    // send to user followers
    if !is_mod_action {
        inboxes.append(
            &mut 
        )
    }

    if board.local {
        // send directly to board subscribers
        AnnounceActivity::send(activity.clone().try_into()?, board, context).await?;
    } else {
        // send to the board, which will then forward to subscribers
        inboxes.push(board.shared_inbox_or_inbox());
    }

    send_tinyboards_activity(context, activity.clone(), actor, inboxes, false).await?;
    Ok(())
}