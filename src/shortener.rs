use std::sync::{Arc, Mutex};
use url::Url;

use nanoid::nanoid;

#[derive(Debug, Clone)]
pub struct ShortenUrl {
    pub short_id: String,
    pub url: String,
    pub target_url: String,
    pub visits: i32,
}

#[derive(Debug, Clone)]
pub struct Shortener {
    domain: String,
    urls: Arc<Mutex<Vec<ShortenUrl>>>,
}

impl Shortener {
    pub fn new(domain: String) -> Self {
        Shortener {
            domain,
            urls: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn store(&self, url: String) -> Result<ShortenUrl, String> {
        self.is_valid_url(&url)?;

        let mut urls = self.urls.lock().map_err(|_| "Failed to acquire lock")?;
        let id = nanoid!(10);
        let u = ShortenUrl {
            short_id: id.clone(),
            url: url.clone(),
            target_url: format!("{}/{}", self.domain, id),
            visits: 0,
        };
        urls.push(u.clone());
        Ok(u)
    }

    fn is_valid_url(&self, url_str: &str) -> Result<(), String> {
        match Url::parse(url_str) {
            Ok(_) => Ok(()),
            Err(_) => Err("Invalid URL".to_owned()),
        }
    }

    pub fn read(&self, short_id: String) -> Result<ShortenUrl, String> {
        let mut urls = self.urls.lock().map_err(|_| "Failed to acquire lock")?;
        if let Some(elem) = urls.iter_mut().find(|elem| elem.short_id == short_id) {
            elem.visits += 1;
            Ok(elem.clone())
        } else {
            Err("Url not found !".to_owned())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Shortener;

    #[test]
    fn store_invalid_url() {
        let shortener = Shortener::new("http://localhost".to_owned());
        let result = shortener.store("wrong/url".to_string());
        match result {
            Ok(_) => assert!(false, "Expected an error, but got Ok"),
            Err(err) => assert_eq!(err, "Invalid URL"),
        }
    }

    #[test]
    fn store_new_url() {
        let shortener = Shortener::new("http://localhost".to_owned());
        let result = shortener.store("https://doc.rust-lang.org/book".to_string());

        match result {
            Ok(result) => {
                assert_eq!(result.url, "https://doc.rust-lang.org/book".to_string());
                assert_eq!(result.short_id.len(), 10);
                assert_eq!(
                    result.target_url,
                    format!("{}/{}", "http://localhost".to_owned(), result.short_id)
                );
                assert_eq!(result.visits, 0);
            }
            Err(_) => assert!(false, "Expected a result, but got an error"),
        }
    }

    #[test]
    fn read_url() -> Result<(), String> {
        let shortener = Shortener::new("http://localhost".to_owned());
        let result = shortener.store("https://doc.rust-lang.org/book".to_string())?;

        let retrieved = shortener.read(result.short_id.clone());
        match retrieved {
            Ok(retrieved) => {
                assert_eq!(retrieved.url, result.url);
                assert_eq!(retrieved.short_id, result.short_id);
                assert_eq!(retrieved.target_url, result.target_url);
                assert_eq!(retrieved.visits, 1);
            }
            Err(_) => assert!(false, "Expected a result, but got an error"),
        }

        Ok(())
    }
}
