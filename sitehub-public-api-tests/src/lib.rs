//! API tests for the public REST API.
//! These tests run against a live server and `SurrealDB` instance.

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn health_check_returns_ok() {
        let resp = reqwest::get("http://localhost:3000/api/health")
            .await
            .expect("failed to reach server");

        assert_eq!(resp.status(), 200);

        let body: serde_json::Value = resp.json().await.expect("invalid json");
        assert_eq!(body["status"], "ok");
    }
}
