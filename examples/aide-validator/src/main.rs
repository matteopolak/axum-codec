use std::sync::Arc;

use aide::axum::ApiRouter;
use axum::{extract::State, response::IntoResponse, Extension};
use axum_codec::{
	routing::{get, post},
	Codec,
};

#[axum_codec::apply(encode, decode)]
struct User {
	name: String,
	#[validate(range(min = 0, max = 120))]
	age: u8,
}

async fn me() -> User {
	User {
		name: "Alice".into(),
		age: 42,
	}
}

#[axum_codec::apply(encode)]
struct Greeting {
	message: String,
}

async fn greet(Codec(user): Codec<User>) -> Greeting {
	Greeting {
		message: format!("Hello, {}! You are {} years old.", user.name, user.age),
	}
}

async fn state(State(state): State<String>) -> Greeting {
	Greeting { message: state }
}

async fn openapi(Extension(api): Extension<Arc<aide::openapi::OpenApi>>) -> impl IntoResponse {
	axum::Json(api)
}

#[tokio::main]
async fn main() {
	let mut api = aide::openapi::OpenApi::default();

	let app = ApiRouter::new()
		.api_route("/me", get(me).into())
		.api_route("/greet", post(greet).into())
		.api_route("/state", get(state).into())
		.route("/openapi.json", axum::routing::get(openapi))
		.finish_api(&mut api)
		.layer(Extension(Arc::new(api)))
		.with_state("Hello, world!".to_string());

	let listener = tokio::net::TcpListener::bind(("127.0.0.1", 3000))
		.await
		.unwrap();

	axum::serve(listener, app).await.unwrap();
}