#[cfg(test)]
mod flair_template_tests {
    use crate::{
        models::flair::flair_template::{FlairTemplate, FlairTemplateForm},
        traits::Crud,
        utils::DbPool,
    };
    use serial_test::serial;

    async fn setup_test_board(pool: &DbPool) -> i32 {
        use crate::models::board::{Board, BoardForm};
        use crate::utils::naive_now;

        let board_form = BoardForm {
            name: Some("testboard".to_string()),
            title: Some("Test Board".to_string()),
            description: Some(Some("Test Description".to_string())),
            creator_id: Some(1),
            creation_date: Some(naive_now()),
            ..Default::default()
        };

        Board::create(pool, &board_form)
            .await
            .expect("Failed to create test board")
            .id
    }

    async fn cleanup_board(pool: &DbPool, board_id: i32) {
        use crate::models::board::Board;
        let _ = Board::delete(pool, board_id).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_create_flair_template() {
        let pool = &crate::utils::build_db_pool_for_tests();
        let board_id = setup_test_board(pool).await;

        let flair_form = FlairTemplateForm {
            board_id: Some(board_id),
            text: Some("Discussion".to_string()),
            text_color: Some(Some("#FFFFFF".to_string())),
            background_color: Some(Some("#3B82F6".to_string())),
            css_class: Some(None),
            flair_type: Some("post".to_string()),
            mod_only: Some(false),
            max_emojis: Some(Some(3)),
            is_editable: Some(true),
            allow_user_text: Some(false),
            creation_date: Some(crate::utils::naive_now()),
            updated: Some(None),
            is_deleted: Some(false),
            display_order: Some(0),
        };

        let result = FlairTemplate::create(pool, &flair_form).await;
        assert!(result.is_ok(), "Failed to create flair template: {:?}", result.err());

        let flair = result.unwrap();
        assert_eq!(flair.text, "Discussion");
        assert_eq!(flair.background_color, Some("#3B82F6".to_string()));
        assert_eq!(flair.flair_type, "post");
        assert!(!flair.mod_only);

        // Cleanup
        let _ = FlairTemplate::delete(pool, flair.id).await;
        cleanup_board(pool, board_id).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_create_mod_only_flair() {
        let pool = &crate::utils::build_db_pool_for_tests();
        let board_id = setup_test_board(pool).await;

        let flair_form = FlairTemplateForm {
            board_id: Some(board_id),
            text: Some("Official".to_string()),
            text_color: Some(Some("#FFFFFF".to_string())),
            background_color: Some(Some("#DC2626".to_string())),
            css_class: Some(None),
            flair_type: Some("post".to_string()),
            mod_only: Some(true),
            max_emojis: Some(None),
            is_editable: Some(false),
            allow_user_text: Some(false),
            creation_date: Some(crate::utils::naive_now()),
            updated: Some(None),
            is_deleted: Some(false),
            display_order: Some(1),
        };

        let result = FlairTemplate::create(pool, &flair_form).await;
        assert!(result.is_ok());

        let flair = result.unwrap();
        assert!(flair.mod_only, "Flair should be mod-only");

        // Cleanup
        let _ = FlairTemplate::delete(pool, flair.id).await;
        cleanup_board(pool, board_id).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_get_flair_templates_by_board() {
        let pool = &crate::utils::build_db_pool_for_tests();
        let board_id = setup_test_board(pool).await;

        // Create multiple flair templates
        let flairs_data = vec![
            ("Discussion", "post", false, 0),
            ("Question", "post", false, 1),
            ("Official", "post", true, 2),
        ];

        let mut created_ids = Vec::new();

        for (text, flair_type, mod_only, order) in flairs_data {
            let form = FlairTemplateForm {
                board_id: Some(board_id),
                text: Some(text.to_string()),
                background_color: Some(Some("#3B82F6".to_string())),
                flair_type: Some(flair_type.to_string()),
                mod_only: Some(mod_only),
                display_order: Some(order),
                creation_date: Some(crate::utils::naive_now()),
                is_deleted: Some(false),
                ..Default::default()
            };

            let flair = FlairTemplate::create(pool, &form).await.unwrap();
            created_ids.push(flair.id);
        }

        // Test getting all post flairs
        let result = FlairTemplate::get_by_board(pool, board_id, Some("post")).await;
        assert!(result.is_ok());
        let flairs = result.unwrap();
        assert_eq!(flairs.len(), 3, "Should have 3 post flairs");

        // Verify ordering
        assert_eq!(flairs[0].text, "Discussion");
        assert_eq!(flairs[1].text, "Question");
        assert_eq!(flairs[2].text, "Official");

        // Cleanup
        for id in created_ids {
            let _ = FlairTemplate::delete(pool, id).await;
        }
        cleanup_board(pool, board_id).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_get_user_assignable_flairs() {
        let pool = &crate::utils::build_db_pool_for_tests();
        let board_id = setup_test_board(pool).await;

        // Create mix of mod-only and user-assignable flairs
        let user_flair = FlairTemplateForm {
            board_id: Some(board_id),
            text: Some("Discussion".to_string()),
            flair_type: Some("post".to_string()),
            mod_only: Some(false),
            creation_date: Some(crate::utils::naive_now()),
            is_deleted: Some(false),
            ..Default::default()
        };

        let mod_flair = FlairTemplateForm {
            board_id: Some(board_id),
            text: Some("Official".to_string()),
            flair_type: Some("post".to_string()),
            mod_only: Some(true),
            creation_date: Some(crate::utils::naive_now()),
            is_deleted: Some(false),
            ..Default::default()
        };

        let user_id = FlairTemplate::create(pool, &user_flair).await.unwrap().id;
        let mod_id = FlairTemplate::create(pool, &mod_flair).await.unwrap().id;

        // Get user-assignable flairs
        let result = FlairTemplate::get_user_assignable(pool, board_id, "post").await;
        assert!(result.is_ok());
        let flairs = result.unwrap();

        assert_eq!(flairs.len(), 1, "Should only have 1 user-assignable flair");
        assert_eq!(flairs[0].text, "Discussion");
        assert!(!flairs[0].mod_only);

        // Cleanup
        let _ = FlairTemplate::delete(pool, user_id).await;
        let _ = FlairTemplate::delete(pool, mod_id).await;
        cleanup_board(pool, board_id).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_get_mod_only_flairs() {
        let pool = &crate::utils::build_db_pool_for_tests();
        let board_id = setup_test_board(pool).await;

        // Create mix of mod-only and user-assignable flairs
        let user_flair = FlairTemplateForm {
            board_id: Some(board_id),
            text: Some("Discussion".to_string()),
            flair_type: Some("post".to_string()),
            mod_only: Some(false),
            creation_date: Some(crate::utils::naive_now()),
            is_deleted: Some(false),
            ..Default::default()
        };

        let mod_flair = FlairTemplateForm {
            board_id: Some(board_id),
            text: Some("Official".to_string()),
            flair_type: Some("post".to_string()),
            mod_only: Some(true),
            creation_date: Some(crate::utils::naive_now()),
            is_deleted: Some(false),
            ..Default::default()
        };

        let user_id = FlairTemplate::create(pool, &user_flair).await.unwrap().id;
        let mod_id = FlairTemplate::create(pool, &mod_flair).await.unwrap().id;

        // Get mod-only flairs
        let result = FlairTemplate::get_mod_only(pool, board_id, "post").await;
        assert!(result.is_ok());
        let flairs = result.unwrap();

        assert_eq!(flairs.len(), 1, "Should only have 1 mod-only flair");
        assert_eq!(flairs[0].text, "Official");
        assert!(flairs[0].mod_only);

        // Cleanup
        let _ = FlairTemplate::delete(pool, user_id).await;
        let _ = FlairTemplate::delete(pool, mod_id).await;
        cleanup_board(pool, board_id).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_soft_delete_flair() {
        let pool = &crate::utils::build_db_pool_for_tests();
        let board_id = setup_test_board(pool).await;

        let flair_form = FlairTemplateForm {
            board_id: Some(board_id),
            text: Some("Test".to_string()),
            flair_type: Some("post".to_string()),
            creation_date: Some(crate::utils::naive_now()),
            is_deleted: Some(false),
            ..Default::default()
        };

        let flair = FlairTemplate::create(pool, &flair_form).await.unwrap();
        assert!(!flair.is_deleted);

        // Soft delete
        let result = FlairTemplate::soft_delete(pool, flair.id).await;
        assert!(result.is_ok());
        let deleted = result.unwrap();
        assert!(deleted.is_deleted, "Flair should be soft deleted");

        // Verify it doesn't appear in active queries
        let active_result = FlairTemplate::get_active(pool, flair.id).await;
        assert!(active_result.is_err(), "Soft deleted flair should not be retrievable as active");

        // Cleanup
        let _ = FlairTemplate::delete(pool, flair.id).await;
        cleanup_board(pool, board_id).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_reorder_flair() {
        let pool = &crate::utils::build_db_pool_for_tests();
        let board_id = setup_test_board(pool).await;

        let flair_form = FlairTemplateForm {
            board_id: Some(board_id),
            text: Some("Test".to_string()),
            flair_type: Some("post".to_string()),
            display_order: Some(5),
            creation_date: Some(crate::utils::naive_now()),
            is_deleted: Some(false),
            ..Default::default()
        };

        let flair = FlairTemplate::create(pool, &flair_form).await.unwrap();
        assert_eq!(flair.display_order, 5);

        // Reorder
        let result = FlairTemplate::reorder(pool, flair.id, 10).await;
        assert!(result.is_ok());
        let reordered = result.unwrap();
        assert_eq!(reordered.display_order, 10, "Order should be updated");

        // Cleanup
        let _ = FlairTemplate::delete(pool, flair.id).await;
        cleanup_board(pool, board_id).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_can_user_assign_permissions() {
        let pool = &crate::utils::build_db_pool_for_tests();
        let board_id = setup_test_board(pool).await;

        // Create user-assignable flair
        let user_flair = FlairTemplateForm {
            board_id: Some(board_id),
            text: Some("Discussion".to_string()),
            flair_type: Some("post".to_string()),
            mod_only: Some(false),
            creation_date: Some(crate::utils::naive_now()),
            is_deleted: Some(false),
            ..Default::default()
        };

        // Create mod-only flair
        let mod_flair = FlairTemplateForm {
            board_id: Some(board_id),
            text: Some("Official".to_string()),
            flair_type: Some("post".to_string()),
            mod_only: Some(true),
            creation_date: Some(crate::utils::naive_now()),
            is_deleted: Some(false),
            ..Default::default()
        };

        let user_id = FlairTemplate::create(pool, &user_flair).await.unwrap().id;
        let mod_id = FlairTemplate::create(pool, &mod_flair).await.unwrap().id;

        // Regular user should be able to assign user flair
        let can_assign_user = FlairTemplate::can_user_assign(pool, user_id, false).await;
        assert!(can_assign_user.is_ok());
        assert!(can_assign_user.unwrap(), "Regular user should assign user flair");

        // Regular user should NOT be able to assign mod flair
        let can_assign_mod = FlairTemplate::can_user_assign(pool, mod_id, false).await;
        assert!(can_assign_mod.is_ok());
        assert!(!can_assign_mod.unwrap(), "Regular user cannot assign mod flair");

        // Mod should be able to assign both
        let mod_assign_user = FlairTemplate::can_user_assign(pool, user_id, true).await;
        assert!(mod_assign_user.unwrap());

        let mod_assign_mod = FlairTemplate::can_user_assign(pool, mod_id, true).await;
        assert!(mod_assign_mod.unwrap());

        // Cleanup
        let _ = FlairTemplate::delete(pool, user_id).await;
        let _ = FlairTemplate::delete(pool, mod_id).await;
        cleanup_board(pool, board_id).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_update_flair_template() {
        let pool = &crate::utils::build_db_pool_for_tests();
        let board_id = setup_test_board(pool).await;

        let flair_form = FlairTemplateForm {
            board_id: Some(board_id),
            text: Some("Original".to_string()),
            background_color: Some(Some("#3B82F6".to_string())),
            flair_type: Some("post".to_string()),
            creation_date: Some(crate::utils::naive_now()),
            is_deleted: Some(false),
            ..Default::default()
        };

        let flair = FlairTemplate::create(pool, &flair_form).await.unwrap();

        // Update
        let update_form = FlairTemplateForm {
            text: Some("Updated".to_string()),
            background_color: Some(Some("#DC2626".to_string())),
            updated: Some(Some(crate::utils::naive_now())),
            ..Default::default()
        };

        let result = FlairTemplate::update(pool, flair.id, &update_form).await;
        assert!(result.is_ok());
        let updated = result.unwrap();
        assert_eq!(updated.text, "Updated");
        assert_eq!(updated.background_color, Some("#DC2626".to_string()));

        // Cleanup
        let _ = FlairTemplate::delete(pool, flair.id).await;
        cleanup_board(pool, board_id).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_flair_type_filtering() {
        let pool = &crate::utils::build_db_pool_for_tests();
        let board_id = setup_test_board(pool).await;

        // Create both post and user flairs
        let post_flair = FlairTemplateForm {
            board_id: Some(board_id),
            text: Some("Post Flair".to_string()),
            flair_type: Some("post".to_string()),
            creation_date: Some(crate::utils::naive_now()),
            is_deleted: Some(false),
            ..Default::default()
        };

        let user_flair = FlairTemplateForm {
            board_id: Some(board_id),
            text: Some("User Flair".to_string()),
            flair_type: Some("user".to_string()),
            creation_date: Some(crate::utils::naive_now()),
            is_deleted: Some(false),
            ..Default::default()
        };

        let post_id = FlairTemplate::create(pool, &post_flair).await.unwrap().id;
        let user_id = FlairTemplate::create(pool, &user_flair).await.unwrap().id;

        // Get only post flairs
        let post_flairs = FlairTemplate::get_by_board(pool, board_id, Some("post")).await.unwrap();
        assert_eq!(post_flairs.len(), 1);
        assert_eq!(post_flairs[0].flair_type, "post");

        // Get only user flairs
        let user_flairs = FlairTemplate::get_by_board(pool, board_id, Some("user")).await.unwrap();
        assert_eq!(user_flairs.len(), 1);
        assert_eq!(user_flairs[0].flair_type, "user");

        // Get all flairs
        let all_flairs = FlairTemplate::get_by_board(pool, board_id, None).await.unwrap();
        assert_eq!(all_flairs.len(), 2);

        // Cleanup
        let _ = FlairTemplate::delete(pool, post_id).await;
        let _ = FlairTemplate::delete(pool, user_id).await;
        cleanup_board(pool, board_id).await;
    }
}
