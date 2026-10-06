use better_auth::AuthSchema;
use entity::{accounts, sessions, users, verifications};

pub struct AppAuthSchema;

impl AuthSchema for AppAuthSchema {
    type User = users::Model;
    type Session = sessions::Model;
    type Account = accounts::Model;
    type Verification = verifications::Model;
}
