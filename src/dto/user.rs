use serde::{Deserialize, Serialize};

use crate::models::{enums::UserRole, user::User};

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub email: String,
    pub password: String,
    pub name: String,
    pub role: UserRole,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    pub email: Option<String>,
    pub name: Option<String>,
    pub password: Option<String>,
    pub role: Option<UserRole>,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: i64,
    pub email: String,
    pub name: String,
    pub role: UserRole,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            email: user.email,
            name: user.name,
            role: user.role,
        }
    }
}

impl From<&str> for UserRole {
    fn from(role: &str) -> Self {
        match role {
            "ADMIN" => UserRole::Admin,
            "PROFESSOR" => UserRole::Professor,
            "ASSISTANT" => UserRole::Assistant,
            "STAFF" => UserRole::Staff,
            _ => UserRole::Staff,
        }
    }
}
