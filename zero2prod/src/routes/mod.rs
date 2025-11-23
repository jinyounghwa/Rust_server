mod health_check;
mod subscriptions;
mod confirmation;
mod newsletters;

pub use health_check::health_check;
pub use subscriptions::subscribe;
pub use confirmation::confirm_subscription;
pub use newsletters::{send_newsletter_to_all, send_newsletter_to_confirmed};

// greet 함수를 직접 정의
use actix_web::Responder;

pub async fn greet(req: actix_web::HttpRequest) -> impl Responder {
    let name = req.match_info().get("name").unwrap_or("World");
    format!("Hello {}!", name)
}