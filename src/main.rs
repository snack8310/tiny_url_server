use actix_cors::Cors;
use actix_web::{
    get,
    web::{self},
    App, HttpResponse, HttpServer, Responder,
};
use log::info;
use sqlx::{mysql::MySqlPoolOptions, Pool, MySql};
use log::LevelFilter;

use settings::Settings;

mod api;
mod settings;

use simple_logger::SimpleLogger;

#[get("/")]
async fn index() -> impl Responder {
    let content = "Generate a short code for url";
    HttpResponse::Ok().body(content)
}

#[derive(Clone)]
struct Data{
    pool: Pool<MySql>,
    url_pres: Vec<String>
}

#[actix_web::main]
async fn main() -> Result<(), sqlx::Error> {
    let s = Settings::new().unwrap();
    let ip = s.server.get_ip();
    let url = s.database.url;

    SimpleLogger::new().with_level(LevelFilter::Info).init().unwrap();

    info!("server listening at http://{:?}", ip);
    let pres = s.tiny_url.pre();
    info!("pre: {:?}", pres);

    let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect(&url)
        .await?;

    let data = Data{
        pool: pool,
        url_pres: pres,
    };

    HttpServer::new(move || {
        // let cors = Cors::default()
        //     .allowed_origin("http://0.0.0.0:8080")
        //     .allowed_methods(vec!["GET", "POST"])
        //     .allowed_header(http::header::CONTENT_TYPE)
        //     .max_age(3600);
        let cors = Cors::permissive();

        App::new()
            .wrap(cors)
            .app_data(web::Data::new(data.clone()))
            .service(index)
            .service(api::api::get_domain_url)
            .service(api::api::create_link)
            .service(api::api::get_all_links)
            .service(api::api::get_from_link)
            .service(api::api::get_origin_url_from_link)
    })
    .bind(&ip)?
    .run()
    .await?;

    Ok(())
}
