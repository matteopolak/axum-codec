use std::sync::{Arc, Mutex};

use aide::axum::ApiRouter;
use axum::{
	extract::{Path, State},
	http::StatusCode,
};
use axum_codec::{
	routing::{delete, get, patch, post},
	Codec, IntoCodecResponse,
};

pub fn routes() -> ApiRouter {
	ApiRouter::new()
		.api_route("/", get(get_all).into())
		.api_route("/", post(create).into())
		.api_route("/:id", get(get_one).into())
		.api_route("/:id", patch(update_one).into())
		.api_route("/:id", delete(delete_one).into())
		.with_state(Arc::new(Tasks::default()))
}

#[derive(Default)]
pub struct Tasks(Mutex<Vec<TodoHandle>>);

pub struct TodoHandle {
	deleted: bool,
	inner: Todo,
}

#[axum_codec::apply(encode)]
#[derive(Clone)]
pub struct Todo {
	id: u64,
	/// A title describing the task to be done.
	title: String,
	/// Whether the task has been completed.
	completed: bool,
}

#[axum_codec::apply(decode)]
pub struct CreateTodo {
	/// A title describing the task to be done.
	title: String,
}

#[axum_codec::apply(decode)]
pub struct UpdateTodo {
	/// A title describing the task to be done.
	title: Option<String>,
	/// Whether the task has been completed.
	completed: Option<bool>,
}

async fn get_all(State(tasks): State<Arc<Tasks>>) -> impl IntoCodecResponse {
	let tasks = tasks
		.0
		.lock()
		.unwrap()
		.iter()
		.filter(|handle| !handle.deleted)
		.map(|handle| handle.inner.clone())
		.collect::<Vec<_>>();

	Codec(tasks)
}

async fn create(
	State(tasks): State<Arc<Tasks>>,
	Codec(todo): Codec<CreateTodo>,
) -> impl IntoCodecResponse {
	let mut tasks = tasks.0.lock().unwrap();
	let id = tasks.len() as u64 + 1;

	let todo = Todo {
		id,
		completed: false,
		title: todo.title,
	};

	tasks.push(TodoHandle {
		deleted: false,
		inner: todo.clone(),
	});

	Codec(todo)
}

async fn get_one(State(tasks): State<Arc<Tasks>>, Path(id): Path<u64>) -> impl IntoCodecResponse {
	let tasks = tasks.0.lock().unwrap();
	let handle = match tasks.get(id as usize - 1) {
		Some(handle) if !handle.deleted => handle,
		_ => {
			return Err((
				StatusCode::NOT_FOUND,
				Codec(axum_codec::rejection::Message {
					code: "not_found",
					content: format!("Task with id {} not found", id),
				}),
			))
		}
	};

	Ok(Codec(handle.inner.clone()))
}

async fn update_one(
	State(tasks): State<Arc<Tasks>>,
	Path(id): Path<u64>,
	Codec(todo): Codec<UpdateTodo>,
) -> impl IntoCodecResponse {
	let mut tasks = tasks.0.lock().unwrap();
	let handle = match tasks.get_mut(id as usize - 1) {
		Some(handle) if !handle.deleted => handle,
		_ => {
			return Err((
				StatusCode::NOT_FOUND,
				Codec(axum_codec::rejection::Message {
					code: "not_found",
					content: format!("Task with id {} not found", id),
				}),
			))
		}
	};

	if let Some(title) = todo.title {
		handle.inner.title = title;
	}

	if let Some(completed) = todo.completed {
		handle.inner.completed = completed;
	}

	Ok(Codec(handle.inner.clone()))
}

async fn delete_one(
	State(tasks): State<Arc<Tasks>>,
	Path(id): Path<u64>,
) -> impl IntoCodecResponse {
	let mut tasks = tasks.0.lock().unwrap();
	let handle = match tasks.get_mut(id as usize - 1) {
		Some(handle) if !handle.deleted => handle,
		_ => {
			return Err((
				StatusCode::NOT_FOUND,
				Codec(axum_codec::rejection::Message {
					code: "not_found",
					content: format!("Task with id {} not found", id),
				}),
			))
		}
	};

	handle.deleted = true;

	Ok(Codec(handle.inner.clone()))
}
