/// Authentication module
///
/// Handles JWT token generation/validation, password hashing,
/// and refresh token management.

mod jwt;
mod password;
mod claims;
mod refresh_token;

pub use jwt::generate_access_token;
pub use jwt::validate_access_token;
pub use password::hash_password;
pub use password::verify_password;
pub use claims::Claims;
pub use refresh_token::generate_refresh_token;
pub use refresh_token::save_refresh_token;
pub use refresh_token::validate_refresh_token;
pub use refresh_token::revoke_refresh_token;
