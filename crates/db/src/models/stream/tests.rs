#[cfg(test)]
mod stream_tests {
    use crate::{
        models::stream::stream::{CreateStreamForm, Stream, StreamSortType, StreamTimeRange, UpdateStreamForm},
        traits::Crud,
        utils::DbPool,
    };
    use serial_test::serial;

    async fn setup_test_user(pool: &DbPool) -> i32 {
        use crate::models::person::{Person, PersonForm};

        let user_form = PersonForm {
            name: Some("testuser".to_string()),
            password_encrypted: Some("hashed_password".to_string()),
            email: Some(Some("test@example.com".to_string())),
            is_banned: Some(false),
            creation_date: Some(crate::utils::naive_now()),
            ..Default::default()
        };

        Person::create(pool, &user_form)
            .await
            .expect("Failed to create test user")
            .id
    }

    async fn cleanup_user(pool: &DbPool, user_id: i32) {
        use crate::models::person::Person;
        let _ = Person::delete(pool, user_id).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_create_stream() {
        let pool = &crate::utils::build_db_pool_for_tests();
        let user_id = setup_test_user(pool).await;

        let stream_form = CreateStreamForm {
            creator_id: user_id,
            name: "Tech News".to_string(),
            slug: "tech-news".to_string(),
            description: Some("Latest technology news".to_string()),
            is_public: true,
            share_token: None,
            default_sort: StreamSortType::Hot.to_string(),
            default_time_range: StreamTimeRange::All.to_string(),
        };

        let result = Stream::create(pool, &stream_form).await;
        assert!(result.is_ok(), "Failed to create stream: {:?}", result.err());

        let stream = result.unwrap();
        assert_eq!(stream.name, "Tech News");
        assert_eq!(stream.slug, "tech-news");
        assert!(stream.is_public);
        assert_eq!(stream.default_sort, "hot");

        // Cleanup
        let _ = Stream::delete(pool, stream.id).await;
        cleanup_user(pool, user_id).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_create_private_stream_with_share_token() {
        let pool = &crate::utils::build_db_pool_for_tests();
        let user_id = setup_test_user(pool).await;

        let share_token = Stream::generate_share_token();

        let stream_form = CreateStreamForm {
            creator_id: user_id,
            name: "Private Stream".to_string(),
            slug: "private-stream".to_string(),
            description: None,
            is_public: false,
            share_token: Some(share_token.clone()),
            default_sort: StreamSortType::New.to_string(),
            default_time_range: StreamTimeRange::Week.to_string(),
        };

        let stream = Stream::create(pool, &stream_form).await.unwrap();
        assert!(!stream.is_public);
        assert_eq!(stream.share_token, Some(share_token));

        // Cleanup
        let _ = Stream::delete(pool, stream.id).await;
        cleanup_user(pool, user_id).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_generate_share_token() {
        let token1 = Stream::generate_share_token();
        let token2 = Stream::generate_share_token();

        assert_eq!(token1.len(), 32, "Token should be 32 characters");
        assert_eq!(token2.len(), 32, "Token should be 32 characters");
        assert_ne!(token1, token2, "Tokens should be unique");
        assert!(token1.chars().all(|c| c.is_alphanumeric()), "Token should be alphanumeric");
    }

    #[tokio::test]
    #[serial]
    async fn test_generate_slug() {
        assert_eq!(Stream::generate_slug("Tech News"), "tech-news");
        assert_eq!(Stream::generate_slug("My Awesome Stream!"), "my-awesome-stream");
        assert_eq!(Stream::generate_slug("  Spaces  "), "spaces");
        assert_eq!(Stream::generate_slug("Special@#$Characters"), "specialcharacters");
        assert_eq!(Stream::generate_slug("Multiple---Dashes"), "multiple---dashes");
    }

    #[tokio::test]
    #[serial]
    async fn test_slug_uniqueness_check() {
        let pool = &crate::utils::build_db_pool_for_tests();
        let user_id = setup_test_user(pool).await;

        let stream_form = CreateStreamForm {
            creator_id: user_id,
            name: "Test Stream".to_string(),
            slug: "test-stream".to_string(),
            description: None,
            is_public: true,
            share_token: None,
            default_sort: StreamSortType::Hot.to_string(),
            default_time_range: StreamTimeRange::All.to_string(),
        };

        let stream = Stream::create(pool, &stream_form).await.unwrap();

        // Check uniqueness - should be false (slug exists)
        let is_unique = Stream::is_slug_unique(pool, user_id, "test-stream", None).await;
        assert!(is_unique.is_ok());
        assert!(!is_unique.unwrap(), "Slug should not be unique");

        // Check uniqueness for different slug
        let is_unique2 = Stream::is_slug_unique(pool, user_id, "different-slug", None).await;
        assert!(is_unique2.is_ok());
        assert!(is_unique2.unwrap(), "Different slug should be unique");

        // Check uniqueness excluding current stream
        let is_unique3 = Stream::is_slug_unique(pool, user_id, "test-stream", Some(stream.id)).await;
        assert!(is_unique3.is_ok());
        assert!(is_unique3.unwrap(), "Slug should be unique when excluding self");

        // Cleanup
        let _ = Stream::delete(pool, stream.id).await;
        cleanup_user(pool, user_id).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_user_stream_quota() {
        let pool = &crate::utils::build_db_pool_for_tests();
        let user_id = setup_test_user(pool).await;

        // Initially user should be under quota
        let has_quota = Stream::check_user_quota(pool, user_id).await;
        assert!(has_quota.is_ok());
        assert!(has_quota.unwrap(), "User should have quota available");

        // Get initial count
        let count = Stream::get_user_stream_count(pool, user_id).await;
        assert!(count.is_ok());
        assert_eq!(count.unwrap(), 0);

        // Create a stream
        let stream_form = CreateStreamForm {
            creator_id: user_id,
            name: "Test Stream".to_string(),
            slug: "test-stream".to_string(),
            description: None,
            is_public: true,
            share_token: None,
            default_sort: StreamSortType::Hot.to_string(),
            default_time_range: StreamTimeRange::All.to_string(),
        };

        let stream = Stream::create(pool, &stream_form).await.unwrap();

        // Count should increase
        let count2 = Stream::get_user_stream_count(pool, user_id).await.unwrap();
        assert_eq!(count2, 1);

        // Cleanup
        let _ = Stream::delete(pool, stream.id).await;
        cleanup_user(pool, user_id).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_get_stream_by_slug() {
        let pool = &crate::utils::build_db_pool_for_tests();
        let user_id = setup_test_user(pool).await;

        let stream_form = CreateStreamForm {
            creator_id: user_id,
            name: "Test Stream".to_string(),
            slug: "test-stream".to_string(),
            description: None,
            is_public: true,
            share_token: None,
            default_sort: StreamSortType::Hot.to_string(),
            default_time_range: StreamTimeRange::All.to_string(),
        };

        let created_stream = Stream::create(pool, &stream_form).await.unwrap();

        // Get by slug
        let result = Stream::get_by_slug(pool, user_id, "test-stream").await;
        assert!(result.is_ok());
        let stream = result.unwrap();
        assert_eq!(stream.id, created_stream.id);
        assert_eq!(stream.name, "Test Stream");

        // Try non-existent slug
        let result2 = Stream::get_by_slug(pool, user_id, "non-existent").await;
        assert!(result2.is_err());

        // Cleanup
        let _ = Stream::delete(pool, created_stream.id).await;
        cleanup_user(pool, user_id).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_get_stream_by_share_token() {
        let pool = &crate::utils::build_db_pool_for_tests();
        let user_id = setup_test_user(pool).await;

        let share_token = Stream::generate_share_token();

        let stream_form = CreateStreamForm {
            creator_id: user_id,
            name: "Shared Stream".to_string(),
            slug: "shared-stream".to_string(),
            description: None,
            is_public: false,
            share_token: Some(share_token.clone()),
            default_sort: StreamSortType::Hot.to_string(),
            default_time_range: StreamTimeRange::All.to_string(),
        };

        let created_stream = Stream::create(pool, &stream_form).await.unwrap();

        // Get by share token
        let result = Stream::get_by_share_token(pool, &share_token).await;
        assert!(result.is_ok());
        let stream = result.unwrap();
        assert_eq!(stream.id, created_stream.id);

        // Try invalid token
        let result2 = Stream::get_by_share_token(pool, "invalid_token").await;
        assert!(result2.is_err());

        // Cleanup
        let _ = Stream::delete(pool, created_stream.id).await;
        cleanup_user(pool, user_id).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_get_user_streams() {
        let pool = &crate::utils::build_db_pool_for_tests();
        let user_id = setup_test_user(pool).await;

        // Create multiple streams
        let mut stream_ids = Vec::new();
        for i in 0..3 {
            let form = CreateStreamForm {
                creator_id: user_id,
                name: format!("Stream {}", i),
                slug: format!("stream-{}", i),
                description: None,
                is_public: true,
                share_token: None,
                default_sort: StreamSortType::Hot.to_string(),
                default_time_range: StreamTimeRange::All.to_string(),
            };
            let stream = Stream::create(pool, &form).await.unwrap();
            stream_ids.push(stream.id);
        }

        // Get all user streams
        let result = Stream::get_user_streams(pool, user_id, false).await;
        assert!(result.is_ok());
        let streams = result.unwrap();
        assert_eq!(streams.len(), 3);

        // Cleanup
        for id in stream_ids {
            let _ = Stream::delete(pool, id).await;
        }
        cleanup_user(pool, user_id).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_soft_delete_stream() {
        let pool = &crate::utils::build_db_pool_for_tests();
        let user_id = setup_test_user(pool).await;

        let stream_form = CreateStreamForm {
            creator_id: user_id,
            name: "Test Stream".to_string(),
            slug: "test-stream".to_string(),
            description: None,
            is_public: true,
            share_token: None,
            default_sort: StreamSortType::Hot.to_string(),
            default_time_range: StreamTimeRange::All.to_string(),
        };

        let stream = Stream::create(pool, &stream_form).await.unwrap();
        assert!(!stream.is_deleted);

        // Soft delete
        let result = Stream::soft_delete(pool, stream.id).await;
        assert!(result.is_ok());
        let deleted = result.unwrap();
        assert!(deleted.is_deleted);

        // Should not appear in user streams
        let user_streams = Stream::get_user_streams(pool, user_id, false).await.unwrap();
        assert_eq!(user_streams.len(), 0);

        // But should appear when including deleted
        let user_streams_with_deleted = Stream::get_user_streams(pool, user_id, true).await.unwrap();
        assert_eq!(user_streams_with_deleted.len(), 1);

        // Cleanup
        let _ = Stream::delete(pool, stream.id).await;
        cleanup_user(pool, user_id).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_restore_stream() {
        let pool = &crate::utils::build_db_pool_for_tests();
        let user_id = setup_test_user(pool).await;

        let stream_form = CreateStreamForm {
            creator_id: user_id,
            name: "Test Stream".to_string(),
            slug: "test-stream".to_string(),
            description: None,
            is_public: true,
            share_token: None,
            default_sort: StreamSortType::Hot.to_string(),
            default_time_range: StreamTimeRange::All.to_string(),
        };

        let stream = Stream::create(pool, &stream_form).await.unwrap();

        // Soft delete
        let _ = Stream::soft_delete(pool, stream.id).await;

        // Restore
        let result = Stream::restore(pool, stream.id).await;
        assert!(result.is_ok());
        let restored = result.unwrap();
        assert!(!restored.is_deleted);

        // Should appear in user streams again
        let user_streams = Stream::get_user_streams(pool, user_id, false).await.unwrap();
        assert_eq!(user_streams.len(), 1);

        // Cleanup
        let _ = Stream::delete(pool, stream.id).await;
        cleanup_user(pool, user_id).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_update_stream_visibility() {
        let pool = &crate::utils::build_db_pool_for_tests();
        let user_id = setup_test_user(pool).await;

        let stream_form = CreateStreamForm {
            creator_id: user_id,
            name: "Test Stream".to_string(),
            slug: "test-stream".to_string(),
            description: None,
            is_public: true,
            share_token: None,
            default_sort: StreamSortType::Hot.to_string(),
            default_time_range: StreamTimeRange::All.to_string(),
        };

        let stream = Stream::create(pool, &stream_form).await.unwrap();
        assert!(stream.is_public);
        assert!(stream.share_token.is_none());

        // Make private with share token
        let share_token = Stream::generate_share_token();
        let result = Stream::update_visibility(pool, stream.id, false, Some(share_token.clone())).await;
        assert!(result.is_ok());
        let updated = result.unwrap();
        assert!(!updated.is_public);
        assert_eq!(updated.share_token, Some(share_token));

        // Cleanup
        let _ = Stream::delete(pool, stream.id).await;
        cleanup_user(pool, user_id).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_regenerate_share_token() {
        let pool = &crate::utils::build_db_pool_for_tests();
        let user_id = setup_test_user(pool).await;

        let old_token = Stream::generate_share_token();

        let stream_form = CreateStreamForm {
            creator_id: user_id,
            name: "Test Stream".to_string(),
            slug: "test-stream".to_string(),
            description: None,
            is_public: false,
            share_token: Some(old_token.clone()),
            default_sort: StreamSortType::Hot.to_string(),
            default_time_range: StreamTimeRange::All.to_string(),
        };

        let stream = Stream::create(pool, &stream_form).await.unwrap();

        // Regenerate token
        let result = Stream::regenerate_share_token(pool, stream.id).await;
        assert!(result.is_ok());
        let updated = result.unwrap();
        assert!(updated.share_token.is_some());
        assert_ne!(updated.share_token.unwrap(), old_token);

        // Cleanup
        let _ = Stream::delete(pool, stream.id).await;
        cleanup_user(pool, user_id).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_user_owns_stream() {
        let pool = &crate::utils::build_db_pool_for_tests();
        let user_id = setup_test_user(pool).await;
        let other_user_id = setup_test_user(pool).await;

        let stream_form = CreateStreamForm {
            creator_id: user_id,
            name: "Test Stream".to_string(),
            slug: "test-stream".to_string(),
            description: None,
            is_public: true,
            share_token: None,
            default_sort: StreamSortType::Hot.to_string(),
            default_time_range: StreamTimeRange::All.to_string(),
        };

        let stream = Stream::create(pool, &stream_form).await.unwrap();

        // Owner should own the stream
        let owns = Stream::user_owns_stream(pool, stream.id, user_id).await;
        assert!(owns.is_ok());
        assert!(owns.unwrap());

        // Other user should not own the stream
        let not_owns = Stream::user_owns_stream(pool, stream.id, other_user_id).await;
        assert!(not_owns.is_ok());
        assert!(!not_owns.unwrap());

        // Cleanup
        let _ = Stream::delete(pool, stream.id).await;
        cleanup_user(pool, user_id).await;
        cleanup_user(pool, other_user_id).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_user_can_access_public_stream() {
        let pool = &crate::utils::build_db_pool_for_tests();
        let user_id = setup_test_user(pool).await;
        let other_user_id = setup_test_user(pool).await;

        let stream_form = CreateStreamForm {
            creator_id: user_id,
            name: "Public Stream".to_string(),
            slug: "public-stream".to_string(),
            description: None,
            is_public: true,
            share_token: None,
            default_sort: StreamSortType::Hot.to_string(),
            default_time_range: StreamTimeRange::All.to_string(),
        };

        let stream = Stream::create(pool, &stream_form).await.unwrap();

        // Owner can access
        let can_access = Stream::user_can_access(pool, stream.id, Some(user_id), None).await;
        assert!(can_access.is_ok());
        assert!(can_access.unwrap());

        // Other user can access (public)
        let can_access2 = Stream::user_can_access(pool, stream.id, Some(other_user_id), None).await;
        assert!(can_access2.is_ok());
        assert!(can_access2.unwrap());

        // Anonymous can access (public)
        let can_access3 = Stream::user_can_access(pool, stream.id, None, None).await;
        assert!(can_access3.is_ok());
        assert!(can_access3.unwrap());

        // Cleanup
        let _ = Stream::delete(pool, stream.id).await;
        cleanup_user(pool, user_id).await;
        cleanup_user(pool, other_user_id).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_user_can_access_private_stream_with_token() {
        let pool = &crate::utils::build_db_pool_for_tests();
        let user_id = setup_test_user(pool).await;
        let other_user_id = setup_test_user(pool).await;

        let share_token = Stream::generate_share_token();

        let stream_form = CreateStreamForm {
            creator_id: user_id,
            name: "Private Stream".to_string(),
            slug: "private-stream".to_string(),
            description: None,
            is_public: false,
            share_token: Some(share_token.clone()),
            default_sort: StreamSortType::Hot.to_string(),
            default_time_range: StreamTimeRange::All.to_string(),
        };

        let stream = Stream::create(pool, &stream_form).await.unwrap();

        // Owner can access
        let can_access = Stream::user_can_access(pool, stream.id, Some(user_id), None).await;
        assert!(can_access.is_ok());
        assert!(can_access.unwrap());

        // Other user cannot access without token
        let cannot_access = Stream::user_can_access(pool, stream.id, Some(other_user_id), None).await;
        assert!(cannot_access.is_ok());
        assert!(!cannot_access.unwrap());

        // Other user can access with valid token
        let can_access_with_token = Stream::user_can_access(pool, stream.id, Some(other_user_id), Some(&share_token)).await;
        assert!(can_access_with_token.is_ok());
        assert!(can_access_with_token.unwrap());

        // Anonymous cannot access without token
        let anon_cannot = Stream::user_can_access(pool, stream.id, None, None).await;
        assert!(anon_cannot.is_ok());
        assert!(!anon_cannot.unwrap());

        // Anonymous can access with token
        let anon_can_with_token = Stream::user_can_access(pool, stream.id, None, Some(&share_token)).await;
        assert!(anon_can_with_token.is_ok());
        assert!(anon_can_with_token.unwrap());

        // Cleanup
        let _ = Stream::delete(pool, stream.id).await;
        cleanup_user(pool, user_id).await;
        cleanup_user(pool, other_user_id).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_update_stream() {
        let pool = &crate::utils::build_db_pool_for_tests();
        let user_id = setup_test_user(pool).await;

        let stream_form = CreateStreamForm {
            creator_id: user_id,
            name: "Original Name".to_string(),
            slug: "original-slug".to_string(),
            description: Some("Original description".to_string()),
            is_public: true,
            share_token: None,
            default_sort: StreamSortType::Hot.to_string(),
            default_time_range: StreamTimeRange::All.to_string(),
        };

        let stream = Stream::create(pool, &stream_form).await.unwrap();

        // Update
        let update_form = UpdateStreamForm {
            name: Some("Updated Name".to_string()),
            description: Some(Some("Updated description".to_string())),
            default_sort: Some(StreamSortType::New.to_string()),
            updated: Some(crate::utils::naive_now()),
            ..Default::default()
        };

        let result = Stream::update(pool, stream.id, &update_form).await;
        assert!(result.is_ok());
        let updated = result.unwrap();
        assert_eq!(updated.name, "Updated Name");
        assert_eq!(updated.description, Some("Updated description".to_string()));
        assert_eq!(updated.default_sort, "new");
        assert_eq!(updated.slug, "original-slug"); // Slug unchanged

        // Cleanup
        let _ = Stream::delete(pool, stream.id).await;
        cleanup_user(pool, user_id).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_get_public_streams() {
        let pool = &crate::utils::build_db_pool_for_tests();
        let user_id = setup_test_user(pool).await;

        // Create mix of public and private streams
        let mut stream_ids = Vec::new();

        for i in 0..3 {
            let form = CreateStreamForm {
                creator_id: user_id,
                name: format!("Public Stream {}", i),
                slug: format!("public-{}", i),
                description: None,
                is_public: true,
                share_token: None,
                default_sort: StreamSortType::Hot.to_string(),
                default_time_range: StreamTimeRange::All.to_string(),
            };
            let stream = Stream::create(pool, &form).await.unwrap();
            stream_ids.push(stream.id);
        }

        // Create private stream
        let private_form = CreateStreamForm {
            creator_id: user_id,
            name: "Private Stream".to_string(),
            slug: "private".to_string(),
            description: None,
            is_public: false,
            share_token: None,
            default_sort: StreamSortType::Hot.to_string(),
            default_time_range: StreamTimeRange::All.to_string(),
        };
        let private_stream = Stream::create(pool, &private_form).await.unwrap();
        stream_ids.push(private_stream.id);

        // Get public streams
        let result = Stream::get_public_streams(pool, None, None).await;
        assert!(result.is_ok());
        let public_streams = result.unwrap();
        assert_eq!(public_streams.len(), 3);
        assert!(public_streams.iter().all(|s| s.is_public));

        // Test pagination
        let limited = Stream::get_public_streams(pool, Some(2), None).await.unwrap();
        assert_eq!(limited.len(), 2);

        // Cleanup
        for id in stream_ids {
            let _ = Stream::delete(pool, id).await;
        }
        cleanup_user(pool, user_id).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_stream_sort_type() {
        assert_eq!(StreamSortType::Hot.to_string(), "hot");
        assert_eq!(StreamSortType::New.to_string(), "new");
        assert_eq!(StreamSortType::Top.to_string(), "top");
        assert_eq!(StreamSortType::Active.to_string(), "active");

        assert_eq!(StreamSortType::from_str("hot"), StreamSortType::Hot);
        assert_eq!(StreamSortType::from_str("new"), StreamSortType::New);
        assert_eq!(StreamSortType::from_str("top"), StreamSortType::Top);
        assert_eq!(StreamSortType::from_str("active"), StreamSortType::Active);
        assert_eq!(StreamSortType::from_str("invalid"), StreamSortType::Hot); // Default
    }

    #[tokio::test]
    #[serial]
    async fn test_stream_time_range() {
        assert_eq!(StreamTimeRange::Day.to_string(), "day");
        assert_eq!(StreamTimeRange::Week.to_string(), "week");
        assert_eq!(StreamTimeRange::Month.to_string(), "month");
        assert_eq!(StreamTimeRange::Year.to_string(), "year");
        assert_eq!(StreamTimeRange::All.to_string(), "all");

        assert_eq!(StreamTimeRange::from_str("day"), StreamTimeRange::Day);
        assert_eq!(StreamTimeRange::from_str("week"), StreamTimeRange::Week);
        assert_eq!(StreamTimeRange::from_str("month"), StreamTimeRange::Month);
        assert_eq!(StreamTimeRange::from_str("year"), StreamTimeRange::Year);
        assert_eq!(StreamTimeRange::from_str("all"), StreamTimeRange::All);
        assert_eq!(StreamTimeRange::from_str("invalid"), StreamTimeRange::All); // Default
    }
}
