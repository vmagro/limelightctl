use std::collections::HashMap;
use std::sync::Arc;
use std::sync::RwLock;

use axum::Json;
use axum::Router;
use axum::extract::Query;
use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::get;
use axum::routing::post;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Deserialize)]
struct IndexQuery {
    index: usize,
}

#[derive(Debug, Default)]
struct AppState {
    pipelines: HashMap<usize, Pipeline>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Pipeline {
    desc: String,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let pipelines = Arc::new(RwLock::new(AppState::default()));

    let app = Router::new()
        .route("/status", get(status))
        .route("/pipeline-atindex", get(pipeline_atindex))
        .route("/upload-pipeline", post(upload_pipeline))
        .with_state(pipelines);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn status(State(state): State<Arc<RwLock<AppState>>>) -> Json<serde_json::Value> {
    Json(
        serde_json::json!({"mock": true, "num_pipelines": state.read().expect("read lock poisoned").pipelines.len()}),
    )
}

async fn pipeline_atindex(
    State(state): State<Arc<RwLock<AppState>>>,
    Query(params): Query<IndexQuery>,
) -> (StatusCode, Json<Option<Pipeline>>) {
    match state
        .read()
        .expect("read lock poisoned")
        .pipelines
        .get(&params.index)
    {
        Some(p) => (StatusCode::OK, Json(Some(p.clone()))),
        None => (StatusCode::NOT_FOUND, Json(None)),
    }
}

async fn upload_pipeline(
    State(state): State<Arc<RwLock<AppState>>>,
    Query(params): Query<IndexQuery>,
    Json(pipeline): Json<Pipeline>,
) -> StatusCode {
    eprintln!("writing pipeline at {}", params.index);
    state
        .write()
        .expect("write lock poisoned")
        .pipelines
        .insert(params.index, pipeline);
    StatusCode::OK
}
