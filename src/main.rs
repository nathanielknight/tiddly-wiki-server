use axum::{
    error_handling::HandleError,
    http::StatusCode,
    response::Html,
    routing::{get, on_service, MethodFilter},
    Router,
    Extension,
};
use serde::Serialize;
use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use tower_http::services::fs::ServeFile;

#[tokio::main]
async fn main() {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3032));
    println!("listening on {}", addr);

    let tiddlers = Arc::new(Mutex::new(tiddler::Tiddlers::new()));
    let static_wiki = HandleError::new(ServeFile::new("./tiddlywiki.html"), handle_io_error);

    let app = Router::new()
        .route("/", on_service(MethodFilter::GET, static_wiki))
        .route("/status", get(status))
        .route("/recipes/default/tiddlers.json", get(all_tiddlers))
        .layer(Extension(tiddlers));

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn home() -> Html<&'static str> {
    Html("<h1>Hello, Tiddlywiki!</h1>")
}

async fn handle_io_error(err: std::io::Error) -> (StatusCode, String) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("Internal Server Error: {}", err),
    )
}

#[derive(Serialize)]
struct Status {
    username: &'static str,
    anonymous: bool,
    read_only: bool,
    space: Space,
    tiddlywiki_version: &'static str,
}

#[derive(Serialize)]
struct Space {
    recipe: &'static str,
}

const STATUS: Status = Status {
    username: "nknight",
    anonymous: false,
    read_only: false,
    space: Space { recipe: "default" },
    tiddlywiki_version: "5.2.2",
};

async fn status() -> axum::Json<Status> {
    axum::Json(STATUS)
}

type DataStore = Arc<Mutex<tiddler::Tiddlers>>;

async fn all_tiddlers(Extension(ds): Extension<DataStore>) -> axum::Json<Vec<serde_json::Value>> {
    let mut lock = ds.lock().expect("failed to lock tiddlers");
    let tiddlers = &mut *lock;
    let all: Vec<serde_json::Value> = tiddlers.all().iter().map(|t| t.as_value()).collect();
    axum::Json(all)
}

mod tiddler {
    use serde_json::{Number, Value};

    pub(crate) struct Tiddlers {
        tiddlers: std::collections::HashMap<String, Tiddler>,
    }

    impl Tiddlers {
        pub(crate) fn all(&self) -> Vec<Tiddler> {
            self.tiddlers.values().map(|t| t.clone()).collect()
        }

        pub(crate) fn new() -> Self {
            Tiddlers {
                tiddlers: std::collections::HashMap::new(),
            }
        }
    }

    #[derive(Clone)]
    pub(crate) struct Tiddler {
        title: String,
        text: String,
        revision: u64,
        meta: serde_json::Value,
    }

    impl Tiddler {
        pub(crate) fn as_value(&self) -> Value {
            let mut meta = self.meta.clone();
            meta["title"] = Value::String(self.title.clone());
            meta["text"] = Value::String(self.text.clone());
            meta["revision"] = Value::Number(Number::from(self.revision));
            meta
        }
}
