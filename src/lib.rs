pub mod game;
pub mod render;

use game::{Board, Game};
use http::{header, HeaderMap, HeaderValue, StatusCode};
use render::{SVGOptions, TextOptions};
use serde::Deserialize;
use worker::*;

const KV_NAMESPACE: &str = "games";

macro_rules! fail {
    ($c:expr, $e:expr) => {
        return Response::error($e.to_string(), $c.into())
    };
}

macro_rules! build_headers {
    ($($k:expr => $v:expr),*) => {
        {
            let mut map = HeaderMap::new();
            $(map.insert($k, HeaderValue::from($v));)*
            map
        }
    };
}

#[derive(Deserialize, Debug)]
struct RenderParams {
    next: Option<bool>,
    alive: Option<char>,
    dead: Option<char>,
    separator: Option<char>,
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
        TextOptions::new(p.alive, p.dead, p.separator)
    }
}

async fn render(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let name = match ctx.param("name") {
        Some(n) => n,
        None => fail!(StatusCode::BAD_REQUEST, "name is required"),
    };

    let ext = name.split('.').last().unwrap_or("txt");
    let name = name.trim_end_matches(&format!(".{}", ext));

    let kv = match ctx.env.kv(KV_NAMESPACE) {
        Ok(kv) => kv,
        Err(e) => fail!(StatusCode::INTERNAL_SERVER_ERROR, e),
    };

    let mut game = match kv.get(name).json::<Game>().await {
        Ok(Some(g)) => g,
        Ok(None) => fail!(
            StatusCode::NOT_FOUND,
            format!("game '{}' does not exist", name)
        ),
        Err(e) => fail!(StatusCode::INTERNAL_SERVER_ERROR, e),
    };

    let params = match req.query::<RenderParams>() {
        Ok(p) => p,
        Err(e) => fail!(StatusCode::BAD_REQUEST, e),
    };

    if params.next.unwrap_or(false) {
        game.next();
        if let Err(e) = kv.put(name, &game)?.execute().await {
            fail!(StatusCode::INTERNAL_SERVER_ERROR, e);
        }
    }

    let headers = build_headers! {
        header::ETAG => game.generation,
        "x-life-generation" => game.generation,
        "x-life-delta" => game.delta
    };

    let res = ResponseBuilder::new().with_headers(headers.into());

    match ext {
        "svg" => {
            let svg = match render::svg(&game, params.into()) {
                Ok(svg) => svg,
                Err(e) => fail!(StatusCode::INTERNAL_SERVER_ERROR, e),
            };
            Ok(res
                .with_header(header::CONTENT_TYPE.as_str(), "image/svg+xml")?
                .fixed(svg.into()))
        }
        _ => {
            let text = render::text(&game, params.into());
            res.with_header(header::CONTENT_TYPE.as_str(), "text/plain; charset=utf-8")?
                .ok(text)
        }
    }
}

#[derive(Deserialize, Debug)]
struct CreatorParams {
    alive: Option<char>,
    dead: Option<char>,
    separator: Option<char>,
}

async fn create(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let name = match ctx.param("name") {
        Some(n) => n,
        None => fail!(StatusCode::BAD_REQUEST, "name is required"),
    };

    if !name.chars().all(|c| c.is_alphanumeric() || c == '-') {
        fail!(
            StatusCode::BAD_REQUEST,
            "game name must be alphanumeric or '-'"
        );
    }

    let params = match req.query::<CreatorParams>() {
        Ok(p) => p,
        Err(e) => fail!(StatusCode::BAD_REQUEST, e),
    };

    let body = match req.text().await {
        Ok(b) => b,
        Err(e) => fail!(StatusCode::BAD_REQUEST, e),
    };

    let board = match Board::from_seed(body, params.alive, params.dead, params.separator) {
        Ok(b) => b,
        Err(e) => fail!(StatusCode::BAD_REQUEST, e),
    };

    let kv = match ctx.env.kv(KV_NAMESPACE) {
        Ok(kv) => kv,
        Err(e) => fail!(StatusCode::INTERNAL_SERVER_ERROR, e),
    };

    let game_exists = match kv.get(name).text().await {
        Ok(Some(_)) => true,
        Ok(None) => false,
        Err(_) => false,
    };

    if game_exists {
        fail!(
            StatusCode::CONFLICT,
            format!("game '{}' already exists", name)
        );
    }

    let game = Game::from(board);
    if let Err(e) = kv.put(name, &game)?.execute().await {
        fail!(StatusCode::INTERNAL_SERVER_ERROR, e);
    }

    ResponseBuilder::new()
        .with_status(StatusCode::CREATED.into())
        .ok(render::text(&game, Default::default()))
}

#[event(fetch)]
async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    console_error_panic_hook::set_once();

    let mut response = Router::new()
        .get("/", |_, _| {
            let url = "https://github.com/robherley/game-of-life".parse()?;
            Response::redirect(url)
        })
        .get("/favicon.ico", |_, _| {
            fail!(StatusCode::NOT_FOUND, "not found")
        })
        .get("/_ping", |_, _| Response::ok("pong"))
        .get_async("/:name", render)
        .post_async("/:name", create)
        .run(req, env)
        .await?;

    [
        (header::ACCESS_CONTROL_ALLOW_ORIGIN, "*"),
        (header::CACHE_CONTROL, "no-cache, no-store"),
        (header::EXPIRES, "Thu, 01 Jan 1970 00:00:00 GMT"),
    ]
    .iter()
    .for_each(|(k, v)| {
        let _ = response.headers_mut().set(k.as_str(), v);
    });

    Ok(response)
}
