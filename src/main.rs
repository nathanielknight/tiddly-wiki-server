use axum::{
    error_handling::HandleError,
    extract,
    http::StatusCode,
    routing::{delete, get, on_service, put, MethodFilter},
    Extension, Router,
};
use serde::Serialize;
use serde_json::{Number, Value};
use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use tower_http::services::fs::ServeFile;

type DataStore = Arc<Mutex<Tiddlers>>;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let addr = SocketAddr::from(([127, 0, 0, 1], 3032));
    println!("listening on {}", addr);

    let tiddlers = Arc::new(Mutex::new(Tiddlers::new()));
    let static_wiki = HandleError::new(ServeFile::new("./tiddlywiki.html"), handle_io_error);

    let app = Router::new()
        .route("/", on_service(MethodFilter::GET, static_wiki))
        .route("/status", get(status))
        .route("/recipes/default/tiddlers.json", get(all_tiddlers))
        .route(
            "/recipes/default/tiddlers/:title",
            put(put_tiddler).get(get_tiddler),
        )
        // NOTE(nknight): For some reason both the 'default' and 'efault' versions of this URL get hit.
        .route("/bags/default/tiddlers/:title", delete(delete_tiddler))
        .route("/bags/efault/tiddlers/:title", delete(delete_tiddler))
        .layer(Extension(tiddlers));

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

// -----------------------------------------------------------------------------------
// Views
async fn all_tiddlers(Extension(ds): Extension<DataStore>) -> axum::Json<Vec<serde_json::Value>> {
    let mut lock = ds.lock().expect("failed to lock tiddlers");
    let tiddlers = &mut *lock;
    let all: Vec<serde_json::Value> = tiddlers.all().iter().map(|t| t.as_value()).collect();
    axum::Json(all)
}

async fn get_tiddler(
    Extension(ds): Extension<DataStore>,
    extract::Path(title): extract::Path<String>,
) -> Result<axum::Json<serde_json::Value>, axum::response::Response> {
    let mut lock = ds.lock().expect("failed to lock tiddlers");
    let tiddlers = &mut *lock;

    if let Some(t) = tiddlers.get(&title) {
        Ok(axum::Json(t.as_value()))
    } else {
        let mut resp = axum::response::Response::default();
        *resp.status_mut() = StatusCode::NOT_FOUND;
        Err(resp)
    }
}

async fn delete_tiddler(
    Extension(ds): Extension<DataStore>,
    extract::Path(title): extract::Path<String>,
) -> axum::response::Response<String> {
    let mut lock = ds.lock().expect("failed to lock tiddlers");
    let tiddlers = &mut *lock;
    tiddlers.pop(&title);

    let mut resp = axum::response::Response::default();
    *resp.status_mut() = StatusCode::NO_CONTENT;
    resp
}

async fn put_tiddler(
    Extension(ds): Extension<DataStore>,
    extract::Json(v): extract::Json<serde_json::Value>,
    extract::Path(title): extract::Path<String>,
) -> Result<axum::http::Response<String>, String> {
    use axum::http::response::Response;
    let mut new_tiddler =
        Tiddler::from_value(v).map_err(|e| format!("Error converting tiddler: {}", e))?;
    let mut lock = ds.lock().expect("failed to lock tiddlers");
    let tiddlers = &mut *lock;

    if let Some(_old_tiddler) = tiddlers.pop(&title) {
        new_tiddler.revision += 1;
    }
    let new_revision = new_tiddler.revision;
    tiddlers.put(new_tiddler);
    Response::builder()
        .status(StatusCode::NO_CONTENT)
        .header("Etag", format!("default/{}/{}:", title, new_revision))
        .body(String::new())
        .map_err(|e| format!("Error building response: {}", e))
}

// -----------------------------------------------------------------------------------
// Models and serialization/parsing

pub(crate) struct Tiddlers {
    tiddlers: std::collections::HashMap<String, Tiddler>,
}

impl Tiddlers {
    pub(crate) fn all(&self) -> Vec<Tiddler> {
        self.tiddlers.values().cloned().collect()
    }

    pub(crate) fn new() -> Self {
        Tiddlers {
            tiddlers: std::collections::HashMap::new(),
        }
    }

    pub(crate) fn get(&self, title: &str) -> Option<Tiddler> {
        tracing::debug!("getting tiddler: {}", title);
        self.tiddlers.get(title).cloned()
    }

    pub(crate) fn put(&mut self, tiddler: Tiddler) {
        tracing::debug!("putting tiddler: {}", tiddler.title);
        let title = tiddler.title.clone();
        self.tiddlers.insert(title, tiddler);
    }

    pub(crate) fn pop(&mut self, title: &str) -> Option<Tiddler> {
        tracing::debug!("popping tiddler: {}", title);
        self.tiddlers.remove(title)
    }
}

#[derive(Clone, Serialize)]
pub(crate) struct Tiddler {
    title: String,
    revision: u64,
    meta: serde_json::Value,
}

impl Tiddler {
    pub(crate) fn as_value(&self) -> Value {
        let mut meta = self.meta.clone();
        meta["title"] = Value::String(self.title.clone());
        meta["revision"] = Value::Number(Number::from(self.revision));
        meta
    }

    pub(crate) fn from_value(value: Value) -> anyhow::Result<Tiddler> {
        let obj = match value.clone() {
            Value::Object(m) => m,
            _ => anyhow::bail!("from_value expects a JSON Object"),
        };
        let title = match obj.get("title") {
            Some(Value::String(s)) => s,
            _ => anyhow::bail!("tiddler['title'] should be a string"),
        };
        let revision = match obj.get("revision") {
            Some(Value::Number(n)) => n
                .as_u64()
                .ok_or_else(|| anyhow::anyhow!("revision should be a u64 (not {})", n))?,
            Some(Value::String(s)) => s
                .parse::<u64>()
                .map_err(|_| anyhow::anyhow!("couldn't parse a revision number from '{}'", s))?,
            None => 0,
            _ => anyhow::bail!("tiddler['revision'] should be a number"),
        };
        let tiddler = Tiddler {
            title: title.clone(),
            revision,
            meta: value,
        };
        Ok(tiddler)
    }
}

// -----------------------------------------------------------------------------------
// Utility functions

async fn handle_io_error(err: std::io::Error) -> (StatusCode, String) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("Internal Server Error: {}", err),
    )
}

// -----------------------------------------------------------------------------------
// Static Status

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
