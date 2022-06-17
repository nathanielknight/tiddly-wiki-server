// NOTE(nknight): This is the API I'm trying to conform to: https://tiddlywiki.com/#WebServer%20API
// TODO: serve static files: https://tiddlywiki.com/#WebServer%20API%3A%20Get%20File
// TODO: render main wiki

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

    let datastore = initialize_datastore().expect("Error initializing datastore");
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
        .layer(Extension(datastore));

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .expect("Error running server");
}

fn initialize_datastore() -> AppResult<DataStore> {
    let init_script = include_str!("./init.sql");
    let cxn = rusqlite::Connection::open("./tiddlers.sqlite3").map_err(AppError::from)?;
    cxn.execute_batch(init_script).map_err(AppError::from)?;
    let tiddlers = Tiddlers { cxn };
    Ok(Arc::new(Mutex::new(tiddlers)))
}

// -----------------------------------------------------------------------------------
// Views
#[axum_macros::debug_handler]
async fn all_tiddlers(
    Extension(ds): Extension<DataStore>,
) -> AppResult<axum::Json<Vec<serde_json::Value>>> {
    let mut lock = ds.lock().expect("failed to lock tiddlers");
    let tiddlers = &mut *lock;
    let all: Vec<serde_json::Value> = tiddlers.all()?.iter().map(|t| t.as_value()).collect();
    Ok(axum::Json(all))
}

#[axum_macros::debug_handler]
async fn get_tiddler(
    Extension(ds): Extension<DataStore>,
    extract::Path(title): extract::Path<String>,
) -> AppResult<axum::http::Response<String>> {
    use serde_json::ser::to_string_pretty;

    let mut lock = ds.lock().expect("failed to lock tiddlers");
    let tiddlers = &mut *lock;

    if let Some(t) = tiddlers.get(&title)? {
        let body = to_string_pretty(&t.as_value())
            .map_err(|e| AppError::Serialization(format!("error serializing tiddler: {}", e)))?;
        axum::response::Response::builder()
            .status(StatusCode::OK)
            .header("content-type", "application/json")
            .body(body)
            .map_err(|e| AppError::Response(format!("error building response: {}", e)))
    } else {
        let body = String::new();
        axum::response::Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(body)
            .map_err(|e| AppError::Response(format!("error building 404 response: {}", e)))
    }
}

async fn delete_tiddler(
    Extension(ds): Extension<DataStore>,
    extract::Path(title): extract::Path<String>,
) -> AppResult<axum::response::Response<String>> {
    let mut lock = ds.lock().expect("failed to lock tiddlers");
    let tiddlers = &mut *lock;
    tiddlers.pop(&title)?;

    let mut resp = axum::response::Response::default();
    *resp.status_mut() = StatusCode::NO_CONTENT;
    Ok(resp)
}

async fn put_tiddler(
    Extension(ds): Extension<DataStore>,
    extract::Json(v): extract::Json<serde_json::Value>,
    extract::Path(title): extract::Path<String>,
) -> AppResult<axum::http::Response<String>> {
    use axum::http::response::Response;
    let mut new_tiddler = Tiddler::from_value(v)?;
    let mut lock = ds.lock().expect("failed to lock tiddlers");
    let tiddlers = &mut *lock;

    if let Some(_old_tiddler) = tiddlers.pop(&title)? {
        new_tiddler.revision += 1;
    }
    let new_revision = new_tiddler.revision;
    tiddlers.put(new_tiddler)?;
    Response::builder()
        .status(StatusCode::NO_CONTENT)
        .header("Etag", format!("default/{}/{}:", title, new_revision))
        .body(String::new())
        .map_err(|e| AppError::Response(format!("Error building response: {}", e)))
}

// -----------------------------------------------------------------------------------
// Models and serialization/parsing

pub(crate) struct Tiddlers {
    cxn: rusqlite::Connection,
}

