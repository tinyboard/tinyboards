use crate::Perform;
use actix_web::web::Data;
use tinyboards_api_common::{
    admin::{AddAdmin, AddAdminResponse},
    data::TinyBoardsContext,
    utils::{require_user, send_system_message},
};
use tinyboards_db::{
    models::{
        moderator::mod_actions::{ModAddAdmin, ModAddAdminForm},
        person::{
            local_user::{AdminPerms, LocalUser},
            person::Person,
        },
    },
    //schema::comments::level,
    traits::Crud,
};
use tinyboards_db_views::structs::{LocalUserView, PersonView};
use tinyboards_utils::{error::TinyBoardsError, passhash::verify_password};

#[async_trait::async_trait(?Send)]
impl<'des> Perform<'des> for AddAdmin {
    type Response = AddAdminResponse;
    type Route = ();

    #[tracing::instrument(skip(context, auth))]
    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        _: Self::Route,
        auth: Option<&str>,
    ) -> Result<Self::Response, TinyBoardsError> {
        let data: &AddAdmin = &self;

        // require admin to add new admin
        // TODO - reconfigure this logic to only allow site owner to add new admin
        let view = require_user(context.pool(), context.master_key(), auth)
            .await
            .require_admin(AdminPerms::Full)
            .unwrap()?;

        let level = data.level;
        let target_name = &data.username;
        let password = data.password.as_ref();

        // only the owner can add an admin with full permissions, or transfer ownership
        let can_add_full_perms = view.local_user.is_owner();
        if !can_add_full_perms
            & ((level & AdminPerms::Full + AdminPerms::Owner + AdminPerms::System.as_i32()) > 0)
        {
            return Err(TinyBoardsError::from_message(
                403,
                "You cannot add an admin with equal or higher permissions.",
            ));
        }

        // ownership transfer
        if can_add_full_perms && (level & AdminPerms::Owner.as_i32() > 0) {
            // this requires password confirmation
            if !verify_password(
                &view.local_user.passhash,
                password.unwrap_or(&String::new()),
            ) {
                return Err(TinyBoardsError::from_message(403, "Invalid password"));
            }

            // demote the logged in user (former owner) to only full permissions from owner
            LocalUser::update_admin(
                context.pool(),
                view.local_user.id,
                AdminPerms::Full.as_i32(),
            )
            .await?;

            Person::update_admin(context.pool(), view.person.id, AdminPerms::Full.as_i32()).await?;
        }

        // get the user to be updated
        let target_user_view = LocalUserView::get_by_name(context.pool(), target_name).await?;

        // update added user to be an admin
        LocalUser::update_admin(
            context.pool(),
            target_user_view.local_user.id,
            level.clone(),
        )
        .await?;
        // update added person to be an admin
        Person::update_admin(
            context.pool(),
            target_user_view.person.id.clone(),
            level.clone(),
        )
        .await?;

        // log this mod action
        let mod_add_admin_form = ModAddAdminForm {
            mod_person_id: view.person.id,
            other_person_id: target_user_view.person.id,
            removed: Some(Some(level.clone() == 0)),
        };

        // submit to the mod log
        ModAddAdmin::create(context.pool(), &mod_add_admin_form).await?;

        // send notification
        let message = String::from(if level > 0 {
            "👑 You have been promoted and are now an **admin** of this instance. Welcome to the team."
        } else {
            "❌ You are no longer an admin."
        });
        send_system_message(
            context.pool(),
            Some(target_user_view.person.id),
            None,
            message,
        )
        .await?;

        // get list of admins
        let admins = PersonView::admins(context.pool()).await?;

        Ok(AddAdminResponse { admins })
    }
}
