use crate::{models::moderator::mod_actions::*, traits::Crud};
use diesel::{result::Error, *};

impl Crud for ModAddBoard {
    type Form = ModAddBoardForm;
    type IdType = i32;
    fn read(conn: &mut PgConnection, id_: i32) -> Result<Self, Error> {
        use crate::schema::mod_add_board::dsl::*;
        mod_add_board.find(id_).first::<Self>(conn)
    }
    fn delete(conn: &mut PgConnection, id_: i32) -> Result<usize, Error> {
        use crate::schema::mod_add_board::dsl::*;
        diesel::delete(mod_add_board.find(id_)).execute(conn)
    }
    fn create(conn: &mut PgConnection, form: &Self::Form) -> Result<Self, Error> {
        use crate::schema::mod_add_board::dsl::*;
        let new = diesel::insert_into(mod_add_board)
            .values(form)
            .get_result::<Self>(conn)?;
        Ok(new)
    }
    fn update(conn: &mut PgConnection, id_: i32, form: &Self::Form) -> Result<Self, Error> {
        use crate::schema::mod_add_board::dsl::*;
        diesel::update(mod_add_board.find(id_))
            .set(form)
            .get_result::<Self>(conn)
    }
}

impl Crud for ModAddAdmin {
    type Form = ModAddAdminForm;
    type IdType = i32;
    fn read(conn: &mut PgConnection, id_: i32) -> Result<Self, Error> {
        use crate::schema::mod_add_admin::dsl::*;
        mod_add_admin.find(id_).first::<Self>(conn)
    }
    fn delete(conn: &mut PgConnection, id_: i32) -> Result<usize, Error> {
        use crate::schema::mod_add_admin::dsl::*;
        diesel::delete(mod_add_admin.find(id_)).execute(conn)
    }
    fn create(conn: &mut PgConnection, form: &Self::Form) -> Result<Self, Error> {
        use crate::schema::mod_add_admin::dsl::*;
        let new = diesel::insert_into(mod_add_admin)
            .values(form)
            .get_result::<Self>(conn)?;
        Ok(new)
    }
    fn update(conn: &mut PgConnection, id_: i32, form: &Self::Form) -> Result<Self, Error> {
        use crate::schema::mod_add_admin::dsl::*;
        diesel::update(mod_add_admin.find(id_))
            .set(form)
            .get_result::<Self>(conn)
    }
}

impl Crud for ModAddBoardMod {
    type Form = ModAddBoardModForm;
    type IdType = i32;
    fn read(conn: &mut PgConnection, id_: i32) -> Result<Self, Error> {
        use crate::schema::mod_add_board_mod::dsl::*;
        mod_add_board_mod.find(id_).first::<Self>(conn)
    }
    fn delete(conn: &mut PgConnection, id_: i32) -> Result<usize, Error> {
        use crate::schema::mod_add_board_mod::dsl::*;
        diesel::delete(mod_add_board_mod.find(id_)).execute(conn)
    }
    fn create(conn: &mut PgConnection, form: &Self::Form) -> Result<Self, Error> {
        use crate::schema::mod_add_board_mod::dsl::*;
        let new = diesel::insert_into(mod_add_board_mod)
            .values(form)
            .get_result::<Self>(conn)?;
        Ok(new)
    }
    fn update(conn: &mut PgConnection, id_: i32, form: &Self::Form) -> Result<Self, Error> {
        use crate::schema::mod_add_board_mod::dsl::*;
        diesel::update(mod_add_board_mod.find(id_))
            .set(form)
            .get_result::<Self>(conn)
    }
}

