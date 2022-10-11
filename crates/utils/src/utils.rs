use actix_web::dev::ConnectionInfo;
use crate::IpAddr;

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