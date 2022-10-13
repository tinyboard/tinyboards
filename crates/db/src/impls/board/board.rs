// use crate::{
//     models::board::{
//         board::Board
//     },
//     traits::{Crud, Followable, Joinable, Bannable},
//     SubscribedType,
// };
// use diesel::{
//     dsl::*,
//     result::Error,
//     ExpressionMethods,
//     PgConnection,
//     QueryDsl,
//     RunQueryDsl,
//     TextExpressionMethods,
// };

pub mod safe_type {
    use crate::{schema::board::*, models::board::board::BoardSafe, traits::ToSafe};

    type Columns = (
        id,
        name,
        title,
        description,
        published,
        updated,
        deleted,
        nsfw,
    );

    impl ToSafe for BoardSafe {
        type SafeColumns = Columns;
        fn safe_columns_tuple() -> Self::SafeColumns {
            (
                id,
                name,
                title,
                description,
                published,
                updated,
                deleted,
                nsfw,
            )
        }
    }
}

