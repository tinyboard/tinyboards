use actix_web::{
    body::BodyStream,
    error::ErrorBadRequest,
    http::{
        header::{HeaderName, ACCEPT_ENCODING, HOST},
        StatusCode,
    },
    web,
    Error,
    HttpRequest,
    HttpResponse,
};
use futures::stream::{Stream, StreamExt};
use tinyboards_api_common::utils::{get_user_view_from_jwt, blocking, require_user};
use tinyboards_api_common::data::TinyBoardsContext;
use tinyboards_db::models::site::site::Site;
use tinyboards_utils::{rate_limit::RateLimitCell, REQWEST_TIMEOUT};
use reqwest::Body;
use reqwest_middleware::{ClientWithMiddleware, RequestBuilder};
use serde::{Deserialize, Serialize};

pub fn config(
    cfg: &mut web::ServiceConfig,
    client: ClientWithMiddleware,
    rate_limit: &RateLimitCell,
) {
    cfg
    .app_data(web::Data::new(client))
    .service(
        web::resource("/pictrs/image")
        .wrap(rate_limit.image())
        .route(web::post().to(upload)),
    )
    .service(
        web::resource("/pictrs/image/{filename}")
        .route(web::get().to(full_res)),
    )
    .service(
        web::resource("/pictrs/image/delete/{token}/{filename}")
        .route(web::get().to(delete))
    );
}

#[derive(Debug, Serialize, Deserialize)]
struct Image {
    file: String,
    delete_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Images {
    msg: String,
    files: Option<Vec<Image>>,
}

#[derive(Deserialize)]
struct PictrsParams {
    format: Option<String>,
    thumbnail: Option<i32>,
}

#[derive(Deserialize)]
enum PictrsPurgeParams {
    #[serde(rename = "file")]
    File(String),
    #[serde(rename = "alias")]
    Alias(String),
}

fn adapt_request(
    request: &HttpRequest,
    client: &ClientWithMiddleware,
    url: String,
) -> RequestBuilder {
    const INVALID_HEADERS: &[HeaderName] = &[ACCEPT_ENCODING, HOST];

    let client_request = client
        .request(request.method().clone(), url)
        .timeout(REQWEST_TIMEOUT);

    request
        .headers()
        .iter()
        .fold(client_request, |client_req, (key, value)| {
            if INVALID_HEADERS.contains(key) {
                client_req
            } else {
                client_req.header(key, value)
            }
        })
}

async fn full_res(
    filename: web::Path<String>,
    web::Query(params): web::Query<PictrsParams>,
    req: HttpRequest,
    client: web::Data<ClientWithMiddleware>,
    context: web::Data<TinyBoardsContext>,
) -> Result<HttpResponse, Error> {
    
    let site = blocking(context.pool(), move |conn| {
        Site::read_local(conn)
    })
    .await?
    .map_err(ErrorBadRequest)?;

    if site.private_instance {
        let jwt = req
            .headers()
            .get("Authorization")
            .unwrap()
            .to_str()
            .unwrap();

        if get_user_view_from_jwt(Some(jwt), context.pool(), context.master_key())
            .await
            .is_err()
        {
            return Ok(HttpResponse::Unauthorized().finish());
        };
    }

    let name = &filename.into_inner();

    let pictrs_conf = context.settings().pictrs_config()?;
    let url = if params.format.is_none() && params.thumbnail.is_none() {
        format!("{}image/original/{}", pictrs_conf.url, name,)
    } else {
        let format = params
            .format
            .unwrap_or_else(|| name.split('.').last().unwrap_or("jpg").to_string());
        
        let mut url = format!("{}image/process.{}?src={}", pictrs_conf.url, format, name,);

        if let Some(size) = params.thumbnail {
            url = format!("{}&thumbnail={}", url, size);
        }
        url
    };
    
    image(url, req, client).await
}

async fn image(
    url: String,
    req: HttpRequest,
    client: web::Data<ClientWithMiddleware>,
) -> Result<HttpResponse, Error> {
    let mut client_req = adapt_request(&req, &client, url);

    if let Some(addr) = req.head().peer_addr {
        client_req = client_req.header("X-Forwarded-For", addr.to_string());
    }

    if let Some(addr) = req.head().peer_addr {
        client_req = client_req.header("X-Forwarded-For", addr.to_string());
    }

    let res = client_req.send().await.map_err(ErrorBadRequest)?;

    if res.status() == StatusCode::NOT_FOUND {
        return Ok(HttpResponse::NotFound().finish());
    }

    let mut client_res = HttpResponse::build(res.status());

    for (name, value) in res.headers().iter().filter(|(h, _)| *h != "connection") {
        client_res.insert_header((name.clone(), value.clone()));
    }

    Ok(client_res.body(BodyStream::new(res.bytes_stream())))
}

async fn upload(
    req: HttpRequest,
    body: web::Payload,
    client: web::Data<ClientWithMiddleware>,
    context: web::Data<TinyBoardsContext>
) -> Result<HttpResponse, Error> {
    
    let auth = req
        .headers()
        .get("Authorization")
        .unwrap()
        .to_str()
        .unwrap();

    require_user(context.pool(), context.master_key(), Some(auth))
    .await
    .unwrap()?;

    let pictrs_conf = context.settings().pictrs_config()?;

    let image_url = format!("{}image", pictrs_conf.url);

    let mut client_req = adapt_request(&req, &client, image_url);

    if let Some(addr) = req.head().peer_addr {
        client_req = client_req.header("X-Forwarded-For", addr.to_string());
    };

    let res = client_req
        .body(Body::wrap_stream(make_send(body)))
        .send()
        .await
        .map_err(ErrorBadRequest)?;

    let status = res.status();
    let images = res.json::<Images>().await.map_err(ErrorBadRequest)?;

    Ok(HttpResponse::build(status).json(images))
}

async fn delete(
    components: web::Path<(String, String)>,
    req: HttpRequest,
    client: web::Data<ClientWithMiddleware>,
    context: web::Data<TinyBoardsContext>,
  ) -> Result<HttpResponse, Error> {
    let (token, file) = components.into_inner();
  
    let pictrs_conf = context.settings().pictrs_config()?;
    let url = format!("{}image/delete/{}/{}", pictrs_conf.url, &token, &file);
  
    let mut client_req = adapt_request(&req, &client, url);
  
    if let Some(addr) = req.head().peer_addr {
      client_req = client_req.header("X-Forwarded-For", addr.to_string());
    }
  
    let res = client_req.send().await.map_err(ErrorBadRequest)?;
  
    Ok(HttpResponse::build(res.status()).body(BodyStream::new(res.bytes_stream())))
  }

fn make_send<S>(mut stream: S) -> impl Stream<Item = S::Item> + Send + Unpin + 'static 
    where
        S: Stream + Unpin + 'static,
        S::Item: Send,
{
    let (tx, rx) = tokio::sync::mpsc::channel(8);

    actix_web::rt::spawn(async move {
        while let Some(res) = stream.next().await {
            if tx.send(res).await.is_err() {
                break;
            }
        }
    });

    SendStream { rx }
}

struct SendStream<T> {
    rx: tokio::sync::mpsc::Receiver<T>,
}

impl<T> Stream for SendStream<T>
where
    T: Send,
{
    type Item = T;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        std::pin::Pin::new(&mut self.rx).poll_recv(cx)
    }
}