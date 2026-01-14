use actix_cors::Cors;
use actix_multipart::Multipart;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use futures_util::stream::StreamExt as _;
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::sleep;
use uuid::Uuid;

mod cleanup;
mod conversion;
mod validation;

use cleanup::CleanupService;
use conversion::{convert_video, ConversionFormat};
use validation::{validate_conversion, validate_file_size};

const VIDEO_MAX_SIZE: usize = 200 * 1024 * 1024; // 200 MB
const AUDIO_MAX_SIZE: usize = 100 * 1024 * 1024; // 100 MB
const CLEANUP_INTERVAL_MINUTES: u64 = 5;
const FILE_RETENTION_MINUTES: u64 = 10;

#[derive(Debug, Serialize, Deserialize)]
struct ConversionRequest {
    format: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ConversionResponse {
    success: bool,
    message: String,
    file_id: Option<String>,
    filename: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ErrorResponse {
    success: bool,
    error: String,
}

#[derive(Clone)]
struct AppState {
    upload_dir: PathBuf,
    output_dir: PathBuf,
    cleanup_service: Arc<Mutex<CleanupService>>,
}

#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "service": "Video Converter API",
        "version": "1.0.0",
        "status": "running",
        "endpoints": {
            "upload": "POST /api/upload",
            "download": "GET /api/download/{file_id}",
            "health": "GET /api/health"
        }
    }))
}

#[get("/api/health")]
async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

#[post("/api/upload")]
async fn upload_file(
    mut payload: Multipart,
    state: web::Data<AppState>,
) -> Result<HttpResponse, actix_web::Error> {
    let mut target_format: Option<String> = None;
    let mut uploaded_file_path: Option<PathBuf> = None;
    let mut original_filename: Option<String> = None;

    while let Some(item) = payload.next().await {
        let mut field = item?;
        let content_disposition = field.content_disposition();
        let field_name = content_disposition.get_name().unwrap_or("");

        if field_name == "format" {
            let mut format_bytes = Vec::new();
            while let Some(chunk) = field.next().await {
                let data = chunk?;
                format_bytes.extend_from_slice(&data);
            }
            target_format = Some(String::from_utf8_lossy(&format_bytes).to_string());
        } else if field_name == "file" {
            let filename = content_disposition
                .get_filename()
                .map(|f| sanitize_filename::sanitize(f))
                .unwrap_or_else(|| format!("upload_{}", Uuid::new_v4()));

            original_filename = Some(filename.clone());

            let file_id = Uuid::new_v4().to_string();
            let file_ext = std::path::Path::new(&filename)
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("tmp");

            let filepath = state
                .upload_dir
                .join(format!("{}_{}.{}", file_id, "upload", file_ext));

            let mut file_size = 0usize;
            let filepath_clone = filepath.clone();
            let mut f = web::block(move || std::fs::File::create(filepath_clone))
                .await??;

            while let Some(chunk) = field.next().await {
                let data = chunk?;
                file_size += data.len();

                // Check size limit during upload
                if file_size > VIDEO_MAX_SIZE {
                    return Ok(HttpResponse::BadRequest().json(ErrorResponse {
                        success: false,
                        error: format!(
                            "File size exceeds maximum limit of {} MB",
                            VIDEO_MAX_SIZE / (1024 * 1024)
                        ),
                    }));
                }

                f = web::block(move || {
                    f.write_all(&data)?;
                    Ok::<_, std::io::Error>(f)
                })
                .await??;
            }

            uploaded_file_path = Some(filepath);
        }
    }

    // Validate required fields
    let format = match target_format {
        Some(f) => f.trim().to_lowercase(),
        None => {
            return Ok(HttpResponse::BadRequest().json(ErrorResponse {
                success: false,
                error: "Missing 'format' field".to_string(),
            }));
        }
    };

    let input_path = match uploaded_file_path {
        Some(p) => p,
        None => {
            return Ok(HttpResponse::BadRequest().json(ErrorResponse {
                success: false,
                error: "No file uploaded".to_string(),
            }));
        }
    };

    // Get input file extension
    let input_ext = input_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    // Validate conversion
    if let Err(e) = validate_conversion(&input_ext, &format) {
        // Clean up uploaded file
        let _ = fs::remove_file(&input_path);
        return Ok(HttpResponse::BadRequest().json(ErrorResponse {
            success: false,
            error: e,
        }));
    }

    // Validate file size based on target format
    let max_size = if format == "mp3" {
        AUDIO_MAX_SIZE
    } else {
        VIDEO_MAX_SIZE
    };

    if let Err(e) = validate_file_size(&input_path, max_size) {
        let _ = fs::remove_file(&input_path);
        return Ok(HttpResponse::BadRequest().json(ErrorResponse {
            success: false,
            error: e,
        }));
    }

    // Perform conversion
    let conversion_format = ConversionFormat::from_str(&format);
    let file_id = Uuid::new_v4().to_string();
    let output_filename = format!(
        "{}_{}.{}",
        original_filename
            .unwrap_or_else(|| "converted".to_string())
            .trim_end_matches(&format!(".{}", input_ext)),
        file_id,
        format
    );
    let output_path = state.output_dir.join(&output_filename);

    match convert_video(&input_path, &output_path, conversion_format) {
        Ok(_) => {
            // Clean up input file
            let _ = fs::remove_file(&input_path);

            // Register file for cleanup
            let mut cleanup = state.cleanup_service.lock().await;
            cleanup.register_file(output_path.clone());

            info!("Conversion successful: {}", output_filename);

            Ok(HttpResponse::Ok().json(ConversionResponse {
                success: true,
                message: "Conversion successful".to_string(),
                file_id: Some(file_id),
                filename: Some(output_filename),
            }))
        }
        Err(e) => {
            // Clean up both files on error
            let _ = fs::remove_file(&input_path);
            let _ = fs::remove_file(&output_path);

            error!("Conversion failed: {}", e);

            Ok(HttpResponse::InternalServerError().json(ErrorResponse {
                success: false,
                error: format!("Conversion failed: {}", e),
            }))
        }
    }
}

