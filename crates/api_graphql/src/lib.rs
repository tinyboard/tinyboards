pub mod queries;
use actix_web::web::{self, Data as ActixData};
use async_graphql::{Data as GraphQLData, EmptyMutation, EmptySubscription, Schema};
use async_graphql_actix_web::{GraphQL, GraphQLRequest, GraphQLResponse};
use queries::Query;
//use tinyboards_api_common::data::TinyBoardsContext;

pub fn gen_schema() -> Schema<Query, EmptyMutation, EmptySubscription> {
    Schema::new(Query, EmptyMutation, EmptySubscription)
}

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
