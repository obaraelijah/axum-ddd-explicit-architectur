#[cfg(test)]
mod tests {
    use crate::{
        config::connect::connect_test, handler::{CreateCircleRequestBody, CreateCircleResponseBody, UpdateCircleRequestBody}, router, AppState
    };
    use axum::{http::{header::CONTENT_TYPE, StatusCode}, Router};
    use domain::{
        aggregate::{
            circle::Circle,
            member::Member,
            value_object::{circle_id::CircleId, grade::Grade, major::Major, member_id::MemberId},
        },
        interface::circle_repository_interface::CircleRepositoryInterface,
    };
    use infrastructure::circle_repository_with_my_sql::CircleRepositoryWithMySql;
    use tower::ServiceExt;

    // TODO: ignore test because it requires a running database
    #[tokio::test]
    #[ignore]
    async fn test_version() -> anyhow::Result<()> {
        let pool = connect_test().await.expect("database should connect");
        let state = AppState {
            circle_repository: CircleRepositoryWithMySql::new(pool.clone()),
            pool,
        };
        let app = router().with_state(state);
        let response = app
            .oneshot(
                axum::http::Request::builder()
                    .method("GET")
                    .uri("/")
                    .body(axum::body::Body::empty())?,
            )
            .await?;
        assert_eq!(response.status(), StatusCode::OK);
        let response_body = String::from_utf8(
            axum::body::to_bytes(response.into_body(), usize::MAX)
                .await?
                .to_vec(),
        )?;
        assert_eq!(response_body, "0.1.0");
        Ok(())
    }

    #[tokio::test]
    #[ignore]
    async fn test_create_circle() -> anyhow::Result<()> {
        let pool = connect_test().await.expect("database should connect");
        let state = AppState {
            circle_repository: CircleRepositoryWithMySql::new(pool.clone()),
            pool,
        };
        let app = router().with_state(state.clone());
        let response = app
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/circle")
                    .header(CONTENT_TYPE, "application/json")
                    .body(axum::body::Body::new(serde_json::to_string(
                        &CreateCircleRequestBody {
                            circle_name: "circle_name1".to_string(),
                            capacity: 10,
                            owner_name: "owner1".to_string(),
                            owner_age: 21,
                            owner_grade: 3,
                            owner_major: "Music".to_string(),
                        },
                    )?))?,
            )
            .await?;
        assert_eq!(response.status(), StatusCode::OK);
        let response_body = serde_json::from_slice::<'_, CreateCircleResponseBody>(
            &axum::body::to_bytes(response.into_body(), usize::MAX).await?,
        )?;

        let created = state
            .circle_repository
            .find_by_id(&CircleId::from(response_body.circle_id))
            .await?;
        let circle = Circle::reconstruct(
            CircleId::from(response_body.circle_id),
            "circle_name1".to_string(),
            Member::reconstruct(
                MemberId::from(response_body.owner_id),
                "owner1".to_string(),
                21,
                Grade::try_from(3)?,
                Major::Music,
            ),
            10,
            vec![],
        );
        assert_eq!(created, circle);
        Ok(())
    }

    #[tokio::test]
    #[ignore]
    async fn test_fetch_circle() -> anyhow::Result<()> {
        let pool = connect_test().await.expect("database should connect");
        let state = AppState {
            circle_repository: CircleRepositoryWithMySql::new(pool.clone()),
            pool,
        };
        let app = router().with_state(state);
        let unexist_circle_id = 0;
        let response = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("GET")
                    .uri(format!("/circle/{}", unexist_circle_id))
                    .body(axum::body::Body::empty())?,
            )
            .await?;
        assert_eq!(response.status(), StatusCode::OK);
        let response_body = String::from_utf8(
            axum::body::to_bytes(response.into_body(), usize::MAX)
                .await?
                .to_vec(),
        )?;
        assert_eq!(response_body, "Circle not found");

        let (circle_id, owner_id) = build_circle(&app).await?;

        let fetched_response = app
            .oneshot(
                axum::http::Request::builder()
                    .method("GET")
                    .uri(format!("/circle/{}", circle_id))
                    .body(axum::body::Body::empty())?,
            )
            .await?;
        assert_eq!(fetched_response.status(), StatusCode::OK);
        let fetched_response_body = String::from_utf8(
            axum::body::to_bytes(fetched_response.into_body(), usize::MAX)
                .await?
                .to_vec(),
        )?;
        assert_eq!(
            fetched_response_body,
            format!(
                "{{\"circle_id\":{},\"circle_name\":\"Music club\",\"capacity\":10,\"owner\":{{\"id\":{},\"name\":\"John Lennon\",\"age\":21,\"grade\":3,\"major\":\"Music\"}},\"members\":[]}}",
                circle_id,owner_id
            )
        );
        Ok(())
    }

    #[tokio::test]
    #[ignore]
    async fn test_update_circle() -> anyhow::Result<()> {
        let pool = connect_test().await.expect("database should connect");
        let state = AppState {
            circle_repository: CircleRepositoryWithMySql::new(pool.clone()),
            pool,
        };
        let app = router().with_state(state.clone());
        let (circle_id, _) = build_circle(&app).await?;
        let update_response = app
            .oneshot(
                axum::http::Request::builder()
                    .method("PUT")
                    .uri(format!("/circle/{}", circle_id))
                    .header(CONTENT_TYPE, "application/json")
                    .body(axum::body::Body::new(serde_json::to_string(
                        &UpdateCircleRequestBody {
                            circle_name: Some("Football club".to_string()),
                            capacity: Some(20),
                        },
                    )?))?,
            )
            .await?;
        assert_eq!(update_response.status(), StatusCode::OK);

        let updated_circle = state
            .circle_repository
            .find_by_id(&CircleId::from(circle_id))
            .await?;
        assert_eq!(updated_circle.name, "Football club");
        assert_eq!(updated_circle.capacity, 20);

        Ok(())
    }

    async fn build_circle(app: &Router) -> anyhow::Result<(i16, i16)> {
        let create_response = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/circle")
                    .header(CONTENT_TYPE, "application/json")
                    .body(axum::body::Body::new(serde_json::to_string(
                        &CreateCircleRequestBody {
                            circle_name: "Music club".to_string(),
                            capacity: 10,
                            owner_name: "John Lennon".to_string(),
                            owner_age: 21,
                            owner_grade: 3,
                            owner_major: "Music".to_string(),
                        },
                    )?))?,
            )
            .await?;
        assert_eq!(create_response.status(), StatusCode::OK);
        let create_response_body = serde_json::from_slice::<CreateCircleResponseBody>(
            &axum::body::to_bytes(create_response.into_body(), usize::MAX).await?,
        )?;

        Ok((
            create_response_body.circle_id,
            create_response_body.owner_id,
        ))
    }
}
