use std::borrow::Cow;

use axum::{
	extract::{DefaultBodyLimit, State},
	response::Response,
	Router,
};
use axum_codec::{
	routing::{get, post},
	Accept, BorrowCodec, Codec, CodecRejection, IntoCodecResponse,
};

#[axum_codec::apply(encode, decode)]
struct User {
	#[validate(length(min = 1, max = 100))]
	name: String,
	#[validate(range(min = 0, max = 150))]
	age: u8,
}

async fn me() -> impl IntoCodecResponse {
	Codec(User {
		name: "Alice".into(),
		age: 42,
	})
}

#[axum_codec::apply(encode)]
struct Greeting {
	message: String,
}

async fn greet(Codec(user): Codec<User>) -> Codec<Greeting> {
	Codec(Greeting {
		message: format!("Hello, {}! You are {} years old.", user.name, user.age),
	})
}

async fn state(State(state): State<String>) -> Codec<Greeting> {
	Codec(Greeting { message: state })
}

#[axum_codec::apply(encode, decode)]
struct BorrowGreeting<'d> {
	#[serde(borrow)]
	message: Cow<'d, str>,
}

async fn borrow_greet(
	accept: Accept,
	greeting: BorrowCodec<BorrowGreeting<'_>>,
) -> Result<Response, CodecRejection> {
	let greeting = greeting.decode()?;
	let is_zero = matches!(greeting.message, Cow::Borrowed(..));

	Ok(
		Codec(Greeting {
			message: if is_zero {
				"Borrowing from input"
			} else {
				"Not borrowing from input (for JSON, probably using an escaped character)"
			}
			.to_string(),
		})
		.into_codec_response(accept.into()),
	)
}

#[tokio::main]
async fn main() {
	let app = Router::new()
		.route("/me", get(me).into())
		.route("/greet", post(greet).into())
		.route("/borrow-greet", post(borrow_greet).into())
		.route("/state", get(state).into())
		.layer(DefaultBodyLimit::max(1024))
		.with_state("Hello, world!".to_string());

	let listener = tokio::net::TcpListener::bind(("127.0.0.1", 3000))
		.await
		.unwrap();

	axum::serve(listener, app).await.unwrap();
}
