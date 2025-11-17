pub mod basic_auth;
pub mod form_login;
pub mod pat;
pub mod oauth2;
pub mod two_factor;

pub use basic_auth::BasicAuthProvider;
pub use form_login::FormLoginProvider;
pub use pat::PatProvider;
pub use oauth2::OAuth2Provider;
pub use two_factor::TwoFactorAuthProvider;

