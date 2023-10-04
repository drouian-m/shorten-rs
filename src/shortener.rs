use std::sync::{Arc, Mutex};

use nanoid::nanoid;

#[derive(Debug, Clone)]
pub struct Url {
    pub short_id: String,
    pub url: String,
    pub target_url: String,
    pub visits: i32,
}
#[derive(Debug, Clone)]
pub struct Shortener {
    domain: String,
    urls: Arc<Mutex<Vec<Url>>>,
}

impl Shortener {
    pub fn new(domain: String) -> Self {
        Shortener {
            domain,
            urls: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn store(&self, url: String) -> Result<Url, &'static str> {
        let mut urls = self.urls.lock().map_err(|_| "Failed to acquire lock")?;

        let id = nanoid!(10);
        let u = Url {
            short_id: id.clone(),
            url: url.clone(),
            target_url: format!("{}/{}", self.domain, id),
            visits: 0,
        };
        urls.push(u.clone());
        println!("{:?}", urls);
        Ok(u)
    }

    pub fn read(&self, short_id: String) -> Result<Url, &'static str> {
        let mut urls = self.urls.lock().map_err(|_| "Failed to acquire lock")?;
        println!("{:?}", urls);
        if let Some(elem) = urls.iter_mut().find(|elem| elem.short_id == short_id) {
            elem.visits += 1;
            Ok(elem.clone())
        } else {
            Err("Url not found !")
        }
    }
}
