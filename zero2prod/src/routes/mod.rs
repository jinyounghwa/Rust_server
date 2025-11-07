mod health_check;
mod subscriptions;

pub use health_check::health_check;
pub use subscriptions::subscribe;

// greet 함수를 직접 정의
use actix_web::Responder;

pub async fn greet(req: actix_web::HttpRequest) -> impl Responder {
    let name = req.match_info().get("name").unwrap_or("World");
    format!("Hello {}!", name)
}