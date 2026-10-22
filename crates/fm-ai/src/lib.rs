use fm_core::{Config, Result, error};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::time::Duration;

#[derive(Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}
pub struct Client {
    http: reqwest::Client,
    base: String,
    key: String,
    config: Config,
}
pub fn is_local(base: &str) -> bool {
    url::Url::parse(base)
        .ok()
        .and_then(|u| u.host_str().map(str::to_string))
        .is_some_and(|h| matches!(h.as_str(), "127.0.0.1" | "[::1]" | "::1" | "localhost"))
}
pub fn validate_url(base: &str) -> Result<()> {
    let u = url::Url::parse(base).map_err(|_| error("Ungültige Basis-URL"))?;
    if !u.username().is_empty()
        || u.password().is_some()
        || u.query().is_some()
        || u.fragment().is_some()
    {
        return Err(error(
            "Basis-URL darf keine Zugangsdaten, Parameter oder Fragmente enthalten",
        ));
    }
    if u.scheme() != "https" && !(u.scheme() == "http" && is_local(base)) {
        return Err(error(
            "Externe KI-Anbieter benötigen eine HTTPS-URL. HTTP ist nur lokal erlaubt.",
        ));
    }
    Ok(())
}
impl Client {
    pub async fn complete_json<T: serde::de::DeserializeOwned>(
        &self,
        messages: Vec<Message>,
    ) -> Result<T> {
        let text = self.complete(messages).await?.concat();
        let mut value: Value = parse_json(&text)?;
        self.redact_json(&mut value);
        serde_json::from_value(value).map_err(|_| {
            error("Der strukturierte KI-Vorschlag entspricht nicht dem erwarteten Format")
        })
    }
    fn redact_json(&self, value: &mut Value) {
        match value {
            Value::String(s) => *s = self.redact(s),
            Value::Array(values) => {
                for value in values {
                    self.redact_json(value);
                }
            }
            Value::Object(values) => {
                let previous = std::mem::take(values);
                for (key, mut value) in previous {
                    self.redact_json(&mut value);
                    values.insert(self.redact(&key), value);
                }
            }
            _ => {}
        }
    }
    pub fn new(config: Config, key: String) -> Result<Self> {
        Self::with_timeout(config, key, Duration::from_secs(90))
    }
    pub fn with_timeout(config: Config, key: String, timeout: Duration) -> Result<Self> {
        validate_url(&config.base_url)?;
        if !is_local(&config.base_url) && !config.cloud_consent {
            return Err(error(
                "Bitte zuerst bestätigen, dass Textausschnitte an den Cloud-Anbieter gesendet werden dürfen",
            ));
        }
        let http = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(10))
            .timeout(timeout)
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|_| error("HTTP-Client konnte nicht gestartet werden"))?;
        Ok(Self {
            base: config.base_url.trim_end_matches('/').into(),
            http,
            key,
            config,
        })
    }
    fn request(&self, method: reqwest::Method, path: &str) -> reqwest::RequestBuilder {
        let r = self.http.request(method, format!("{}{path}", self.base));
        if self.key.is_empty() {
            r
        } else {
            r.bearer_auth(&self.key)
        }
    }
    fn transport(e: reqwest::Error) -> fm_core::Error {
        if e.is_timeout() {
            error("Zeitüberschreitung: Der KI-Anbieter antwortet nicht rechtzeitig")
        } else {
            error("Verbindung fehlgeschlagen. Basis-URL und Erreichbarkeit prüfen.")
        }
    }
    async fn checked(
        response: std::result::Result<reqwest::Response, reqwest::Error>,
    ) -> Result<reqwest::Response> {
        let r = response.map_err(Self::transport)?;
        match r.status().as_u16() {
            200..=299 => Ok(r),
            401 | 403 => Err(error("API-Key ungültig oder keine Berechtigung")),
            404 => Err(error("Modell oder API-Endpunkt nicht gefunden")),
            429 => Err(error(
                "Anfragelimit oder Guthaben des KI-Anbieters erreicht",
            )),
            _ => Err(error("Der KI-Anbieter meldet einen Serverfehler")),
        }
    }
    fn redact(&self, text: &str) -> String {
        if self.key.is_empty() {
            text.to_string()
        } else {
            text.replace(&self.key, "[API-Key entfernt]")
        }
    }
    pub async fn models(&self) -> Result<Vec<String>> {
        let v: Value = Self::checked(self.request(reqwest::Method::GET, "/models").send().await)
            .await?
            .json()
            .await
            .map_err(|_| error("Ungültige Modellliste vom KI-Anbieter"))?;
        let a = v["data"]
            .as_array()
            .ok_or_else(|| error("Ungültige Modellliste vom KI-Anbieter"))?;
        Ok(a.iter()
            .filter_map(|m| m["id"].as_str())
            .map(|m| self.redact(m))
            .collect())
    }
    pub async fn test(&self) -> Result<Vec<String>> {
        let models = self.models().await?;
        if !self.config.chat_model.is_empty()
            && !models.iter().any(|m| m == &self.config.chat_model)
        {
            return Err(error(
                "Chat-Modell nicht in der Modellliste gefunden. Modellnamen prüfen.",
            ));
        }
        if !self.config.chat_model.is_empty() {
            self.complete(vec![Message {
                role: "user".into(),
                content: "Antworte nur mit OK.".into(),
            }])
            .await?;
        }
        Ok(models)
    }
    pub async fn complete(&self, messages: Vec<Message>) -> Result<Vec<String>> {
        if self.config.chat_model.trim().is_empty() {
            return Err(error(
                "Bitte zuerst ein Chat-Modell in den Einstellungen wählen",
            ));
        }
        let r=Self::checked(self.request(reqwest::Method::POST,"/chat/completions").json(&json!({"model":self.config.chat_model,"messages":messages,"temperature":self.config.temperature,"max_tokens":self.config.max_tokens,"stream":true})).send().await).await?;
        let mut stream = r.bytes_stream();
        let mut buffer = Vec::new();
        let mut tokens = Vec::new();
        let mut size = 0usize;
        let mut done = false;
        while let Some(bytes) = stream.next().await {
            let bytes = bytes.map_err(Self::transport)?;
            size += bytes.len();
            if size > 8_000_000 {
                return Err(error("KI-Antwort überschreitet die erlaubte Grösse"));
            }
            buffer.extend_from_slice(&bytes);
            while let Some(pos) = buffer.iter().position(|b| *b == b'\n') {
                let line = buffer.drain(..=pos).collect::<Vec<_>>();
                let line = std::str::from_utf8(&line)
                    .map_err(|_| error("Ungültige Zeichen in der KI-Antwort"))?
                    .trim();
                if let Some(data) = line.strip_prefix("data:") {
                    let data = data.trim();
                    if data == "[DONE]" {
                        done = true;
                        break;
                    }
                    let v: Value = serde_json::from_str(data)
                        .map_err(|_| error("Fehlerhafte SSE-Antwort vom KI-Anbieter"))?;
                    if v.get("error").is_some() {
                        return Err(error("KI-Anbieter hat die Antwort abgebrochen"));
                    }
                    if let Some(t) = v["choices"][0]["delta"]["content"].as_str() {
                        tokens.push(t.to_string());
                    }
                }
            }
            if done {
                break;
            }
        }
        if !done || tokens.is_empty() {
            return Err(error("Unvollständige Streaming-Antwort vom KI-Anbieter"));
        }
        // Erst nach dem Zusammenfügen redigieren: Schlüssel können über SSE-Blöcke verteilt sein.
        let answer = self.redact(&tokens.concat());
        Ok(answer
            .chars()
            .collect::<Vec<_>>()
            .chunks(64)
            .map(|c| c.iter().collect())
            .collect())
    }
    pub async fn embeddings(&self, input: Vec<String>) -> Result<Vec<Vec<f32>>> {
        if self.config.embedding_model.is_empty() {
            return Err(error("Kein Embedding-Modell eingestellt"));
        }
        let count = input.len();
        let v: Value = Self::checked(
            self.request(reqwest::Method::POST, "/embeddings")
                .json(&json!({"model":self.config.embedding_model,"input":input}))
                .send()
                .await,
        )
        .await?
        .json()
        .await
        .map_err(|_| error("Ungültige Embedding-Antwort"))?;
        let a = v["data"]
            .as_array()
            .ok_or_else(|| error("Ungültige Embedding-Antwort"))?;
        let mut out = vec![Vec::new(); count];
        for item in a {
            let i = item["index"]
                .as_u64()
                .ok_or_else(|| error("Embedding-Index fehlt"))? as usize;
            if i >= count {
                return Err(error("Ungültiger Embedding-Index"));
            }
            let vector: Vec<f32> = serde_json::from_value(item["embedding"].clone())
                .map_err(|_| error("Ungültiger Embedding-Vektor"))?;
            if vector.is_empty() || vector.iter().any(|f| !f.is_finite()) || !out[i].is_empty() {
                return Err(error("Ungültiger Embedding-Vektor"));
            }
            out[i] = vector;
        }
        if out.iter().any(Vec::is_empty) || out.iter().any(|v| v.len() != out[0].len()) {
            return Err(error("Unvollständige Embedding-Antwort"));
        }
        Ok(out)
    }
}
pub fn parse_json<T: serde::de::DeserializeOwned>(text: &str) -> Result<T> {
    let text = text.trim();
    let text = if text.starts_with("```") && text.ends_with("```") {
        text.split_once('\n')
            .map(|(_, rest)| rest[..rest.len().saturating_sub(3)].trim())
            .unwrap_or(text)
    } else {
        text
    };
    serde_json::from_str(text).map_err(|_| error("Das KI-Modell hat keinen gültigen strukturierten Vorschlag geliefert. Bitte erneut versuchen oder das Ziel selbst wählen."))
}
pub fn store_key(reference: &str, key: &str) -> Result<()> {
    #[cfg(windows)]
    {
        let e = keyring::Entry::new("folder-mind", reference)
            .map_err(|_| error("Windows Credential Manager nicht verfügbar"))?;
        if key.is_empty() {
            match e.delete_credential() {
                Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
                Err(_) => Err(error("API-Key konnte nicht entfernt werden")),
            }
        } else {
            e.set_password(key).map_err(|_| {
                error("API-Key konnte nicht im Windows Credential Manager gespeichert werden")
            })
        }
    }
    #[cfg(not(windows))]
    {
        let _ = (reference, key);
        Err(error(
            "Dauerhafte API-Key-Ablage ist in diesem Build nur unter Windows verfügbar",
        ))
    }
}
pub fn load_key(reference: &str) -> Result<String> {
    #[cfg(windows)]
    {
        match keyring::Entry::new("folder-mind", reference)
            .map_err(|_| error("Windows Credential Manager nicht verfügbar"))?
            .get_password()
        {
            Ok(k) => Ok(k),
            Err(keyring::Error::NoEntry) => Ok(String::new()),
            Err(_) => Err(error(
                "API-Key konnte nicht aus dem Windows Credential Manager gelesen werden",
            )),
        }
    }
    #[cfg(not(windows))]
    {
        let _ = reference;
        Ok(String::new())
    }
}