impl Crud for ModBanFromBoard {
    type Form = ModBanFromBoardForm;
    type IdType = i32;
    fn read(conn: &mut PgConnection, id_: i32) -> Result<Self, Error> {
        use crate::schema::mod_ban_from_board::dsl::*;
        mod_ban_from_board.find(id_).first::<Self>(conn)
    }
    fn delete(conn: &mut PgConnection, id_: i32) -> Result<usize, Error> {
        use crate::schema::mod_ban_from_board::dsl::*;
        diesel::delete(mod_ban_from_board.find(id_)).execute(conn)
    }
    fn create(conn: &mut PgConnection, form: &Self::Form) -> Result<Self, Error> {
        use crate::schema::mod_ban_from_board::dsl::*;
        let new = diesel::insert_into(mod_ban_from_board)
            .values(form)
            .get_result::<Self>(conn)?;
        Ok(new)
    }
    fn update(conn: &mut PgConnection, id_: i32, form: &Self::Form) -> Result<Self, Error> {
        use crate::schema::mod_ban_from_board::dsl::*;
        diesel::update(mod_ban_from_board.find(id_))
            .set(form)
            .get_result::<Self>(conn)
    }
}

impl Crud for ModBan {
    type Form = ModBanForm;
    type IdType = i32;
    fn read(conn: &mut PgConnection, id_: i32) -> Result<Self, Error> {
        use crate::schema::mod_ban::dsl::*;
        mod_ban.find(id_).first::<Self>(conn)
    }
    fn delete(conn: &mut PgConnection, id_: i32) -> Result<usize, Error> {
        use crate::schema::mod_ban::dsl::*;
        diesel::delete(mod_ban.find(id_)).execute(conn)
    }
    fn create(conn: &mut PgConnection, form: &Self::Form) -> Result<Self, Error> {
        use crate::schema::mod_ban::dsl::*;
        let new = diesel::insert_into(mod_ban)
            .values(form)
            .get_result::<Self>(conn)?;
        Ok(new)
    }
    fn update(conn: &mut PgConnection, id_: i32, form: &Self::Form) -> Result<Self, Error> {
        use crate::schema::mod_ban::dsl::*;
        diesel::update(mod_ban.find(id_))
            .set(form)
            .get_result::<Self>(conn)
    }
}

impl Crud for ModLockPost {
    type Form = ModLockPostForm;
    type IdType = i32;
    fn read(conn: &mut PgConnection, id_: i32) -> Result<Self, Error> {
        use crate::schema::mod_lock_post::dsl::*;
        mod_lock_post.find(id_).first::<Self>(conn)
    }
    fn delete(conn: &mut PgConnection, id_: i32) -> Result<usize, Error> {
        use crate::schema::mod_lock_post::dsl::*;
        diesel::delete(mod_lock_post.find(id_)).execute(conn)
    }
    fn create(conn: &mut PgConnection, form: &Self::Form) -> Result<Self, Error> {
        use crate::schema::mod_lock_post::dsl::*;
        let new = diesel::insert_into(mod_lock_post)
            .values(form)
            .get_result::<Self>(conn)?;
        Ok(new)
    }
    fn update(conn: &mut PgConnection, id_: i32, form: &Self::Form) -> Result<Self, Error> {
        use crate::schema::mod_lock_post::dsl::*;
        diesel::update(mod_lock_post.find(id_))
            .set(form)
            .get_result::<Self>(conn)
    }
}

impl Crud for ModRemoveBoard {
    type Form = ModRemoveBoardForm;
    type IdType = i32;
    fn read(conn: &mut PgConnection, id_: i32) -> Result<Self, Error> {
        use crate::schema::mod_remove_board::dsl::*;
        mod_remove_board.find(id_).first::<Self>(conn)
    }
    fn delete(conn: &mut PgConnection, id_: i32) -> Result<usize, Error> {
        use crate::schema::mod_remove_board::dsl::*;
        diesel::delete(mod_remove_board.find(id_)).execute(conn)
    }
    fn create(conn: &mut PgConnection, form: &Self::Form) -> Result<Self, Error> {
        use crate::schema::mod_remove_board::dsl::*;
        let new = diesel::insert_into(mod_remove_board)
            .values(form)
            .get_result::<Self>(conn)?;
        Ok(new)
    }
    fn update(conn: &mut PgConnection, id_: i32, form: &Self::Form) -> Result<Self, Error> {
        use crate::schema::mod_remove_board::dsl::*;
        diesel::update(mod_remove_board.find(id_))
            .set(form)
            .get_result::<Self>(conn)
    }
}

