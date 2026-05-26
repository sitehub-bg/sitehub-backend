//! Driving adapter: public REST API consumed by school static websites.
//! Tenant identified by subdomain. No user authentication.

use axum::Router;

pub fn router() -> Router {
    Router::new()
}
