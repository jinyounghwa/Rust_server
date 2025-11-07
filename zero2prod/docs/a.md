```rust
fn main() -> std::io::Result<()>{
    let body = async move{
        HttpServer::new(||{
            App::new()
            .route("/", web::get().to(greet))
            .route("/{name}", web::get().to(greet))
        })
        .bind("127.0.0.1:8080")?
        .run()
        .await
    };
    tokio::runtime::Builder::new_multi_thread()
    .enable_all()
    .build()
    .expect("Filed to build runtime")
    .block_on(body)
    }
```