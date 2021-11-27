#![allow(non_snake_case)]
use dotenv;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::env;
use tide::http::mime;
use tide::log::debug;
use tide::utils::After;
use tide::{Request, Response, Result, StatusCode};

#[derive(Debug, Deserialize, Serialize)]
struct E11tyConfig {
    path: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct E11tyResponse {
    statusCode: u16,
    headers: Value,
    body: String,
}

#[async_std::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    const NOT_FOUND_HTML_PAGE: &str = "<html><body>
    <h1>uh oh, we couldn't find that url</h1>
    <p>
      probably, this would be served from the file system or
      included with `include_bytes!`
    </p>
  </body></html>";

    const INTERNAL_SERVER_ERROR_HTML_PAGE: &str = "<html><body>
    <h1>whoops! it's not you, it's us</h1>
    <p>
      we're very sorry, but something seems to have gone wrong on our end
    </p>
  </body></html>";
    tide::log::start();
    let mut app = tide::new();
    app.with(After(|response: Response| async move {
        debug!("{:?}", response);
        let response = match response.status() {
            StatusCode::NotFound => Response::builder(404)
                .content_type(mime::HTML)
                .body(NOT_FOUND_HTML_PAGE)
                .build(),
            StatusCode::InternalServerError => Response::builder(500)
                .content_type(mime::HTML)
                .body(INTERNAL_SERVER_ERROR_HTML_PAGE)
                .build(),

            _ => response,
        };

        Ok(response)
    }));

    app.at("/*").get(index);
    let addr = env::var("ADDRESS").unwrap_or("0.0.0.0:8090".to_string());
    app.listen(addr).await?;
    Ok(())
}

pub async fn index(req: Request<()>) -> tide::Result {
    let path = req.url().path();
    let e11ty_conf = E11tyConfig {
        path: path.to_string(),
    };

    let uri = env::var("11TY_SERVICE")
        .unwrap_or("http://127.0.0.1:8080/2015-03-31/functions/function/invocations".to_string());
    let res: E11tyResponse = surf::post(uri).body_json(&e11ty_conf)?.recv_json().await?;
    let mut return_response = Response::new(res.statusCode);
    return_response.set_body(res.body);
    match res.headers.as_object() {
        Some(s) => {
            for (key, value) in s {
                println!("key:{}, value: {}", key, value);
                return_response.insert_header(key.as_str(), value.as_str().unwrap_or_default())
            }
        }
        None => {}
    }
    Ok(return_response)
}
