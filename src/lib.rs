use std::collections::HashMap;

use worker::*;

mod utils;

fn log_request(req: &Request) {
    console_log!(
        "{} - [{}], located at: {:?}, within: {}",
        Date::now().to_string(),
        req.path(),
        req.cf().coordinates().unwrap_or_default(),
        req.cf().region().unwrap_or_else(|| "unknown region".into())
    );
}

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    log_request(&req);

    // Optionally, get more helpful error messages written to the console in the case of a panic.
    utils::set_panic_hook();

    // Optionally, use the Router to handle matching endpoints, use ":name" placeholders, or "*name"
    // catch-alls to match on specific patterns. Alternatively, use `Router::with_data(D)` to
    // provide arbitrary data that will be accessible in each route via the `ctx.data()` method.
    let router = Router::new();

    // Add as many routes as your Worker needs! Each route will get a `Request` for handling HTTP
    // functionality and a `RouteContext` which you can use to  and get route parameters and
    // Environment bindings like KV Stores, Durable Objects, Secrets, and Variables.
    router
        .get_async("/", |req, _ctx| async move {
            // Get the value of the `color1` and `color2` query parameters from the URL.
            let url = req.url()?;
            let queries: HashMap<_, _> = url.query_pairs().collect();

            if let Some(color1) = queries.get("color1") {
                if let Some(color2) = queries.get("color2") {
                    // If both `color1` and `color2` are present, return a gradient with those colors.
                    let gradient = utils::generate_image(
                        color1.trim_start_matches('#'),
                        color2.trim_start_matches('#'),
                    );

                    let mut headers = Headers::new();
                    headers.set("Content-Type", "image/png")?;
                    headers.set("Content-Disposition", "inline")?;
                    headers.set("Content-Length", &gradient.len().to_string())?;

                    let res = Response::from_bytes(gradient)?.with_headers(headers);

                    return Ok(res);
                }
            }

            Response::empty()
        })
        .get_async("/random", |_req, _ctx| async move {
            // Generate a random gradient and return it.
            let color1 = utils::generate_random_hex_color();
            let color2 = utils::generate_random_hex_color();

            let gradient = utils::generate_image(&color1, &color2);

            let mut headers = Headers::new();
            headers.set("Content-Type", "image/png")?;
            headers.set("Content-Disposition", "inline")?;
            headers.set("Content-Length", &gradient.len().to_string())?;
            headers.set("Cache-Control", "no-cache, no-store")?;

            let res = Response::from_bytes(gradient)?.with_headers(headers);

            Ok(res)
        })
        .get("/worker-version", |_, ctx| {
            let version = ctx.var("WORKERS_RS_VERSION")?.to_string();
            Response::ok(version)
        })
        .run(req, env)
        .await
}
