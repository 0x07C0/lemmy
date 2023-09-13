use actix_web::{
  body::MessageBody,
  dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
  Error, HttpMessage,
};
use core::future::Ready;
use futures_util::future::LocalBoxFuture;
use lemmy_api_common::{
  context::LemmyContext, lemmy_db_views::structs::LocalUserView, utils::local_user_view_from_jwt,
};
use std::{future::ready, rc::Rc};

#[derive(Clone)]
pub struct SessionMiddleware {
  context: LemmyContext,
  auth_optional: bool,
}

impl SessionMiddleware {
  pub fn new(context: LemmyContext) -> Self {
    SessionMiddleware {
      context,
      auth_optional: false,
    }
  }

  pub fn opt_auth(&self) -> Self {
    let mut no_auth = self.clone();
    no_auth.auth_optional = true;
    no_auth
  }

  pub fn auth(&self) -> Self {
    let mut no_auth = self.clone();
    no_auth.auth_optional = false;
    no_auth
  }
}

impl<S, B> Transform<S, ServiceRequest> for SessionMiddleware
where
  S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
  S::Future: 'static,
  B: MessageBody + 'static,
{
  type Response = ServiceResponse<B>;
  type Error = Error;
  type Transform = SessionService<S>;
  type InitError = ();
  type Future = Ready<Result<Self::Transform, Self::InitError>>;

  fn new_transform(&self, service: S) -> Self::Future {
    ready(Ok(SessionService {
      service: Rc::new(service),
      context: self.context.clone(),
      auth_optional: self.auth_optional,
    }))
  }
}

pub struct SessionService<S> {
  service: Rc<S>,
  context: LemmyContext,
  auth_optional: bool,
}

impl<S, B> Service<ServiceRequest> for SessionService<S>
where
  S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
  S::Future: 'static,
  B: 'static,
{
  type Response = ServiceResponse<B>;
  type Error = Error;
  type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

  forward_ready!(service);

  fn call(&self, req: ServiceRequest) -> Self::Future {
    let svc = self.service.clone();
    let context = self.context.clone();
    let auth_optional = self.auth_optional;

    Box::pin(async move {
      // Try reading jwt from auth header
      if auth_optional {
        let local_user_view_opt = extract_local_user_view(&req, &context).await.ok();
        req.extensions_mut().insert(local_user_view_opt);
      } else {
        let local_user_view = extract_local_user_view(&req, &context).await?;
        req.extensions_mut().insert(local_user_view);
      }

      Ok(svc.call(req).await?)
    })
  }
}

async fn extract_local_user_view(req: &ServiceRequest, context: &LemmyContext) -> Result<LocalUserView, Error> {
  let auth_header = req
    .headers()
    .get(actix_web::http::header::AUTHORIZATION)
    .and_then(|h| h.to_str().ok());

  let auth_header = auth_header.ok_or(actix_web::error::ErrorForbidden("No Bearer token"))?;

  let jwt = auth_header
    .split(" ")
    .skip(1)
    .next()
    .ok_or(actix_web::error::ErrorForbidden("Invalid Bearer token"))?;

  let local_user_view = local_user_view_from_jwt(jwt, &context)
    .await
    .map_err(|_| actix_web::error::ErrorForbidden("Invalid JWT token"))?;
  
  Ok(local_user_view)
}
