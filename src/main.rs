use std::sync::{Arc, Mutex};

use actix_web::{
    get, post,
    web::Data,
    web::{self, Redirect},
    App, HttpResponse, HttpServer, Responder,
};
use serde::Deserialize;
use shorten_rs::shortener::Shortener;

#[derive(Debug, Clone)]
struct State {
    domain: Arc<Mutex<Shortener>>,
}

#[derive(Deserialize)]
struct GenerateRequest {
    url: String,
}

#[get("/error")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[get("/{short_id}")]
async fn redirect(path: web::Path<String>, data: web::Data<State>) -> impl Responder {
    let short_id = path.into_inner();
    println!("{}", short_id);
    match data.domain.lock().unwrap().read(short_id) {
        Ok(shorten) => Redirect::to(shorten.url),
        Err(err) => Redirect::to(format!("/error?msg={}", err)),
    }
}

#[post("/generate")]
async fn generate(data: web::Data<State>, info: web::Json<GenerateRequest>) -> impl Responder {
    match data.domain.lock().unwrap().store(info.url.clone()) {
        Ok(shorten) => HttpResponse::Ok().body(shorten.target_url),
        Err(err) => HttpResponse::InternalServerError().body(err),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let domain = Arc::new(Mutex::new(Shortener::new(
        "http://localhost:8080".to_owned(),
    )));
    let state = State { domain };
    let server = HttpServer::new(move || {
        let app_state = Data::new(state.clone());

        App::new()
            .app_data(app_state)
            .service(hello)
            .service(generate)
            .service(redirect)
    })
    .bind(("0.0.0.0", 8080))?
    .run();

    println!("🚀 application is running on http://localhost:8080");

    server.await
}