impl Tiddlers {
    pub(crate) fn all(&self) -> AppResult<Vec<Tiddler>> {
        tracing::debug!("Retrieving all tiddlers");
        const GET: &str = r#"
            SELECT title, revision, meta FROM tiddlers
        "#;
        let mut stmt = self.cxn.prepare_cached(GET).map_err(AppError::from)?;
        let raw_tiddlers = stmt
            .query_map([], |r| r.get::<usize, serde_json::Value>(2))
            .map_err(AppError::from)?;
        let mut tiddlers = Vec::new();
        for qt in raw_tiddlers {
            let raw = qt.map_err(AppError::from)?;
            let tiddler = Tiddler::from_value(raw)?;
            tiddlers.push(tiddler);
        }
        Ok(tiddlers)
    }

    pub(crate) fn get(&self, title: &str) -> AppResult<Option<Tiddler>> {
        use rusqlite::OptionalExtension;

        tracing::debug!("getting tiddler: {}", title);

        const GET: &str = r#"
            SELECT title, revision, meta FROM tiddlers
            WHERE title = ?
        "#;
        let raw = self
            .cxn
            .query_row(GET, [title], |r| r.get::<usize, serde_json::Value>(2))
            .optional()
            .map_err(|e| AppError::Database(format!("Error retrieving '{}': {}", title, e)))?;
        raw.map(Tiddler::from_value).transpose()
    }

    pub(crate) fn put(&mut self, tiddler: Tiddler) -> AppResult<()> {
        tracing::debug!("putting tiddler: {}", tiddler.title);
        const PUT: &str = r#"
            INSERT INTO tiddlers (title, revision, meta) VALUES (:title, :revision, :meta)
            ON CONFLICT (title) DO UPDATE
            SET title = :title, revision = :revision, meta = :meta
        "#;
        let mut stmt = self
            .cxn
            .prepare_cached(PUT)
            .map_err(|e| AppError::Database(format!("Error preparing statement: {}", e)))?;
        stmt.execute(rusqlite::named_params! {
            ":title": tiddler.title,
            ":revision": tiddler.revision,
            ":meta": tiddler.meta,
        })?;
        tracing::debug!("done");
        Ok(())
    }

    pub(crate) fn pop(&mut self, title: &str) -> AppResult<Option<Tiddler>> {
        tracing::debug!("popping tiddler: {}", title);
        let result = self.get(title)?;
        const DELETE: &str = "DELETE FROM tiddlers WHERE title = :title";
        let mut stmt = self
            .cxn
            .prepare(DELETE)
            .map_err(|e| AppError::Database(format!("Error preparing {}: {}", DELETE, e)))?;
        stmt.execute(rusqlite::named_params! { ":title": title })
            .map_err(|e| AppError::Database(format!("Error removing tiddler: {}", e)))?;
        Ok(result)
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

    pub(crate) fn from_value(value: Value) -> AppResult<Tiddler> {
        let obj = match value.clone() {
            Value::Object(m) => m,
            _ => {
                return Err(AppError::Serialization(
                    "from_value expects a JSON Object".to_string(),
                ))
            }
        };
        let title = match obj.get("title") {
            Some(Value::String(s)) => s,
            _ => {
                return Err(AppError::Serialization(
                    "tiddler['title'] should be a string".to_string(),
                ))
            }
        };
        let revision = match obj.get("revision") {
            None => 0,
            Some(Value::Number(n)) => n.as_u64().ok_or_else(|| {
                AppError::Serialization(format!("revision should be a u64 (not {})", n))
            })?,
            Some(Value::String(s)) => s.parse::<u64>().map_err(|_| {
                AppError::Serialization(format!("couldn't parse a revision number from '{}'", s))
            })?,
            _ => {
                return Err(AppError::Serialization(
                    "tiddler['revision'] should be a number".to_string(),
                ))
            }
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

// -----------------------------------------------------------------------------------
// Error handling

type AppResult<T> = Result<T, AppError>;

#[derive(Debug)]
enum AppError {
    Database(String),
    Response(String),
    Serialization(String),
}

impl axum::response::IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        tracing::error!("{:?}", self);
        let msg = match self {
            AppError::Database(msg) => msg,
            AppError::Response(msg) => msg,
            AppError::Serialization(msg) => msg,
        };
        (StatusCode::INTERNAL_SERVER_ERROR, msg).into_response()
    }
}

impl From<rusqlite::Error> for AppError {
    fn from(err: rusqlite::Error) -> AppError {
        tracing::error!("{:?}", err);
        let msg = err.to_string();
        AppError::Database(msg)
    }
}
