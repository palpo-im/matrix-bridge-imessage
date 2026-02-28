use std::sync::Arc;

use anyhow::Result;

use crate::db::DatabaseManager;
use crate::imessage::{Contact, IMessageClient};
use crate::matrix::MatrixAppservice;

pub struct UserSync {
    matrix_client: Arc<MatrixAppservice>,
    imessage_client: Arc<IMessageClient>,
    db_manager: Arc<DatabaseManager>,
}

impl UserSync {
    pub fn new(
        matrix_client: Arc<MatrixAppservice>,
        imessage_client: Arc<IMessageClient>,
        db_manager: Arc<DatabaseManager>,
    ) -> Self {
        Self {
            matrix_client,
            imessage_client,
            db_manager,
        }
    }

    pub async fn sync_user(&self, contact: &Contact) -> Result<()> {
        let user_id = self.matrix_client.ghost_user_id(
            contact.user_guid.as_deref().unwrap_or(&contact.primary_identifier.clone().unwrap_or_default())
        );

        // Set display name
        if contact.has_name() {
            self.matrix_client
                .set_user_displayname(&user_id, &contact.name())
                .await?;
        }

        Ok(())
    }
}
