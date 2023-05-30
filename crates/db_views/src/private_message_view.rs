use crate::structs::PrivateMessageView;
use diesel::{result::Error, *};
use tinyboards_db::{
    models::local_user::private_messages::{PrivateMessage},
    models::local_user::users::UserSafe,
    schema::{users, private_messages},
    traits::{ToSafe, ViewToVec}, utils::{limit_and_offset, get_conn, DbPool},
};
use diesel_async::RunQueryDsl;

type PrivateMessageViewTuple = (
    PrivateMessage,
    UserSafe,
    UserSafe,
);
use typed_builder::TypedBuilder;

impl PrivateMessageView {
    pub async fn read(
        pool: &DbPool,
        pm_id: i32,
    ) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        
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
            .first::<PrivateMessageViewTuple>(conn)
            .await?;

        Ok(PrivateMessageView { private_message, creator, recipient })
    }
    
    pub async fn get_unread_message_count(pool: &DbPool, person_id: i32) -> Result<i64, Error> {
        let conn = &mut get_conn(pool).await?;
        use diesel::dsl::count;
        private_messages::table
            .filter(private_messages::read.eq(false))
            .filter(private_messages::recipient_id.eq(person_id))
            .filter(private_messages::is_deleted.eq(false))
            .select(count(private_messages::id))
            .first::<i64>(conn)
            .await
    }

    pub async fn mark_thread_as_read(pool: &DbPool, creator_id: i32) -> Result<usize, Error> {
        let conn = &mut get_conn(pool).await?;
        diesel::update(private_messages::table)
            .filter(private_messages::read.eq(false))
            .filter(private_messages::creator_id.eq(creator_id))
            .set(private_messages::read.eq(true))
            .execute(conn)
            .await
    }

    pub async fn thread_exists(pool: &DbPool, creator_id: i32, recipient_id: i32) -> Result<bool, Error> {
        let conn = &mut get_conn(pool).await?;
        use diesel::dsl::count;

        let messages = private_messages::table
            .filter(private_messages::creator_id.eq(creator_id).and(private_messages::recipient_id.eq(recipient_id)))
            .select(count(private_messages::id))
            .first::<i64>(conn)
            .await;

        if messages == Ok(0) {
            Ok(false)
        } else {
            Ok(true)
        }
    }

}

#[derive(TypedBuilder)]
#[builder(field_defaults(default))]
pub struct PrivateMessageQuery<'a> {
    #[builder(!default)]
    pool: &'a DbPool,
    #[builder(!default)]
    recipient_id: i32,
    creator_id: Option<i32>,
    unread_only: Option<bool>,
    page: Option<i64>,
    limit: Option<i64>,
}

#[derive(Default, Clone)]
pub struct PrivateMessageQueryResponse {
    pub messages: Vec<PrivateMessageView>,
    pub count: i64,
    pub unread: i64,
}

impl<'a> PrivateMessageQuery<'a> {
    pub async fn list(self) -> Result<PrivateMessageQueryResponse, Error> {
        let conn = &mut get_conn(self.pool).await?;
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

        let mut count_query = private_messages::table
            .inner_join(users::table.on(private_messages::creator_id.eq(users::id)))
            .inner_join(
                user_alias.on(private_messages::recipient_id.eq(user_alias.field(users::id))),
            )
        .into_boxed();
        
        if self.unread_only.unwrap_or(false) {
            query = query
                .filter(private_messages::read.eq(false));
            count_query = count_query
                .filter(private_messages::read.eq(false));
        } 

        // if creator_id is provided, then grab the thread of private messages,
        // if creator_id is not provided, then grab all the parent messages (whether the thread was initiated by the requester or not)
        if let Some(creator_id) = self.creator_id {
            query = query
                .filter(private_messages::creator_id.eq(creator_id).or(private_messages::recipient_id.eq(self.recipient_id)))
                .filter(private_messages::is_parent.eq(false));
            count_query = count_query
                .filter(private_messages::creator_id.eq(creator_id).or(private_messages::recipient_id.eq(self.recipient_id)))
                .filter(private_messages::is_parent.eq(false));
        } else {
            query = query
                .filter(private_messages::recipient_id.eq(self.recipient_id).and(private_messages::is_parent.eq(true)).or(
                    private_messages::creator_id.eq(self.recipient_id).and(private_messages::is_parent.eq(true)),
                ),
            );
            count_query = count_query
                .filter(private_messages::recipient_id.eq(self.recipient_id).and(private_messages::is_parent.eq(true)).or(
                    private_messages::creator_id.eq(self.recipient_id).and(private_messages::is_parent.eq(true)),
                ),
            );
        }

        let (limit, offset) = limit_and_offset(self.page, self.limit)?;

        query = query
            .filter(private_messages::is_deleted.eq(false))
            .limit(limit)
            .offset(offset)
            .order_by(private_messages::creation_date.desc());
        
        let res = query.load::<PrivateMessageViewTuple>(conn).await?;

        let messages = PrivateMessageView::from_tuple_to_vec(res);
        let count = count_query.count().get_result::<i64>(conn).await?;
        let unread = PrivateMessageView::get_unread_message_count(self.pool, self.recipient_id).await?;

        Ok(PrivateMessageQueryResponse { messages, count, unread })
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