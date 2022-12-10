use actix_web::dev::ConnectionInfo;
use crate::IpAddr;
use crate::error::TinyBoardsError;
use regex::Regex;

pub fn get_ip(conn_info: &ConnectionInfo) -> IpAddr {
    IpAddr(
      conn_info
        .realip_remote_addr()
        .unwrap_or("127.0.0.1:12345")
        .split(':')
        .next()
        .unwrap_or("127.0.0.1")
        .to_string(),
    )
}

pub fn is_valid_email(email: &str) -> Result<(), TinyBoardsError> {

  let email_regex = Regex::new(r"^([a-z0-9_+]([a-z0-9_+.]*[a-z0-9_+])?)@([a-z0-9]+([\-\.]{1}[a-z0-9]+)*\.[a-z]{2,6})").unwrap();  

  if email_regex.is_match(email) {
    Ok(())
  } else {
    Err(TinyBoardsError::from_message("invalid email address"))
  }
} 