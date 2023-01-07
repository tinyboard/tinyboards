use actix_web::dev::ConnectionInfo;
use once_cell::sync::Lazy;
use regex::Regex;
use crate::{IpAddr, settings::structs::Settings};

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

static MENTIONS_REGEX: Lazy<Regex> = Lazy::new(|| {
  Regex::new(r"@(?P<name>[\w.]+)").expect("compile regex")
});

static IMG_TAG_REGEX: Lazy<Regex> = Lazy::new(|| {
  Regex::new(r"<img src=").expect("compile img tag regex")
});

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct MentionData {
  pub name: String,
}

impl MentionData {
  // pub fn is_local(&self, hostname: &str) -> bool {
  //   hostname.eq(&self.domain)
  // }
  pub fn full_name(&self) -> String {
    format!("@{}", &self.name)
  }
}

pub fn scrape_text_for_mentions(text: &str) -> Vec<MentionData> {
  let mut out: Vec<MentionData> = Vec::new();
  for caps in MENTIONS_REGEX.captures_iter(text) {
    out.push(MentionData { 
      name: caps["name"].to_string(),
    });
  }
  out.sort_by_key(|k| k.name.clone());
  out.dedup();
  out
}

pub fn custom_body_parsing(body: &str, settings: &Settings) -> String {

  let base_url = settings.get_protocol_and_hostname();
  let mut result = IMG_TAG_REGEX.replace_all(body, "<img loading='lazy' src=").to_string();
  let rcopy = result.clone();

  for cap in MENTIONS_REGEX
    .captures_iter(rcopy.as_str()) {
      let uname = cap["name"].to_string();
      let profile_link = format!("{}/user/{}", base_url, uname);
      let profile_ref = format!("<a href='{}'>@{}</a>", profile_link, uname);
      //let nuxt_ref = format!("<NuxtLink to='/user/{}'>@{}</NuxtLink>", uname, uname);
      result = result.replace(&format!("@{}", uname), &profile_ref);
    }
  
  result
}