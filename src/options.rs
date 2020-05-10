use std::collections::HashMap;
use std::collections::hash_map::RandomState;

pub struct RouterOptions {
    request_headers_size_limit: Option<usize>,
    request_body_size_limit: Option<usize>,
    response_headers_size_limit: Option<usize>,
    response_body_size_limit: Option<usize>,
    targets: HashMap<String, String>,
}

impl RouterOptions {

    pub fn default() -> Self {
        Self {
            request_headers_size_limit: None,
            request_body_size_limit: None,
            response_headers_size_limit: None,
            response_body_size_limit: None,
            targets: HashMap::with_hasher(RandomState::new()),
        }
    }

    pub fn request_headers_size_limit(&self) -> &Option<usize> {
        &self.request_headers_size_limit
    }

    pub fn request_body_size_limit(&self) -> &Option<usize> {
        &self.request_body_size_limit
    }

    pub fn response_headers_size_limit(&self) -> &Option<usize> {
        &self.response_headers_size_limit
    }

    pub fn response_body_size_limit(&self) -> &Option<usize> {
        &self.response_body_size_limit
    }

    pub fn target<S>(&self, source: S) -> Option<&String>
        where
        S: Into<String>,
    {
        self.targets.get(&source.into())
    }

    pub fn has_target<S>(&self, source: S) -> bool
        where
        S: Into<String>,
    {
        self.targets.contains_key(&source.into())
    }

    pub fn set_request_headers_size_limit(&mut self, limit: usize) {
        self.request_headers_size_limit = Some(limit);
    }

    pub fn set_request_body_size_limit(&mut self, limit: usize) {
        self.request_body_size_limit = Some(limit);
    }

    pub fn set_response_headers_size_limit(&mut self, limit: usize) {
        self.response_headers_size_limit = Some(limit);
    }

    pub fn set_response_body_size_limit(&mut self, limit: usize) {
        self.response_body_size_limit = Some(limit);
    }

    pub fn set_target<S, T>(&mut self, source: S, target: T)
        where
        S: Into<String>,
        T: Into<String>
    {
        self.targets.insert(source.into(), target.into());
    }

    pub fn remove_request_headers_size_limit(&mut self) {
        self.request_headers_size_limit = None;
    }

    pub fn remove_request_body_size_limit(&mut self) {
        self.request_body_size_limit = None;
    }

    pub fn remove_response_headers_size_limit(&mut self) {
        self.response_headers_size_limit = None;
    }

    pub fn remove_response_body_size_limit(&mut self) {
        self.response_body_size_limit = None;
    }

    pub fn remove_target<S, T>(&mut self, target: T)
        where
        T: Into<String>
    {
        self.targets.remove(&target.into());
    }
}