#[get("/api/download/{filename}")]
async fn download_file(
    filename: web::Path<String>,
    state: web::Data<AppState>,
) -> Result<HttpResponse, actix_web::Error> {
    let filename = sanitize_filename::sanitize(filename.into_inner());
    let filepath = state.output_dir.join(&filename);

    if !filepath.exists() {
        return Ok(HttpResponse::NotFound().json(ErrorResponse {
            success: false,
            error: "File not found or already deleted".to_string(),
        }));
    }

    let file_data = web::block(move || fs::read(filepath.clone()))
        .await
        .map_err(|_| actix_web::error::ErrorInternalServerError("Failed to read file"))?
        .map_err(|_| actix_web::error::ErrorInternalServerError("Failed to read file"))?;

    // Delete file immediately after reading
    let filepath_clone = state.output_dir.join(&filename);
    tokio::spawn(async move {
        let _ = fs::remove_file(filepath_clone);
    });

    Ok(HttpResponse::Ok()
        .content_type("application/octet-stream")
        .insert_header(("Content-Disposition", format!("attachment; filename=\"{}\"", filename)))
        .body(file_data))
}

async fn run_cleanup_service(cleanup_service: Arc<Mutex<CleanupService>>) {
    loop {
        sleep(Duration::from_secs(CLEANUP_INTERVAL_MINUTES * 60)).await;
        let mut service = cleanup_service.lock().await;
        service.cleanup_old_files(FILE_RETENTION_MINUTES);
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables
    dotenv::dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // Create necessary directories
    let upload_dir = PathBuf::from("temp/uploads");
    let output_dir = PathBuf::from("temp/outputs");

    fs::create_dir_all(&upload_dir)?;
    fs::create_dir_all(&output_dir)?;

    info!("Created temporary directories");

    // Initialize cleanup service
    let cleanup_service = Arc::new(Mutex::new(CleanupService::new()));

    // Spawn cleanup background task
    let cleanup_service_clone = cleanup_service.clone();
    tokio::spawn(async move {
        run_cleanup_service(cleanup_service_clone).await;
    });

    info!("Started cleanup service");

    // Check FFmpeg availability
    match Command::new("ffmpeg").arg("-version").output() {
        Ok(_) => info!("FFmpeg is available"),
        Err(_) => {
            error!("FFmpeg is not installed or not in PATH!");
            error!("Please install FFmpeg to use this service");
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "FFmpeg not found",
            ));
        }
    }

    let app_state = AppState {
        upload_dir,
        output_dir,
        cleanup_service,
    };

    let bind_address = std::env::var("BIND_ADDRESS").unwrap_or_else(|_| "127.0.0.1".to_string());
    let bind_port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let bind = format!("{}:{}", bind_address, bind_port);

    info!("Starting Video Converter API server at http://{}", bind);
    info!("Video max size: {} MB", VIDEO_MAX_SIZE / (1024 * 1024));
    info!("Audio max size: {} MB", AUDIO_MAX_SIZE / (1024 * 1024));
    info!(
        "File retention: {} minutes",
        FILE_RETENTION_MINUTES
    );

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .wrap(cors)
            .wrap(actix_web::middleware::Logger::default())
            .service(index)
            .service(health_check)
            .service(upload_file)
            .service(download_file)
    })
    .bind(&bind)?
    .run()
    .await
}
