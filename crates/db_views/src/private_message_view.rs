use crate::structs::PrivateMessageView;
use diesel::{result::Error, *};
use tinyboards_db::{
    models::user::private_messages::{PrivateMessage},
    models::user::users::UserSafe,
    schema::{users, private_messages},
    traits::{ToSafe, ViewToVec}, utils::limit_and_offset,
};

type PrivateMessageViewTuple = (
    PrivateMessage,
    UserSafe,
    UserSafe,
);
use typed_builder::TypedBuilder;

impl PrivateMessageView {
    pub fn read(
        conn: &mut PgConnection,
        pm_id: i32,
    ) -> Result<Self, Error> {
        
        let user_alias = diesel::alias!(users as users1);
        
        let (
            private_message,
            creator,
            recipient,
        ) = private_messages::table
            .find(pm_id)
            .inner_join(users::table.on(private_messages::creator_id.eq(users::id)))
            .inner_join(
                user_alias.on(private_messages::recipient_id.eq(user_alias.field(users::id))),
            )
            .order_by(private_messages::creation_date.desc())
            .select((
                private_messages::all_columns,
                UserSafe::safe_columns_tuple(),
                UserSafe::safe_columns_tuple(),
            ))
            .first::<PrivateMessageViewTuple>(conn)?;

        Ok(PrivateMessageView { private_message, creator, recipient })
    }
    
    pub fn get_unread_message_count(conn: &mut PgConnection, user_id: i32) -> Result<i64, Error> {
        use diesel::dsl::count;
        private_messages::table
            .filter(private_messages::read.eq(false))
            .filter(private_messages::recipient_id.eq(user_id))
            .filter(private_messages::is_deleted.eq(false))
            .select(count(private_messages::id))
            .first::<i64>(conn)
    }
}

#[derive(TypedBuilder)]
#[builder(field_defaults(default))]
pub struct PrivateMessageQuery<'a> {
    #[builder(!default)]
    conn: &'a mut PgConnection,
    recipient_id: i32,
    unread_only: Option<bool>,
    page: Option<i64>,
    limit: Option<i64>,
}

impl<'a> PrivateMessageQuery<'a> {
    pub fn list(self) -> Result<Vec<PrivateMessageView>, Error> {
        let user_alias = diesel::alias!(users as users1);
        
        let mut query = private_messages::table
            .inner_join(users::table.on(private_messages::creator_id.eq(users::id)))
            .inner_join(
                user_alias.on(private_messages::recipient_id.eq(user_alias.field(users::id))),
            )
            .select((
                private_messages::all_columns,
                UserSafe::safe_columns_tuple(),
                UserSafe::safe_columns_tuple(),
            ))
        .into_boxed();
        
        // if it's unread we only want unread messages to the end user
        if self.unread_only.unwrap_or(false) {
            query = query
                .filter(private_messages::read.eq(false))
                .filter(private_messages::recipient_id.eq(self.recipient_id));
        }
        // otherwise we return both sent and received messages
        else {
            query = query
                .filter(
                    private_messages::recipient_id
                    .eq(self.recipient_id)
                    .or(private_messages::creator_id.eq(self.recipient_id)),
                )
        }

        let (limit, offset) = limit_and_offset(self.page, self.limit)?;

        query = query
            .filter(private_messages::is_deleted.eq(false))
            .limit(limit)
            .offset(offset)
            .order_by(private_messages::creation_date.desc());
        
        let res = query.load::<PrivateMessageViewTuple>(self.conn)?;

        Ok(PrivateMessageView::from_tuple_to_vec(res))
    }
}



impl ViewToVec for PrivateMessageView {
    type DbTuple = PrivateMessageViewTuple;
    fn from_tuple_to_vec(items: Vec<Self::DbTuple>) -> Vec<Self> {
        items
            .into_iter()
            .map(|a| Self {
                private_message: a.0,
                creator: a.1,
                recipient: a.2,
            })
            .collect::<Vec<Self>>()
    }
}