use axum::{
    body::Body,
    http::{header, StatusCode},
    response::Response,
};
use rust_embed::Embed;

#[derive(Embed)]
#[folder = "../../dashboard/dist/"]
struct DashboardAssets;

fn error_response(status: StatusCode, msg: &str) -> Response {
    Response::builder()
        .status(status)
        .body(Body::from(msg.to_string()))
        .unwrap_or_else(|_| Response::new(Body::empty()))
}

pub fn serve_asset(path: &str) -> Response {
    let path = path.trim_start_matches('/');

    match DashboardAssets::get(path) {
        Some(content) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, mime.as_ref())
                .body(Body::from(content.data))
                .unwrap_or_else(|_| error_response(StatusCode::INTERNAL_SERVER_ERROR, "Failed to build response"))
        }
        None => serve_fallback(),
    }
}

pub fn serve_fallback() -> Response {
    match DashboardAssets::get("index.html") {
        Some(content) => Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "text/html")
            .body(Body::from(content.data))
            .unwrap_or_else(|_| error_response(StatusCode::INTERNAL_SERVER_ERROR, "Failed to build response")),
        None => error_response(
            StatusCode::NOT_FOUND,
            "Dashboard not built. Run `cd dashboard && bun run build`.",
        ),
    }
}
