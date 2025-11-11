use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error,
};
use futures::future::LocalBoxFuture;
use std::rc::Rc;
use std::time::Instant;
use log::info;

/// 커스텀 Logger 미들웨어
/// HTTP 요청과 응답의 세부 정보를 로깅합니다.
pub struct LoggerMiddleware;

impl<S, B> Transform<S, ServiceRequest> for LoggerMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = LoggerMiddlewareService<S>;
    type Future = std::future::Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        std::future::ready(Ok(LoggerMiddlewareService {
            service: Rc::new(service),
        }))
    }
}

pub struct LoggerMiddlewareService<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for LoggerMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let start_time = Instant::now();
        let method = req.method().to_string();
        let path = req.path().to_string();
        let query = req.query_string().to_string();

        // 요청 정보 로깅
        info!("Request started: {} {}", method, path);
        if !query.is_empty() {
            info!("Query string: {}", query);
        }

        let service = self.service.clone();

        Box::pin(async move {
            let res = service.call(req).await?;

            let elapsed = start_time.elapsed();
            let status = res.status();

            // 응답 정보 로깅
            info!(
                "Request completed: {} {} - Status: {} ({}ms)",
                method,
                path,
                status.as_u16(),
                elapsed.as_millis()
            );

            Ok(res)
        })
    }
}
