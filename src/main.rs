use actix_cors::Cors;
use actix_files::Files;
use actix_multipart::Multipart;
use actix_web::web;
use actix_web::web::ServiceConfig;
use actix_web::HttpResponse;
use futures::StreamExt;
use serde::Deserialize;
use shuttle_actix_web::ShuttleActixWeb;
use std::fs;
use std::io::Write;

#[actix_web::post("/upload/{property_uuid}")]
async fn upload(
    path: web::Path<String>,
    mut payload: Multipart,
) -> Result<HttpResponse, actix_web::Error> {
    let property_uuid = path.into_inner();
    let upload_dir = format!("images/{}", sanitize_filename(&property_uuid));
    fs::create_dir_all(&upload_dir)?;

    while let Some(item) = payload.next().await {
        let mut field = item?;
        let content_disposition = field.content_disposition().clone();
        let filename = content_disposition
            .get_filename()
            .map_or("unnamed".to_string(), |f| f.to_string());

        let sanitized_filename = format!("{}/{}", upload_dir, sanitize_filename(&filename));

        let mut f = fs::File::create(&sanitized_filename)?;
        while let Some(chunk) = field.next().await {
            let data = chunk?;
            f.write_all(&data)?;
        }
    }

    Ok(HttpResponse::Ok().finish())
}

#[derive(Deserialize)]
struct DeleteFilePayload {
    filename: String,
}
#[actix_web::delete("/file/{id}")]
async fn delete_file(
    id: web::Path<String>,
    data: web::Json<DeleteFilePayload>,
) -> Result<HttpResponse, actix_web::Error> {
    let basedir = id.into_inner();

    let file_path = format!("images/{}/{}", basedir, data.filename);
    fs::remove_file(file_path)?;

    // if directory is empty, delete it
    let dir_path = format!("images/{}", basedir);
    if fs::read_dir(&dir_path)?.next().is_none() {
        fs::remove_dir(dir_path)?;
    }

    Ok(HttpResponse::Ok().finish())
}

fn sanitize_filename(filename: &str) -> String {
    filename
        .replace("..", "")
        .replace("/", "")
        .replace("\\", "")
}

#[shuttle_runtime::main]
async fn main() -> ShuttleActixWeb<impl FnOnce(&mut ServiceConfig) + Send + Clone + 'static> {
    let config = move |cfg: &mut ServiceConfig| {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header();

        cfg.service(
            web::scope("")
                .wrap(cors)
                .service(upload)
                .service(delete_file)
                .service(Files::new("/images", "images").show_files_listing()),
        );
    };

    Ok(config.into())
}
