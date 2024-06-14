use actix_web::{post, web, App, HttpServer, HttpResponse, Responder};
use serde::{Serialize, Deserialize};
use actix_cors::Cors;
use env_logger::Env;
use log::info;
use std::time::Instant;

#[derive(Serialize)]
struct ResponseData {
    data: Vec<u8>,
    width: usize,
    height: usize,
}

#[derive(Deserialize)]
struct InputData {
    data: Vec<Vec<Vec<u8>>>,
}


#[post("/convert")]
async fn convert(input: web::Json<InputData>) -> HttpResponse {
    let start = Instant::now();
    info!("Received request with data size: {}x{}", input.data.len(), input.data[0].len());

    let data = &input.data;
    let width = data[0].len();
    let height = data.len();
    let mut output = vec![0u8; width * height * 4];  // RGBA
    for i in 0..height {
        for j in 0..width {
            let idx = (i * width + j) * 4;
            output[idx] = data[i][j][0] as u8;
            output[idx + 1] = data[i][j][1] as u8;
            output[idx + 2] = data[i][j][2] as u8;
            output[idx + 3] = 255; // Alpha channel
        }
    }
    let duration = start.elapsed();
    info!("Processed request successfully in {:?}", duration);
    HttpResponse::Ok().json(ResponseData {
        data: output,
        width: width,
        height: height,
    })
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    HttpServer::new(|| {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header();

        let json_cfg = web::JsonConfig::default()
            .limit(10 * 1024 * 1024);  // 10 MB

        App::new()
            .wrap(cors)
            .app_data(json_cfg)
            .service(convert)
    })
    .bind(("127.0.0.1", 5000))?
    .run()
    .await
}
