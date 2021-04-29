#![feature(async_closure)]

use std::future::Future;

use reqwest::{Client, Error, Response};
use serde::Serialize;
use serde_json::{Value};

use tokio::{task::JoinHandle, time::{sleep, Duration}};


const API_VERSION: &str = "5.130";

pub struct ApiManager {
    token: String,
    version: String,
    client: Client,
}

impl ApiManager {
    const API_SERVER: &'static str = "https://api.vk.com/method";
    
    pub fn new<T1, T2>(token: T1, version: T2) -> Self
    where
        T1: Into<String>,
        T2: Into<String>,
    {
        Self {
            token: token.into(),
            version: version.into(),
            client: reqwest::Client::new(),
        }
    }

    pub fn request<T: Serialize + ?Sized>(&self, method: &str, params: &T) -> impl Future<Output = Result<Response, Error>> {
        let request = self.client.get(format!("{}/{}", ApiManager::API_SERVER, method));
        
        let request = request.query(params);
        let request = request.query(
            &[("access_token", &self.token), ("v", &self.version)]
        );

        request.send()
    }

}

#[derive(Debug, Clone)]
pub struct Config {
    tokens: Vec<String>,
    messages: Vec<String>,
    invite_link: String,
}

impl Config {
    pub fn new(tokens: Vec<String>, messages: Vec<String>, invite_link: String) -> Self {
        Config {
            tokens: tokens,
            messages: messages,
            invite_link: invite_link,
        }
    }
}

pub struct RaidSystem {
    handlers: Vec<ApiManager>,
    config: Config,
}

impl RaidSystem {
    pub fn new<T>(config: Config, version: T) -> Self
    where
        T: Into<String> + Clone,
    {
        let mut handlers: Vec<ApiManager> = Vec::new();

        for token in config.tokens.iter().cloned() {
            handlers.push(
                ApiManager::new(
                    token,
                    version.clone(),
                )
            )    
        }

        Self {
            handlers,
            config
        }
    }

    async fn work(handler: ApiManager, invite_link: String, messages: Vec<String>) {
        let user_id = handler.request(
            "account.getProfileInfo", &[()]
        ).await.unwrap().json::<Value>().await.unwrap();
        let user_id = user_id["response"]["id"].as_str().unwrap();

        let resp = handler.request(
            "messages.joinChatByInviteLink", &[("link", &invite_link)]
        ).await.unwrap().json::<Value>().await.unwrap();
        let resp = resp.as_object().unwrap();

        let chat_id: String;
        if resp.contains_key("response") {
            chat_id = resp["response"].as_str().unwrap().into();
        } else {
            // Then it contains key "error"
            let resp = handler.request(
                "messages.getChatPreview", &[("link", &invite_link)]
            ).await.unwrap().json::<Value>().await.unwrap();
            chat_id = resp["preview"]["local_id"].as_str().unwrap().into();
        }

        for message in messages.iter().cycle() {
            let random_id = format!("{}", rand::random::<i32>());

            let resp = handler.request(
                "messages.send",
                &[("chat_id", &chat_id), ("message", &message), ("random_id", &random_id)]
            ).await.unwrap().json::<Value>().await.unwrap();
            let resp = resp.as_object().unwrap();

            if resp.contains_key("error") {
                sleep(Duration::from_millis(500)).await;
            } else {
                sleep(Duration::from_millis(200)).await;
            }
        }

    }

    pub fn run(self) -> Vec<JoinHandle<()>> {
        let mut handles = Vec::new();

        for handler in self.handlers {    
            let invite_link = self.config.invite_link.clone();
            let messages = self.config.messages.clone();
    
            handles.push(tokio::spawn(async move {
                Self::work(handler, invite_link, messages).await
            }));    
        }

        handles
    }
}
