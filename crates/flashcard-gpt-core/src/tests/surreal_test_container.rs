use std::{borrow::Cow, collections::HashMap};

use testcontainers::{
    core::{ContainerPort, WaitFor},
    Image,
};

const NAME: &str = "surrealdb/surrealdb";
const TAG: &str = "v2.0.0-beta.1";

pub const SURREALDB_PORT: ContainerPort = ContainerPort::Tcp(8000);

#[derive(Debug, Clone)]
pub struct SurrealDbTestContainer {
    env_vars: HashMap<String, String>,
}

impl SurrealDbTestContainer {
    /// Sets the user for the SurrealDB instance.
    pub fn with_user(mut self, user: &str) -> Self {
        self.env_vars.insert("SURREAL_USER".to_owned(), user.to_owned());
        self
    }

    /// Sets the password for the SurrealDB instance.
    pub fn with_password(mut self, password: &str) -> Self {
        self.env_vars.insert("SURREAL_PASS".to_owned(), password.to_owned());
        self
    }

    /// Sets authentication for the SurrealDB instance.
    pub fn with_authentication(mut self, authentication: bool) -> Self {
        self.env_vars
            .insert("SURREAL_AUTH".to_owned(), authentication.to_string());
        self
    }

    /// Sets strict mode for the SurrealDB instance.
    pub fn with_strict(mut self, strict: bool) -> Self {
        self.env_vars.insert("SURREAL_STRICT".to_owned(), strict.to_string());
        self
    }

    /// Sets all capabilities for the SurrealDB instance.
    pub fn with_all_capabilities(mut self, allow_all: bool) -> Self {
        self.env_vars
            .insert("SURREAL_CAPS_ALLOW_ALL".to_owned(), allow_all.to_string());
        self
    }
}

impl Default for SurrealDbTestContainer {
    fn default() -> Self {
        let mut env_vars = HashMap::new();
        env_vars.insert("SURREAL_USER".to_owned(), "root".to_owned());
        env_vars.insert("SURREAL_PASS".to_owned(), "root".to_owned());
        env_vars.insert("SURREAL_AUTH".to_owned(), "true".to_owned());
        env_vars.insert("SURREAL_CAPS_ALLOW_ALL".to_owned(), "true".to_owned());
        env_vars.insert("SURREAL_PATH".to_owned(), "memory".to_owned());

        Self { env_vars }
    }
}

impl Image for SurrealDbTestContainer {
    fn name(&self) -> &str {
        NAME
    }

    fn tag(&self) -> &str {
        TAG
    }

    fn ready_conditions(&self) -> Vec<WaitFor> {
        vec![WaitFor::message_on_stderr("Started web server on ")]
    }

    fn env_vars(&self) -> impl IntoIterator<Item = (impl Into<Cow<'_, str>>, impl Into<Cow<'_, str>>)> {
        &self.env_vars
    }

    fn cmd(&self) -> impl IntoIterator<Item = impl Into<Cow<'_, str>>> {
        ["start"]
    }

    fn expose_ports(&self) -> &[ContainerPort] {
        &[SURREALDB_PORT]
    }
}
