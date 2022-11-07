use actix_web::{
    get,
    http::header,
    post,
    web::{self, Json, Path},
    HttpResponse, Responder,
};
use log::info;
use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, MySql};

use crate::{api::ApiResult, Data};

#[derive(Deserialize, Clone, Debug, Serialize, sqlx::FromRow)]
struct NewLink {
    tiny_code: String,
    origin_url: String,
}

#[derive(Deserialize, Clone)]
struct ApiAddLink {
    origin_url: String,
}

impl ApiAddLink {
    fn to_new_link(self) -> NewLink {
        NewLink {
            tiny_code: nanoid!(5),
            origin_url: self.origin_url,
        }
    }
}

#[post("/create")]
async fn create_link(link: Json<ApiAddLink>, data: web::Data<Data>) -> impl Responder {
    info!("create");
    let new_link = link.0.to_new_link();
    let new_code = new_link.tiny_code.clone();
    if let Err(e) = insert_into_tiny_link(data.as_ref().pool.clone(), new_link).await {
        return Json(ApiResult::error(e.to_string()));
    }
    Json(ApiResult::success(Some(new_code)))
}

async fn insert_into_tiny_link(
    pool: Pool<MySql>,
    new_link: NewLink,
) -> Result<u64, sqlx::Error> {
    let insert_id = sqlx::query(r#"insert into tiny_link (tiny_code,origin_url) values (?, ?)"#)
        .bind(new_link.tiny_code)
        .bind(new_link.origin_url)
        .execute(&pool)
        .await?
        .last_insert_id();
    Ok(insert_id)
}

#[get("/{code}")]
async fn get_from_link(path: Path<String>, data: web::Data<Data>) -> impl Responder {
    info!("get_from_link");
    let code = path.into_inner();
    let url = get_original_url(data.as_ref().pool.clone(), code).await;
    let url = match url {
        Ok(x) => x,
        Err(e) => {
            print!("{}", e);
            return HttpResponse::NotFound().finish();
        }
    };
    HttpResponse::Found()
        .append_header((header::LOCATION, url))
        .finish()
}

async fn get_original_url(pool: Pool<MySql>, code: String) -> Result<String, sqlx::Error> {
    let row: (String,) = sqlx::query_as("SELECT origin_url from tiny_link where tiny_code = ?")
        .bind(code)
        .fetch_one(&pool)
        .await?;
    Ok(row.0)
}

#[get("/links")]
async fn get_all_links(data: web::Data<Data>) -> impl Responder {
    info!("get_all_links");
    let links = get_links(data.as_ref().pool.clone()).await;
    let links = match links {
        Ok(x) => x,
        Err(e) => {
            print!("{}", e);
            return Json(ApiResult::error(e.to_string()));
        }
    };
    Json(ApiResult::success(Some(links)))
}

async fn get_links(pool: Pool<MySql>) -> Result<Vec<NewLink>, sqlx::Error> {
    let row = sqlx::query_as::<_, NewLink>("SELECT tiny_code, origin_url from tiny_link")
        .fetch_all(&pool)
        .await?;
    Ok(row)
}

#[get("/domain")]
async fn get_domain_url(data: web::Data<Data>) -> impl Responder {
    Json(ApiResult::success(Some(data.as_ref().url_pres.clone())))
}
