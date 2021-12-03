#![allow(non_snake_case)]
use once_cell::sync::Lazy;
use route_recognizer::{Match, Params, Router};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::env;
use std::fs;
use tide::http::mime;
use tide::log::{debug, info};
use tide::utils::After;
use tide::{Request, Response, Result, StatusCode};

#[derive(Debug, Deserialize, Serialize)]
struct E11tyConfig {
    path: String,
    httpMethod: String,
    queryStringParameters: Map<String, Value>,
}

#[derive(Debug, Deserialize, Serialize)]
struct E11tyResponse {
    statusCode: u16,
    headers: Value,
    body: String,
}

#[derive(Clone, Debug)]
pub struct State {
    routemap: Router<String>,
    headermap: Router<HashMap<String, String>>,
}

#[derive(Default, Debug, Deserialize, Serialize)]
struct ServiceConfig {
    redirects: Option<Vec<Redirect>>,
    build: Option<Build>,
    headers: Option<Vec<Headers>>,
    errorpages: Option<ErrorPages>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Redirect {
    from: String,
    to: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct ErrorPages {
    not_found: Option<String>,
    internal_server_error: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Headers {
    r#for: String,
    values: HashMap<String, String>,
}

#[derive(Default, Debug, Deserialize, Serialize)]
struct Build {
    publish: Option<String>,
}

#[async_std::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    let toml_config = fs::read_to_string("./knative.toml").unwrap_or_default();
    let svc_config: ServiceConfig = toml::from_str(toml_config.as_str()).unwrap_or_default();
    debug!("{:?}", svc_config);

    static ERROR_PAGES: Lazy<HashMap<u32, String>> = Lazy::new(|| {
        let mut defnotfound = "<html><body>
        <h1>uh oh, we couldn't find that url</h1>
        <p>
          probably, this would be served from the file system or
          included with `include_bytes!`
        </p>
      </body></html>"
            .to_string();
        let mut definternalserver = "<html><body>
            <h1>whoops! it's not you, it's us</h1>
            <p>
              we're very sorry, but something seems to have gone wrong on our end
            </p>
          </body></html>"
            .to_string();

        let toml_config = fs::read_to_string("./knative.toml").unwrap_or_default();
        let svc_config: ServiceConfig = toml::from_str(toml_config.as_str()).unwrap_or_default();
        let site = match svc_config.build {
            Some(s) => s.publish.unwrap_or_default(),
            None => "".to_string(),
        };

        if let Some(errors) = svc_config.errorpages {
            if let Some(errorpage) = errors.not_found {
                let path = format!("{}/{}", site, errorpage);
                defnotfound = fs::read_to_string(path).unwrap_or_default();
            }
            if let Some(errorpage) = errors.internal_server_error {
                let path = format!("{}/{}", site, errorpage);
                definternalserver = fs::read_to_string(path).unwrap_or_default();
            }
        }

        let mut m = HashMap::new();
        m.insert(404, defnotfound);
        m.insert(500, definternalserver);
        m
    });

    let mut redirect_routes = Router::new();
    if let Some(ref rs) = svc_config.redirects {
        for r in rs {
            redirect_routes.add(r.from.as_str(), r.to.to_string())
        }
    };

    let mut header_routes = Router::new();
    for header in svc_config.headers.unwrap_or_default() {
        header_routes.add(header.r#for.as_str(), header.values);
    }

    let state = State {
        routemap: redirect_routes,
        headermap: header_routes,
    };
    let mut app = tide::with_state(state);
    tide::log::start();
    app.with(After(|response: Response| async move {
        debug!("{:?}", response);
        let response = match response.status() {
            StatusCode::NotFound => Response::builder(404)
                .content_type(mime::HTML)
                .body(String::from(ERROR_PAGES.get(&404).unwrap()))
                .build(),
            StatusCode::InternalServerError => Response::builder(500)
                .content_type(mime::HTML)
                .body(String::from(ERROR_PAGES.get(&500).unwrap()))
                .build(),

            _ => response,
        };
        Ok(response)
    }));

    if let Some(build_conf) = svc_config.build {
        if let Some(p) = build_conf.publish {
            app.at("/").serve_dir(p)?;
        }
    };

    if let Some(rs) = svc_config.redirects {
        for r in rs {
            info!("adding {}", r.from.as_str());
            app.at(r.from.as_str()).get(index);
        }
    }

    let addr = env::var("ADDRESS").unwrap_or_else(|_| "0.0.0.0:8090".to_string());
    app.listen(addr).await?;
    Ok(())
}

pub async fn index(req: Request<State>) -> tide::Result {
    let path = req.url().path();
    let method = req.method();
    let query = req.url().query_pairs();
    let mut map: Map<String, Value> = Map::new();

    for (key, value) in query {
        map.insert(key.to_string(), Value::String(value.to_string()));
    }

    let e11ty_conf = E11tyConfig {
        path: path.to_string(),
        httpMethod: method.to_string(),
        queryStringParameters: map,
    };
    debug!("{:?}", e11ty_conf);
    let state = &req.state();
    let routes = &state.routemap;
    let headers = &state.headermap;
    let h: HashMap<String, String> = HashMap::new();
    let hm = match headers.recognize(path) {
        Ok(s) => s,
        Err(_) => {
            info!("no match found using default");

            Match::new(&h, Params::new())
        }
    };

    let header_handler = hm.handler();

    let default = "http://127.0.0.1:8080/2015-03-31/functions/function/invocations".to_string();
    let m = match routes.recognize(path) {
        Ok(s) => s,
        Err(_) => {
            info!("no match found using default");
            Match::new(&default, Params::new())
        }
    };

    let uri = m.handler();
    info!("{:?}", uri);
    let res: E11tyResponse = surf::post(uri).body_json(&e11ty_conf)?.recv_json().await?;
    let mut return_response = Response::new(res.statusCode);
    return_response.set_body(res.body);
    for (key, value) in header_handler.iter() {
        return_response.insert_header(key.as_str(), value.as_str());
    }
    if let Some(s) = res.headers.as_object() {
        for (key, value) in s {
            return_response.insert_header(key.as_str(), value.as_str().unwrap_or_default())
        }
    }
    Ok(return_response)
}
