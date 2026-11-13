//! Reading a page of the web on a thread of its own: the browser page asks
//! for one and is told when it has come, or why it did not.
//!
//! The address is only ever http or https, and only what a page says in its
//! markup is kept: no picture is fetched, and nothing a script would draw is
//! run. What comes back is the page as [`skyward_net::html`] reads it.

use anyhow::{Context, Result};
use skyward_net::html::{self, Page};
use skyward_net::http;
use std::io::Read;
use std::sync::mpsc::{self, Receiver, TryRecvError};
use skyward_ui::Lang;

/// How much of a page is read: what a recovery can show, and no more than
/// it can hold.
const MOST: u64 = 512 * 1024;

/// A page being read, on its thread.
pub struct Fetcher {
    taken: Receiver<Result<Page, String>>,
}

impl Fetcher {
    /// Read `url` on a thread of its own, the names in it looked up in
    /// `servers`.
    pub fn start(url: String, servers: http::Servers, lang: Lang) -> Result<Fetcher> {
        let (tx, taken) = mpsc::channel();
        std::thread::Builder::new()
            .name("browser".into())
            .spawn(move || {
                let _ = tx.send(fetch(&url, servers, lang));
            })
            .context("start the thread reading the page")?;
        Ok(Fetcher { taken })
    }

    /// The page read since the last look, or why it was not.
    pub fn taken(&self) -> Option<Result<Page, String>> {
        match self.taken.try_recv() {
            Ok(page) => Some(page),
            Err(TryRecvError::Empty) | Err(TryRecvError::Disconnected) => None,
        }
    }
}

fn fetch(url: &str, servers: http::Servers, lang: Lang) -> Result<Page, String> {
    if !(url.starts_with("http://") || url.starts_with("https://")) {
        return Err(lang.pick(["An address starts with http:// or https://", "Адрес должен начинаться с http:// или https://"]).into());
    }
    let agent = http::agent(servers);
    let mut answer = agent.get(url).call().map_err(|e| format!("{e}"))?;
    let status = answer.status();
    if !status.is_success() {
        return Err(match lang {
            Lang::En => format!("The server answered {status}"),
            Lang::Ru => format!("Сервер ответил {status}"),
        });
    }
    let mut body = Vec::new();
    answer
        .body_mut()
        .with_config()
        .limit(MOST)
        .reader()
        .read_to_end(&mut body)
        .map_err(|e| match lang {
            Lang::En => format!("Couldn't read the page: {e}"),
            Lang::Ru => format!("Не удалось прочитать страницу: {e}"),
        })?;
    let markup = String::from_utf8_lossy(&body);
    Ok(html::parse(&markup, url))
}
