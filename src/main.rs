use actix_files::Files;
use actix_multipart::Multipart;
use actix_web::web::ServiceConfig;
use actix_web::HttpResponse;
use futures::StreamExt;
use shuttle_actix_web::ShuttleActixWeb;
use std::fs;
use std::io::Write;

#[actix_web::post("/upload")]
async fn upload(mut payload: Multipart) -> Result<HttpResponse, actix_web::Error> {
    let upload_dir = "images";
    fs::create_dir_all(upload_dir)?;

    // Process the multipart payload
    while let Some(item) = payload.next().await {
        let mut field = item?;
        let content_disposition = field.content_disposition().clone();
        let filename = content_disposition
            .get_filename()
            .map_or("unnamed".to_string(), |f| f.to_string());

        // Sanitize filename (basic example, improve as needed)
        let sanitized_filename = format!("{}/{}", upload_dir, sanitize_filename(&filename));
        
        // Create a file and write the uploaded data to it
        let mut f = fs::File::create(&sanitized_filename)?;
        while let Some(chunk) = field.next().await {
            let data = chunk?;
            f.write_all(&data)?;
        }
    }

    Ok(HttpResponse::Ok().body("Image uploaded successfully!"))
}

// Simple filename sanitizer (replace with a more robust solution in production)
fn sanitize_filename(filename: &str) -> String {
    filename.replace("..", "").replace("/", "").replace("\\", "")
}

#[shuttle_runtime::main]
async fn main() -> ShuttleActixWeb<impl FnOnce(&mut ServiceConfig) + Send + Clone + 'static> {
    let config = move |cfg: &mut ServiceConfig| {
        cfg.service(upload);
        cfg.service(Files::new("/images", "images").show_files_listing());
    };

    Ok(config.into())
}
