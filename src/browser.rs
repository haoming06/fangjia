extern crate reqwest;
extern crate user_agent;

use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT, CONTENT_TYPE};
use std::collections::HashMap;
use self::reqwest::Response;
use serde::Serialize;
use self::reqwest::header::{COOKIE, SET_COOKIE};
use cookie_store::{CookieStore, Cookie};
use user_agent::SessionClient;
use self::user_agent::Session;
use std::io::{self, Write};
use std::fs;
use std::hash::Hash;

#[derive(Clone)]
pub struct Browser {
    client: reqwest::Client,
    headers: reqwest::header::HeaderMap,
}

impl Browser {
    pub fn new() -> Browser {
        Browser {
            client: reqwest::Client::new(),
            headers: reqwest::header::HeaderMap::new(),
        }.init()
    }
    fn init(mut self) -> Browser {
        self.headers.insert("User-Agent", HeaderValue::from_static("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_11_4) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/51.0.2704.103 Safari/537.36"));
        self.headers.insert("Content-Type", HeaderValue::from_static("application/json; charset=UTF-8"));
        self
    }

    pub fn get<T: Serialize + ?Sized>(&mut self, url: &str, param: &T) -> Response {
        let result = self.client.get(url).query(param).headers(self.headers.clone()).send().unwrap();
        result
    }

    pub fn post<T: Serialize + ?Sized>(&mut self, url: &'static str, param: &T) -> Response {
        let mut result = self.client.post(url).json(param).headers(self.headers.clone()).send().unwrap();
        if (&result.status().is_success() == &true) {
            self.headers.insert("Referer", HeaderValue::from_static(url));
        }
        result
    }
}