use actix_web::dev::ConnectionInfo;
use once_cell::sync::Lazy;
use regex::{Regex};
use crate::{IpAddr, settings::structs::Settings};
use rand::{distributions::Alphanumeric, thread_rng, Rng};

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

static YT_REGEX: Lazy<Regex> = Lazy::new(|| {
  Regex::new(
    r#"(?P<a>https?://|http://)(?P<b>www\.)?(?P<c>youtube\.com/watch\?v=|youtube\.com/user/[a-zA-Z0-9_]+#p/a/u/[a-zA-Z0-9_]+/|youtube\.com/v/|youtube\.com/watch\?v=|youtube\.com/embed/|youtu\.be/|youtube\.com/shorts/)(?P<yt_code>[a-zA-Z0-9_]+)(?P<end>[^\s]*)"#)
    .expect("compile yt link regex")
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
  let mut result = IMG_TAG_REGEX.replace_all(body, "<img loading='lazy' class='img-expand' src=").to_string();
  let rcopy = result.clone();

  for cap in MENTIONS_REGEX
    .captures_iter(rcopy.as_str()) {
      let uname = cap["name"].to_string();
      let profile_link = format!("{}/user/{}", base_url, uname);
      let profile_ref = format!("<a href='{}'>@{}</a>", profile_link, uname);
      //let nuxt_ref = format!("<NuxtLink to='/user/{}'>@{}</NuxtLink>", uname, uname);
      result = result.replace(&format!("@{}", uname), &profile_ref);
    }
  
  let rcopy = result.clone();
  
  for cap in YT_REGEX.captures_iter(rcopy.as_str()) {
    let yt_code = cap["yt_code"].to_string();
    let yt_tag =  format!("<lite-youtube videoid='{}'></lite-youtube>", yt_code);
    
    let mut yt_vec: Vec<&str> = Vec::new();

    if let Some(a) = cap.name("a") {
      yt_vec.push(a.as_str());
    }

    if let Some(b) = cap.name("b") {
      yt_vec.push(b.as_str());
    }

    if let Some(c) = cap.name("c") {
      yt_vec.push(c.as_str());
    }

    if let Some(d) = cap.name("yt_code") {
      yt_vec.push(d.as_str());
    }

    if let Some(e) = cap.name("e") {
      yt_vec.push(e.as_str());
    }

    let yt_link = yt_vec.concat();

    result = result.replace(&yt_link, &yt_tag);
  }
  result
}

pub fn generate_rand_string() -> String {
  thread_rng()
    .sample_iter(&Alphanumeric)
    .map(char::from)
    .take(18)
    .collect()
}