use axum::{extract::State, routing::post, Json, Router};
use cage_bro_runtime::Filesystem;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::server::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/read", post(read))
        .route("/write", post(write))
        .route("/list", post(list))
        .route("/edit", post(edit))
        .route("/search", post(search))
        .route("/delete", post(delete))
}

#[derive(Deserialize)]
struct PathRequest {
    path: String,
}

#[derive(Deserialize)]
struct WriteRequest {
    path: String,
    content: String,
}

#[derive(Deserialize)]
struct EditRequest {
    path: String,
    old_text: String,
    new_text: String,
}

#[derive(Deserialize)]
struct SearchRequest {
    query: String,
    path: Option<String>,
    file_pattern: Option<String>,
    max_results: Option<usize>,
}

async fn read(
    State(state): State<AppState>,
    Json(req): Json<PathRequest>,
) -> Json<Value> {
    tracing::info!(path = %req.path, "file read");
    match state.filesystem.read(&req.path).await {
        Ok(result) => Json(json!({
            "path": result.path,
            "content": result.content,
            "encoding": result.encoding,
        })),
        Err(e) => Json(json!({ "error": e.to_string() })),
    }
}

async fn write(
    State(state): State<AppState>,
    Json(req): Json<WriteRequest>,
) -> Json<Value> {
    tracing::info!(path = %req.path, "file write");
    match state
        .filesystem
        .write(cage_bro_runtime::FileWriteRequest {
            path: req.path,
            content: req.content,
            encoding: None,
        })
        .await
    {
        Ok(()) => Json(json!({ "status": "ok" })),
        Err(e) => Json(json!({ "error": e.to_string() })),
    }
}

async fn list(
    State(state): State<AppState>,
    Json(req): Json<PathRequest>,
) -> Json<Value> {
    tracing::info!(path = %req.path, "file list");
    match state.filesystem.list(&req.path).await {
        Ok(entries) => {
            let items: Vec<Value> = entries
                .iter()
                .map(|e| {
                    json!({
                        "path": e.path,
                        "name": e.name,
                        "is_dir": e.is_dir,
                        "size": e.size,
                        "modified": e.modified,
                    })
                })
                .collect();
            Json(json!({ "entries": items }))
        }
        Err(e) => Json(json!({ "error": e.to_string() })),
    }
}

async fn edit(
    State(state): State<AppState>,
    Json(req): Json<EditRequest>,
) -> Json<Value> {
    tracing::info!(path = %req.path, "file edit");
    match state
        .filesystem
        .edit(cage_bro_runtime::FileEditRequest {
            path: req.path,
            old_text: req.old_text,
            new_text: req.new_text,
        })
        .await
    {
        Ok(()) => Json(json!({ "status": "ok" })),
        Err(e) => Json(json!({ "error": e.to_string() })),
    }
}

async fn search(
    State(state): State<AppState>,
    Json(req): Json<SearchRequest>,
) -> Json<Value> {
    tracing::info!(query = %req.query, "file search");
    match state
        .filesystem
        .search(cage_bro_runtime::FileSearchRequest {
            query: req.query,
            path: req.path,
            file_pattern: req.file_pattern,
            max_results: req.max_results,
        })
        .await
    {
        Ok(results) => {
            let items: Vec<Value> = results
                .iter()
                .map(|r| {
                    json!({
                        "path": r.path,
                        "line_number": r.line_number,
                        "line_content": r.line_content,
                    })
                })
                .collect();
            Json(json!({ "results": items }))
        }
        Err(e) => Json(json!({ "error": e.to_string() })),
    }
}

async fn delete(
    State(state): State<AppState>,
    Json(req): Json<PathRequest>,
) -> Json<Value> {
    tracing::info!(path = %req.path, "file delete");
    match state.filesystem.delete(&req.path).await {
        Ok(()) => Json(json!({ "status": "ok" })),
        Err(e) => Json(json!({ "error": e.to_string() })),
    }
}
