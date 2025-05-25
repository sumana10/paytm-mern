use actix_web::{web, App, HttpResponse, HttpServer};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Starting minimal test server at http://127.0.0.1:3000");
    
    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(|| async { 
                println!("Root endpoint called");
                HttpResponse::Ok().body("Hello world") 
            }))
    })
    .bind("127.0.0.1:3000")?
    .run()
    .await
}
