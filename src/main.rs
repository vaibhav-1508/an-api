use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use warp::{http, Filter};

type Items = HashMap<String, String>;

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Id {
    name: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Item {
    name: String,
    branch: String,
}

#[derive(Clone)]
struct Store {
    student_list: Arc<RwLock<Items>>,
}

impl Store {
    fn new() -> Self {
        Store {
            student_list: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

async fn update_student_list(
    item: Item,
    store: Store,
) -> Result<impl warp::Reply, warp::Rejection> {
    store.student_list.write().insert(item.name, item.branch);

    Ok(warp::reply::with_status(
        "Added items to the student list",
        http::StatusCode::CREATED,
    ))
}

async fn delete_student_list_item(
    id: Id,
    store: Store,
) -> Result<impl warp::Reply, warp::Rejection> {
    store.student_list.write().remove(&id.name);

    Ok(warp::reply::with_status(
        "Removed item from student list",
        http::StatusCode::OK,
    ))
}

async fn get_student_list(store: Store) -> Result<impl warp::Reply, warp::Rejection> {
    let mut result = HashMap::new();
    let r = store.student_list.read();

    for (key, value) in r.iter() {
        result.insert(key, value);
    }

    Ok(warp::reply::json(&result))
}

fn delete_json() -> impl Filter<Extract = (Id,), Error = warp::Rejection> + Clone {
    warp::body::content_length_limit(1024 * 16).and(warp::body::json())
}

fn post_json() -> impl Filter<Extract = (Item,), Error = warp::Rejection> + Clone {
    warp::body::content_length_limit(1024 * 16).and(warp::body::json())
}

#[tokio::main]
async fn main() {
    let store = Store::new();
    let store_filter = warp::any().map(move || store.clone());

    let add_items = warp::post()
        .and(warp::path("v1"))
        .and(warp::path("student"))
        .and(warp::path::end())
        .and(post_json())
        .and(store_filter.clone())
        .and_then(update_student_list);

    let get_items = warp::get()
        .and(warp::path("v1"))
        .and(warp::path("student"))
        .and(warp::path::end())
        .and(store_filter.clone())
        .and_then(get_student_list);

    let delete_item = warp::delete()
        .and(warp::path("v1"))
        .and(warp::path("student"))
        .and(warp::path::end())
        .and(delete_json())
        .and(store_filter.clone())
        .and_then(delete_student_list_item);

    let update_item = warp::put()
        .and(warp::path("v1"))
        .and(warp::path("student"))
        .and(warp::path::end())
        .and(post_json())
        .and(store_filter.clone())
        .and_then(update_student_list);

    let routes = add_items.or(get_items).or(delete_item).or(update_item);

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