impl Crud for ModRemoveComment {
    type Form = ModRemoveCommentForm;
    type IdType = i32;
    fn read(conn: &mut PgConnection, id_: i32) -> Result<Self, Error> {
        use crate::schema::mod_remove_comment::dsl::*;
        mod_remove_comment.find(id_).first::<Self>(conn)
    }
    fn delete(conn: &mut PgConnection, id_: i32) -> Result<usize, Error> {
        use crate::schema::mod_remove_comment::dsl::*;
        diesel::delete(mod_remove_comment.find(id_)).execute(conn)
    }
    fn create(conn: &mut PgConnection, form: &Self::Form) -> Result<Self, Error> {
        use crate::schema::mod_remove_comment::dsl::*;
        let new = diesel::insert_into(mod_remove_comment)
            .values(form)
            .get_result::<Self>(conn)?;
        Ok(new)
    }
    fn update(conn: &mut PgConnection, id_: i32, form: &Self::Form) -> Result<Self, Error> {
        use crate::schema::mod_remove_comment::dsl::*;
        diesel::update(mod_remove_comment.find(id_))
            .set(form)
            .get_result::<Self>(conn)
    }
}

impl Crud for ModRemovePost {
    type Form = ModRemovePostForm;
    type IdType = i32;
    fn read(conn: &mut PgConnection, id_: i32) -> Result<Self, Error> {
        use crate::schema::mod_remove_post::dsl::*;
        mod_remove_post.find(id_).first::<Self>(conn)
    }
    fn delete(conn: &mut PgConnection, id_: i32) -> Result<usize, Error> {
        use crate::schema::mod_remove_post::dsl::*;
        diesel::delete(mod_remove_post.find(id_)).execute(conn)
    }
    fn create(conn: &mut PgConnection, form: &Self::Form) -> Result<Self, Error> {
        use crate::schema::mod_remove_post::dsl::*;
        let new = diesel::insert_into(mod_remove_post)
            .values(form)
            .get_result::<Self>(conn)?;
        Ok(new)
    }
    fn update(conn: &mut PgConnection, id_: i32, form: &Self::Form) -> Result<Self, Error> {
        use crate::schema::mod_remove_post::dsl::*;
        diesel::update(mod_remove_post.find(id_))
            .set(form)
            .get_result::<Self>(conn)
    }
}

impl Crud for ModStickyPost {
    type Form = ModStickyPostForm;
    type IdType = i32;
    fn read(conn: &mut PgConnection, id_: i32) -> Result<Self, Error> {
        use crate::schema::mod_sticky_post::dsl::*;
        mod_sticky_post.find(id_).first::<Self>(conn)
    }
    fn delete(conn: &mut PgConnection, id_: i32) -> Result<usize, Error> {
        use crate::schema::mod_sticky_post::dsl::*;
        diesel::delete(mod_sticky_post.find(id_)).execute(conn)
    }
    fn create(conn: &mut PgConnection, form: &Self::Form) -> Result<Self, Error> {
        use crate::schema::mod_sticky_post::dsl::*;
        let new = diesel::insert_into(mod_sticky_post)
            .values(form)
            .get_result::<Self>(conn)?;
        Ok(new)
    }
    fn update(conn: &mut PgConnection, id_: i32, form: &Self::Form) -> Result<Self, Error> {
        use crate::schema::mod_sticky_post::dsl::*;
        diesel::update(mod_sticky_post.find(id_))
            .set(form)
            .get_result::<Self>(conn)
    }
}