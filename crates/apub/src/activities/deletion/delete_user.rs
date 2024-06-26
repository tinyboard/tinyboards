use crate::{
    activities::{generate_activity_id, send_tinyboards_activity, verify_is_public, verify_person},
    insert_activity,
    objects::{instance::remote_instance_inboxes, person::ApubPerson},
    protocol::activities::deletion::delete_user::DeleteUser,
    SendActivity,
};
use tinyboards_api_common::{
    data::TinyBoardsContext,
    person::{DeleteAccount, DeleteAccountResponse},
    utils::{delete_user_account, require_user},
};
use tinyboards_federation::{
    config::Data,
    kinds::{activity::DeleteType, public},
    protocol::verification::verify_urls_match,
    traits::{ActivityHandler, Actor},
};
use tinyboards_utils::error::TinyBoardsError;
use url::Url;

#[async_trait::async_trait]
impl SendActivity for DeleteAccount {
    type Response = DeleteAccountResponse;
    type Route = ();

    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        context: &Data<TinyBoardsContext>,
        _: &Self::Route,
        auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> {
        let view = require_user(context.pool(), context.master_key(), auth)
            .await
            .unwrap()?;
        let actor: ApubPerson = view.person.into();
        delete_user_account(actor.id, context.pool()).await?;

        let id = generate_activity_id(
            DeleteType::Delete,
            &context.settings().get_protocol_and_hostname(),
        )?;
        let delete = DeleteUser {
            actor: actor.id().into(),
            to: vec![public()],
            object: actor.id().into(),
            kind: DeleteType::Delete,
            id: id.clone(),
            cc: vec![],
        };

        let inboxes = remote_instance_inboxes(context.pool()).await?;
        send_tinyboards_activity(context, delete, &actor, inboxes, true).await?;
        Ok(())
    }
}

/// This can be separate from Delete activity because it doesn't need to be handled in shared inbox
/// (cause instance actor doesn't have shared inbox).
#[async_trait::async_trait]
impl ActivityHandler for DeleteUser {
    type DataType = TinyBoardsContext;
    type Error = TinyBoardsError;

    fn id(&self) -> &Url {
        &self.id
    }

    fn actor(&self) -> &Url {
        self.actor.inner()
    }

    async fn verify(&self, context: &Data<Self::DataType>) -> Result<(), TinyBoardsError> {
        verify_is_public(&self.to, &[])?;
        verify_person(&self.actor, context).await?;
        verify_urls_match(self.actor.inner(), self.object.inner())?;
        Ok(())
    }

    async fn receive(self, context: &Data<Self::DataType>) -> Result<(), TinyBoardsError> {
        insert_activity(&self.id, &self, false, false, context).await?;
        let actor = self.actor.dereference(context).await?;
        delete_user_account(actor.id, context.pool()).await?;
        Ok(())
    }
}
