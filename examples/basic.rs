use axum::{extract::State, Router};
use axum_codec::{
	handler::IntoCodec,
	routing::{get, post},
	Codec,
};

#[axum_codec::derive(encode, decode)]
struct User {
	name: String,
	age: u8,
}

async fn me() -> impl IntoCodec<User> {
	User {
		name: "Alice".into(),
		age: 42,
	}
}

#[axum_codec::derive(encode)]
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

#[tokio::main]
async fn main() {
	let app = Router::new()
		.route("/me", get(me).into())
		.route("/greet", post(greet).into())
		.route("/state", get(state).into())
		.with_state("Hello, world!".to_string());

	let listener = tokio::net::TcpListener::bind(("127.0.0.1", 3000))
		.await
		.unwrap();

	axum::serve(listener, app).await.unwrap();
}
