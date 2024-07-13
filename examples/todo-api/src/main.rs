mod todo;

use std::sync::Arc;

use aide::axum::ApiRouter;
use axum::{response::IntoResponse, Extension};
use axum_codec::{routing::get, IntoCodecResponse};

async fn openapi(Extension(api): Extension<Arc<aide::openapi::OpenApi>>) -> impl IntoCodecResponse {
	axum::Json(api).into_response()
}

#[tokio::main]
async fn main() {
	let mut api = aide::openapi::OpenApi::default();

	let app = ApiRouter::new()
		.nest_api_service("/todos", todo::routes())
		.route("/openapi.json", get(openapi))
		.finish_api(&mut api)
		.layer(Extension(Arc::new(api)));

	let listener = tokio::net::TcpListener::bind(("127.0.0.1", 3000))
		.await
		.unwrap();

	axum::serve(listener, app).await.unwrap();
}
