use crate::structs::{SiteInviteView};
use diesel::{result::Error, *};
use tinyboards_db::{
    models::{
        site::site_invite::SiteInvite,
    },
    schema::site_invite,
    traits::ViewToVec,
    utils::limit_and_offset,
};
use typed_builder::TypedBuilder;

type SiteInviteViewTuple = (
    SiteInvite,
);

#[derive(TypedBuilder)]
#[builder(field_defaults(default))]
pub struct InviteQuery<'a> {
    #[builder(!default)]
    conn: &'a mut PgConnection,
    page: Option<i64>,
    limit: Option<i64>,
}

#[derive(Default, Clone)]
pub struct InviteQueryResponse {
    pub invites: Vec<SiteInviteView>,
    pub count: i64,
}

impl<'a> InviteQuery<'a> {
    pub fn list(self) -> Result<InviteQueryResponse, Error> {
        
        let query = site_invite::table
            .select((
                site_invite::all_columns,
            ))
            .into_boxed();

        let count_query = site_invite::table
        .select((
            site_invite::all_columns,
        ))
        .into_boxed();

        let (limit, offset) = limit_and_offset(self.page, self.limit)?;
    
        let res = query
            .limit(limit)
            .offset(offset)
            .load::<SiteInviteViewTuple>(self.conn)?;

        let invites = SiteInviteView::from_tuple_to_vec(res);
        let count = count_query.count().get_result::<i64>(self.conn)?;

        Ok(InviteQueryResponse { invites, count })
        
    }
}

impl ViewToVec for SiteInviteView {
    type DbTuple = SiteInviteViewTuple;
    fn from_tuple_to_vec(items: Vec<Self::DbTuple>) -> Vec<Self> {
        items
            .into_iter()
            .map(|a| Self {
                invite: a.0,
            })
            .collect::<Vec<Self>>()
    }
}