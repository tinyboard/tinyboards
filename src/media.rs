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
//use futures::stream::{Stream, StreamExt};
use tinyboards_api_common::utils::{get_user_view_from_jwt, blocking, require_user, decode_base64_image};
use tinyboards_api_common::data::TinyBoardsContext;
use tinyboards_db::models::site::site::Site;
use tinyboards_utils::{rate_limit::RateLimitCell, REQWEST_TIMEOUT};
use reqwest_middleware::{ClientWithMiddleware, RequestBuilder};
use serde::{Deserialize, Serialize};
use reqwest::multipart::Part;

pub fn config(
    cfg: &mut web::ServiceConfig,
    client: ClientWithMiddleware,
    rate_limit: &RateLimitCell,
) {
    cfg
    .app_data(web::Data::new(client))
    .service(
        web::resource("/image")
        .wrap(rate_limit.image())
        .route(web::post().to(upload)),
    )
    .service(
        web::resource("/image/{filename}")
        .route(web::get().to(full_res)),
    )
    .service(
        web::resource("/image/delete/{token}/{filename}")
        .route(web::get().to(delete))
    );
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Image {
    file: String,
    delete_token: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Images {
    msg: String,
    files: Option<Vec<Image>>,
    url: Option<String>,
    delete_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct UploadRequest {
    image: Option<String>,
    url: Option<String>,
}

#[derive(Deserialize)]
struct PictrsParams {
    format: Option<String>,
    thumbnail: Option<i32>,
    blur: Option<i32>,
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

        if let Some(blur) = params.blur {
            url = format!("{}&blur={}", url, blur);
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
    data: web::Json<UploadRequest>,
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

    if data.image.is_some() && data.url.is_some() {
        return Err(ErrorBadRequest("you can't input both a base64 string and a url to upload an image"));
    }

    if let Some(img_str_b64) = &data.image {    

        let pictrs_conf = context.settings().pictrs_config()?;
        let image_url = format!("{}image", pictrs_conf.url);
        let (img_bytes, file_name) = decode_base64_image(img_str_b64.to_owned())?;
        let img_part = Part::bytes(img_bytes).file_name(file_name);
        let form = reqwest::multipart::Form::new()
            .part("images[]", img_part);
    
        let res = client
            .post(&image_url)
            .multipart(form)
            .send()
            .await
            .map_err(ErrorBadRequest)?;
    
        let status = res.status();
        let mut images = res.json::<Images>().await.map_err(ErrorBadRequest)?;
        
        if let Some(files) = &images.files {
            images.url = Some(format!("{}/image/{}", context.settings().get_protocol_and_hostname(), files[0].file));
            images.delete_url = Some(format!("{}/image/delete/{}/{}", context.settings().get_protocol_and_hostname(), files[0].delete_token, files[0].file));
        }
        
        Ok(HttpResponse::build(status).json(images))
    } else if let Some(url) = &data.url {
        
        let pictrs_conf = context.settings().pictrs_config()?;
        let image_download_url = format!("{}image/download?url={}", pictrs_conf.url, url);

        let res = client
            .get(&image_download_url)
            .send()
            .await
            .map_err(ErrorBadRequest)?;
        
        let status = res.status();
        let mut images = res.json::<Images>().await.map_err(ErrorBadRequest)?;

        if let Some(files) = &images.files {
            images.url = Some(format!("{}/image/{}", context.settings().get_protocol_and_hostname(), files[0].file));
            images.delete_url = Some(format!("{}/image/delete/{}/{}", context.settings().get_protocol_and_hostname(), files[0].delete_token, files[0].file));
        }
        
        Ok(HttpResponse::build(status).json(images))
    } else {
        return Err(ErrorBadRequest("b64 image or url not provided"));
    }
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

// fn make_send<S>(mut stream: S) -> impl Stream<Item = S::Item> + Send + Unpin + 'static 
//     where
//         S: Stream + Unpin + 'static,
//         S::Item: Send,
// {
//     let (tx, rx) = tokio::sync::mpsc::channel(8);

//     actix_web::rt::spawn(async move {
//         while let Some(res) = stream.next().await {
//             if tx.send(res).await.is_err() {
//                 break;
//             }
//         }
//     });

//     SendStream { rx }
// }

// struct SendStream<T> {
//     rx: tokio::sync::mpsc::Receiver<T>,
// }

// impl<T> Stream for SendStream<T>
// where
//     T: Send,
// {
//     type Item = T;

//     fn poll_next(
//         mut self: std::pin::Pin<&mut Self>,
//         cx: &mut std::task::Context<'_>,
//     ) -> std::task::Poll<Option<Self::Item>> {
//         std::pin::Pin::new(&mut self.rx).poll_recv(cx)
//     }
// }