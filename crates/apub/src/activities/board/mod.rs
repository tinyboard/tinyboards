use crate::{
    activities::send_tinyboards_activity,
    activity_lists::AnnouncableActivities,
    objects::{board::ApubBoard, person::ApubPerson},
    protocol::activities::board::announce::AnnounceActivity,
};

pub mod announce;