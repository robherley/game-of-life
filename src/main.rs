use axum::{
    extract::{Path, Query},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Redirect},
    routing::{get, post},
    Extension, Router,
};
use game_of_life::{
    db::{self, StoreError},
    game::{Board, Game},
    render::{self, SVGOptions, TextOptions},
};
use serde::Deserialize;
use tower_http::trace::{self, TraceLayer};
use tracing::{info, warn, Level};

macro_rules! fail {
    ($c:expr, $e:expr) => {
        return ($c, HeaderMap::new(), $e.to_string())
    };
}

#[derive(Deserialize, Debug)]
struct RenderParams {
    next: Option<bool>,
    alive: Option<char>,
    dead: Option<char>,
    sepatator: Option<char>,
    cell_size: Option<usize>,
    stroke_width: Option<usize>,
    stroke_color: Option<String>,
    fill_color: Option<String>,
}

impl From<RenderParams> for SVGOptions {
    fn from(p: RenderParams) -> Self {
        SVGOptions::new(p.cell_size, p.stroke_width, p.stroke_color, p.fill_color)
    }
}

impl From<RenderParams> for TextOptions {
    fn from(p: RenderParams) -> Self {
        TextOptions::new(p.alive, p.dead, p.sepatator)
    }
}

async fn render(
    Extension(store): Extension<db::Store>,
    Path(game): Path<String>,
    params: Query<RenderParams>,
) -> impl IntoResponse {
    let ext = game.split('.').last().unwrap_or("txt");
    let game = game.trim_end_matches(&format!(".{}", ext));

    let mut board = match store.find(game) {
        Ok(Some(b)) => b,
        Ok(None) => fail!(StatusCode::NOT_FOUND, format!("game '{}' not found", game)),
        Err(e) => fail!(StatusCode::INTERNAL_SERVER_ERROR, e),
    };

    if params.next.unwrap_or(false) {
        board.next();
        match store.update(game, &board) {
            Ok(_) => (),
            Err(e) => fail!(StatusCode::INTERNAL_SERVER_ERROR, e),
        }
    }

    let mut headers: HeaderMap<HeaderValue> = HeaderMap::new();
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("no-cache, no-store"),
    );
    headers.insert(
        header::EXPIRES,
        HeaderValue::from_static("Thu, 01 Jan 1970 00:00:00 GMT"),
    );
    headers.insert(header::ETAG, HeaderValue::from(board.generation));
    headers.insert("x-life-generation", HeaderValue::from(board.generation));
    headers.insert("x-life-delta", HeaderValue::from(board.delta));
    headers.insert(
        header::ACCESS_CONTROL_ALLOW_ORIGIN,
        HeaderValue::from_static("*"),
    );

    match ext {
        "svg" => {
            headers.insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static("image/svg+xml"),
            );
            let svg = match render::svg(&board, params.0.into()) {
                Ok(svg) => svg,
                Err(e) => fail!(StatusCode::INTERNAL_SERVER_ERROR, e),
            };

            return (StatusCode::OK, headers, svg);
        }
        _ => {
            headers.insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static("text/plain; charset=utf-8"),
            );
            let text = render::text(&board, params.0.into());
            return (StatusCode::OK, headers, text);
        }
    }
}

#[derive(Deserialize, Debug)]
struct CreatorParams {
    alive: Option<char>,
    dead: Option<char>,
    sepatator: Option<char>,
}

impl From<CreatorParams> for TextOptions {
    fn from(p: CreatorParams) -> Self {
        TextOptions::new(p.alive, p.dead, p.sepatator)
    }
}

async fn creator(
    Extension(store): Extension<db::Store>,
    Path(name): Path<String>,
    params: Query<CreatorParams>,
    body: String,
) -> impl IntoResponse {
    if !name.chars().all(|c| c.is_alphanumeric() || c == '-') {
        fail!(
            StatusCode::BAD_REQUEST,
            "game name must be alphanumeric or '-'"
        );
    }

    let opts: TextOptions = params.0.into();
    let board = match Board::from_seed(body, opts.alive, opts.dead, opts.separator) {
        Ok(b) => b,
        Err(e) => fail!(StatusCode::BAD_REQUEST, e),
    };

    let mut headers: HeaderMap<HeaderValue> = HeaderMap::new();
    headers.insert(
        header::ACCESS_CONTROL_ALLOW_ORIGIN,
        HeaderValue::from_static("*"),
    );

    let game = Game::from(board);
    match store.create(&name, &game) {
        Ok(_) => (),
        Err(StoreError::SQLError(rusqlite::Error::SqliteFailure(e, _)))
            if e.code == rusqlite::ErrorCode::ConstraintViolation =>
        {
            fail!(
                StatusCode::CONFLICT,
                format!("game '{}' already exists", name)
            );
        }
        Err(StoreError::BoardError(e)) => fail!(StatusCode::BAD_REQUEST, e),
        Err(e) => fail!(StatusCode::INTERNAL_SERVER_ERROR, e),
    }

    (
        StatusCode::CREATED,
        headers,
        render::text(&game, Default::default()),
    )
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().compact().init();

    let db_path: String = match std::env::var("DB_PATH") {
        Ok(p) => p,
        Err(_) => {
            warn!("DB_PATH not set, using in-memory database");
            ":memory:".into()
        }
    };
    info!("database: {}", db_path);

    let database = db::Store::new(db_path)?;
    database.migrate()?;

    let router = Router::new()
        .route(
            "/",
            get(|| async { Redirect::to("https://github.com/robherley/game-of-life") }),
        )
        .route(
            "/favicon.ico",
            get(|| async { StatusCode::NOT_FOUND.into_response() }),
        )
        .route("/:name", get(render))
        .route("/:name", post(creator))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        )
        .layer(Extension(database));

    let addr = format!(
        "{}:{}",
        std::env::var("HOST").unwrap_or("0.0.0.0".into()),
        std::env::var("PORT").unwrap_or("8080".into())
    );

    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!("listening on {}", listener.local_addr()?);
    axum::serve(listener, router).await?;

    Ok(())
}
