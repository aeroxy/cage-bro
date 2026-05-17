use axum::{extract::State, routing::post, Json, Router};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::server::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/launch", post(launch))
        .route("/navigate", post(navigate))
        .route("/screenshot", post(screenshot))
        .route("/click", post(click))
        .route("/type", post(type_text))
        .route("/evaluate", post(evaluate))
        .route("/content", post(content))
        .route("/close", post(close))
}

#[derive(Deserialize)]
struct LaunchRequest {
    port: Option<u16>,
    stealth: Option<bool>,
}

#[derive(Deserialize)]
struct NavigateRequest {
    url: String,
}

#[derive(Deserialize)]
struct ScreenshotRequest {
    quality: Option<u32>,
}

#[derive(Deserialize)]
struct ClickRequest {
    selector: String,
}

#[derive(Deserialize)]
struct TypeRequest {
    selector: String,
    text: String,
}

#[derive(Deserialize)]
struct EvaluateRequest {
    expression: String,
}

async fn launch(
    State(state): State<AppState>,
    Json(req): Json<LaunchRequest>,
) -> Json<Value> {
    match state
        .browser
        .launch(req.port, req.stealth.unwrap_or(false))
        .await
    {
        Ok(msg) => Json(json!({ "status": "ok", "message": msg })),
        Err(e) => Json(json!({ "error": e })),
    }
}

async fn navigate(
    State(state): State<AppState>,
    Json(req): Json<NavigateRequest>,
) -> Json<Value> {
    match state.browser.navigate(&req.url).await {
        Ok(page) => Json(json!({
            "url": page.url,
            "title": page.title,
            "text": page.text,
        })),
        Err(e) => Json(json!({ "error": e })),
    }
}

async fn screenshot(
    State(state): State<AppState>,
    Json(req): Json<ScreenshotRequest>,
) -> Json<Value> {
    match state.browser.screenshot(req.quality).await {
        Ok(s) => Json(json!({
            "data": s.data,
            "format": s.format,
            "width": s.width,
            "height": s.height,
        })),
        Err(e) => Json(json!({ "error": e })),
    }
}

async fn click(
    State(state): State<AppState>,
    Json(req): Json<ClickRequest>,
) -> Json<Value> {
    match state.browser.click(&req.selector).await {
        Ok(()) => Json(json!({ "status": "ok" })),
        Err(e) => Json(json!({ "error": e })),
    }
}

async fn type_text(
    State(state): State<AppState>,
    Json(req): Json<TypeRequest>,
) -> Json<Value> {
    match state.browser.type_text(&req.selector, &req.text).await {
        Ok(()) => Json(json!({ "status": "ok" })),
        Err(e) => Json(json!({ "error": e })),
    }
}

async fn evaluate(
    State(state): State<AppState>,
    Json(req): Json<EvaluateRequest>,
) -> Json<Value> {
    match state.browser.execute_js(&req.expression).await {
        Ok(val) => Json(json!({ "result": val })),
        Err(e) => Json(json!({ "error": e })),
    }
}

async fn content(State(state): State<AppState>) -> Json<Value> {
    match state.browser.get_content().await {
        Ok(page) => Json(json!({
            "url": page.url,
            "title": page.title,
            "text": page.text,
        })),
        Err(e) => Json(json!({ "error": e })),
    }
}

async fn close(State(state): State<AppState>) -> Json<Value> {
    match state.browser.close().await {
        Ok(()) => Json(json!({ "status": "closed" })),
        Err(e) => Json(json!({ "error": e })),
    }
}
