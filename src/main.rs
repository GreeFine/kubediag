#[macro_use]
extern crate lazy_static;

use std::env;

use actix_web::middleware::Logger;
use actix_web::HttpServer;

mod deployment_status;
mod templates;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info");
    }
    pretty_env_logger::init();

    Ok(HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .service(index)
            .service(actix_files::Files::new("/static", "./static"))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await?)
}

use actix_web::{get, App, HttpResponse, Responder};
use log::error;

#[get("/")]
async fn index() -> impl Responder {
    let page_build = templates::load_index().await;
    let Ok(page) = page_build else {
        error!("{page_build:?}");
        return HttpResponse::InternalServerError().finish();
    };
    HttpResponse::Ok().body(page)
}
