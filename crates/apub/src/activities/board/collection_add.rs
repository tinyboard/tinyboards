use crate::{
    activities::{
        board::send_activity_in_board,
        generate_activity_id,
        verify_is_public,
        verify_mod_action,
        verify_person_in_board,
    },
    activity_lists::AnnouncableActivities,
    insert_activity,
    objects::{
        board::ApubBoard,
        person::ApubPerson,
        post::ApubPost,
    },
    protocol::{
        activities::board::{collection_add::CollectionAdd, collection_remove::CollectionRemove},
        InBoard,
    },
    SendActivity,
};
use tinyboards_federation::{
    config::Data,
    fetch::object_id::ObjectId,
    kinds::{activity::AddType, public},
    traits::{ActivityHandler, Actor},
};