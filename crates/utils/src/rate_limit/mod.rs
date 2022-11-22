use crate::{settings::structs::RateLimitConfig, utils::get_ip, IpAddr, TinyBoardsError};
use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    HttpResponse
};
use futures::future::{ok, Ready};
use rate_limiter::{RateLimitType, RateLimiter};
use std::{
    future::Future,
    pin::Pin,
    rc::Rc,
    sync::{Arc, Mutex},
    task::{Context, Poll},
};
use tokio::sync::{mpsc, mpsc::Sender, OnceCell};

pub mod rate_limiter;

#[derive(Debug, Clone)]
pub struct RateLimit {
    pub rate_limiter: Arc<Mutex<RateLimiter>>,
    pub rate_limit_config: RateLimitConfig,
}

#[derive(Debug, Clone)]
pub struct RateLimited {
    rate_limiter: Arc<Mutex<RateLimiter>>,
    rate_limit_config: RateLimitConfig,
    type_: RateLimitType,
}

#[derive(Debug, Clone)]
pub struct RateLimitedGuard {
  rate_limit: Arc<Mutex<RateLimit>>,
  type_: RateLimitType,
}

pub struct RateLimitedMiddleware<S> {
    rate_limited: RateLimited,
    service: Rc<S>,
}

#[derive(Clone)]
pub struct RateLimitCell {
  tx: Sender<RateLimitConfig>,
  rate_limit: Arc<Mutex<RateLimit>>,
}

impl RateLimitCell {
    /// Initialize cell if it wasnt initialized yet. Otherwise returns the existing cell.
    pub async fn new(rate_limit_config: RateLimitConfig) -> &'static Self {
      static LOCAL_INSTANCE: OnceCell<RateLimitCell> = OnceCell::const_new();
      LOCAL_INSTANCE
        .get_or_init(|| async {
          let (tx, mut rx) = mpsc::channel::<RateLimitConfig>(4);
          let rate_limit = Arc::new(Mutex::new(RateLimit {
            rate_limiter: Default::default(),
            rate_limit_config,
          }));
          let rate_limit2 = rate_limit.clone();
          tokio::spawn(async move {
            while let Some(r) = rx.recv().await {
              rate_limit2
                .lock()
                .expect("Failed to lock rate limit mutex for updating")
                .rate_limit_config = r;
            }
          });
          RateLimitCell { tx, rate_limit }
        })
        .await
    }
  
    /// Call this when the config was updated, to update all in-memory cells.
    pub async fn send(&self, config: RateLimitConfig) -> Result<(), TinyBoardsError> {
      self.tx.send(config).await?;
      Ok(())
    }
  
    pub fn message(&self) -> RateLimitedGuard {
      self.kind(RateLimitType::Message)
    }
  
    pub fn post(&self) -> RateLimitedGuard {
      self.kind(RateLimitType::Post)
    }
  
    pub fn register(&self) -> RateLimitedGuard {
      self.kind(RateLimitType::Register)
    }
  
    pub fn image(&self) -> RateLimitedGuard {
      self.kind(RateLimitType::Image)
    }
  
    pub fn comment(&self) -> RateLimitedGuard {
      self.kind(RateLimitType::Comment)
    }
  
    pub fn search(&self) -> RateLimitedGuard {
      self.kind(RateLimitType::Search)
    }
  
    fn kind(&self, type_: RateLimitType) -> RateLimitedGuard {
      RateLimitedGuard {
        rate_limit: self.rate_limit.clone(),
        type_,
      }
    }
  }


impl RateLimit {

    pub fn message(&self) -> RateLimited {
        self.kind(RateLimitType::Message)
      }
    
      pub fn post(&self) -> RateLimited {
        self.kind(RateLimitType::Post)
      }
    
      pub fn register(&self) -> RateLimited {
        self.kind(RateLimitType::Register)
      }
    
      pub fn image(&self) -> RateLimited {
        self.kind(RateLimitType::Image)
      }
    
      pub fn comment(&self) -> RateLimited {
        self.kind(RateLimitType::Comment)
      }
    
      pub fn search(&self) -> RateLimited {
        self.kind(RateLimitType::Search)
      }
    
      fn kind(&self, type_: RateLimitType) -> RateLimited {
        RateLimited {
          rate_limiter: self.rate_limiter.clone(),
          rate_limit_config: self.rate_limit_config.clone(),
          type_,
        }
    }
}

impl RateLimited {
    /// Returns true if request passed the rate limit, false if it failed and should be rejected.
    pub fn check(self, ip_addr: IpAddr) -> bool {
        let rate_limit = self.rate_limit_config;

        let (kind, interval) = match self.type_ {
            
            RateLimitType::Message => (rate_limit.message, rate_limit.message_per_second),
            RateLimitType::Post => (rate_limit.post, rate_limit.post_per_second),
            RateLimitType::Register => (rate_limit.register, rate_limit.register_per_second),
            RateLimitType::Image => (rate_limit.image, rate_limit.image_per_second),
            RateLimitType::Comment => (rate_limit.comment, rate_limit.comment_per_second),
            RateLimitType::Search => (rate_limit.search, rate_limit.search_per_second),
        };

        let mut limiter = self.rate_limiter.lock().expect("mutex poison error");

        limiter.check_rate_limit_full(self.type_, &ip_addr, kind, interval)
    }
}

impl<S> Transform<S, ServiceRequest> for RateLimited
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = actix_web::Error> + 'static,
    S::Future: 'static,
{
    type Response = S::Response;
    type Error = actix_web::Error;
    type InitError = ();
    type Transform = RateLimitedMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(RateLimitedMiddleware { 
            rate_limited: self.clone(),
            service: Rc::new(service),
         })
    }
}

type FutResult<T, E> = dyn Future<Output = Result<T, E>>;

impl<S> Service<ServiceRequest> for RateLimitedMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = actix_web::Error> + 'static,
    S::Future: 'static,
{
    type Response = S::Response;
    type Error = actix_web::Error;
    type Future = Pin<Box<FutResult<Self::Response, Self::Error>>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let ip_addr = get_ip(&req.connection_info());

        let rate_limited = self.rate_limited.clone();
        let service = self.service.clone();

        Box::pin(async move {
            if rate_limited.check(ip_addr) {
                service.call(req).await 
            } else {
                let (http_req, _) = req.into_parts();
                // if rate limit was hit respond error 400
                Ok(ServiceResponse::new(
                    http_req,
                    HttpResponse::BadRequest().finish(),
                ))
            }
        })
    }
}