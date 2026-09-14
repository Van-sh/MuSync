use serde::Deserialize;

#[derive(Deserialize)]
pub struct AuthBody {
    pub username: String,
    pub password: String,
}
