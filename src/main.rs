use std::sync::{Arc, Mutex};

use actix_files as fs;
use actix_web::{
    get, post,
    web::Data,
    web::{self, Redirect},
    App, HttpResponse, HttpServer, Responder,
};
use lazy_static::lazy_static;
use serde::Deserialize;
use shorten_rs::shortener::Shortener;
use tera::{Context, Tera};

lazy_static! {
    pub static ref TEMPLATES: Tera = {
        let mut tera = match Tera::new("templates/**/*.html") {
            Ok(t) => t,
            Err(e) => {
                println!("Parsing error(s): {}", e);
                ::std::process::exit(1);
            }
        };
        tera.autoescape_on(vec![".html", ".sql"]);
        tera
    };
}

#[derive(Debug, Clone)]
struct State {
    domain: Arc<Mutex<Shortener>>,
}

#[derive(Deserialize)]
struct GenerateRequest {
    url: String,
}

#[get("/")]
async fn redirect_ui() -> impl Responder {
    Redirect::to("/ui".to_owned())
}

#[get("/ui")]
async fn home(tera: web::Data<Tera>) -> impl Responder {
    let mut context = Context::new();
    context.insert("title", "Home");
    let template = tera.render("pages/home.html", &context).expect("Error");
    HttpResponse::Ok().body(template)
}

#[get("/ui/error")]
async fn error(tera: web::Data<Tera>) -> impl Responder {
    let mut context = Context::new();
    context.insert("title", "Error");
    let template = tera.render("pages/error.html", &context).expect("Error");
    HttpResponse::Ok().body(template)
}

#[post("/ui/generated")]
async fn ui_generated(
    data: web::Data<State>,
    tera: web::Data<Tera>,
    info: web::Form<GenerateRequest>,
) -> impl Responder {
    match data.domain.lock().unwrap().store(info.url.clone()) {
        Ok(shorten) => {
            let mut context = Context::new();
            context.insert("title", "Generated");
            context.insert("target_url", &shorten.target_url);
            context.insert("url", &shorten.url);
            let template = tera
                .render("pages/generated.html", &context)
                .expect("Error");
            HttpResponse::Ok().body(template)
        }
        Err(err) => {
            let mut context = Context::new();
            context.insert("title", "Error");
            context.insert("error", &err);
            let template = tera.render("pages/error.html", &context).expect("Error");
            HttpResponse::Ok().body(template)
        }
    }
}

#[get("/{short_id}")]
async fn redirect(path: web::Path<String>, data: web::Data<State>) -> impl Responder {
    let short_id = path.into_inner();
    println!("{}", short_id);
    match data.domain.lock().unwrap().read(short_id) {
        Ok(shorten) => Redirect::to(shorten.url),
        Err(err) => Redirect::to(format!("/ui/error?msg={}", err)),
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
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();

    let domain = Arc::new(Mutex::new(Shortener::new(
        "http://localhost:8080".to_owned(),
    )));

    let state = State { domain };
    let server = HttpServer::new(move || {
        let app_state = Data::new(state.clone());

        App::new()
            .app_data(app_state)
            .app_data(web::Data::new(TEMPLATES.clone()))
            .service(
                fs::Files::new("/assets", "static/")
                    .show_files_listing()
                    .use_last_modified(true),
            )
            .service(redirect_ui)
            .service(home)
            .service(ui_generated)
            .service(redirect)
            .service(error)
    })
    .bind(("0.0.0.0", 8080))?
    .run();

    println!("🚀 application is running on http://localhost:8080");

    server.await
}
