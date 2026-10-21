use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Component, Path, PathBuf},
    sync::{Arc, Mutex},
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Dateizugriff fehlgeschlagen: {0}")]
    Io(#[from] std::io::Error),
    #[error("Pfad ausserhalb des Datenordners oder unsicherer Pfad")]
    UnsafePath,
    #[error("{0}")]
    Message(String),
}
pub type Result<T> = std::result::Result<T, Error>;
pub fn error(e: impl std::fmt::Display) -> Error {
    Error::Message(e.to_string())
}
pub fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
pub fn hash(bytes: &[u8]) -> String {
    use sha2::Digest;
    format!("{:x}", sha2::Sha256::digest(bytes))
}

#[derive(Clone, Debug)]
pub struct Guard {
    root: PathBuf,
    lock: Arc<Mutex<()>>,
}
impl Guard {
    pub fn new(root: impl AsRef<Path>) -> Result<Self> {
        let root = dunce::canonicalize(root)?;
        if !root.is_dir() {
            return Err(Error::UnsafePath);
        }
        Ok(Self {
            root,
            lock: Arc::new(Mutex::new(())),
        })
    }
    pub fn root(&self) -> &Path {
        &self.root
    }
    pub fn resolve(&self, relative: impl AsRef<Path>) -> Result<PathBuf> {
        let result = self.checked(relative.as_ref());
        if result.is_err() {
            tracing::warn!("Pfadwächter hat einen unsicheren Zugriff blockiert");
        }
        result
    }
    fn checked(&self, relative: &Path) -> Result<PathBuf> {
        if relative.as_os_str().is_empty() {
            return Ok(self.root.clone());
        }
        let mut path = self.root.clone();
        for part in relative.components() {
            let Component::Normal(name) = part else {
                return Err(Error::UnsafePath);
            };
            let name = name.to_string_lossy();
            if name.contains(['\\', ':', '\0']) || name.ends_with(['.', ' ']) || name.is_empty() {
                return Err(Error::UnsafePath);
            }
            let stem = name.split('.').next().unwrap_or("").to_uppercase();
            if [
                "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7",
                "COM8", "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8",
                "LPT9",
            ]
            .contains(&stem.as_str())
            {
                return Err(Error::UnsafePath);
            }
            path.push(name.as_ref());
            if let Ok(meta) = fs::symlink_metadata(&path) {
                if meta.file_type().is_symlink() {
                    return Err(Error::UnsafePath);
                }
                #[cfg(windows)]
                {
                    use std::os::windows::fs::MetadataExt;
                    if meta.file_attributes() & 0x400 != 0 {
                        return Err(Error::UnsafePath);
                    }
                }
                #[cfg(unix)]
                {
                    use std::os::unix::fs::MetadataExt;
                    if meta.is_file() && meta.nlink() > 1 {
                        return Err(Error::UnsafePath);
                    }
                }
                if !dunce::canonicalize(&path)?.starts_with(&self.root) {
                    return Err(Error::UnsafePath);
                }
            }
        }
        Ok(path)
    }
    pub fn mkdir(&self, path: impl AsRef<Path>) -> Result<()> {
        let _lock = self.lock.lock().map_err(error)?;
        fs::create_dir_all(self.resolve(path)?)?;
        Ok(())
    }
    pub fn write(&self, path: impl AsRef<Path>, bytes: &[u8]) -> Result<()> {
        let _lock = self.lock.lock().map_err(error)?;
        let path = self.resolve(path)?;
        // Eine neue Datei verhindert das Überschreiben fremder Hardlink-Ziele.
        let temp = path.with_file_name(format!(".fm-{}.tmp", uuid::Uuid::new_v4()));
        let relative = temp.strip_prefix(&self.root).map_err(error)?;
        let temp = self.resolve(relative)?;
        use std::io::Write;
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        drop(file);
        if path.exists() {
            fs::remove_file(self.resolve(path.strip_prefix(&self.root).map_err(error)?)?)?;
        }
        fs::rename(temp, path)?;
        Ok(())
    }
    pub fn upload(&self, path: impl AsRef<Path>, bytes: &[u8]) -> Result<()> {
        let _lock = self.lock.lock().map_err(error)?;
        use std::io::Write;
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(self.resolve(path)?)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        Ok(())
    }
    pub fn move_path(&self, from: impl AsRef<Path>, to: impl AsRef<Path>) -> Result<()> {
        let _lock = self.lock.lock().map_err(error)?;
        let from = self.resolve(from)?;
        let to = self.resolve(to)?;
        if to.exists() || from == self.root {
            return Err(error("Ziel existiert bereits oder Datenwurzel geschützt"));
        }
        fs::rename(from, to)?;
        Ok(())
    }
    pub fn trash(&self, relative: impl AsRef<Path>, confirmed: bool) -> Result<()> {
        if !confirmed {
            return Err(error("Bitte Löschen bestätigen"));
        }
        let relative = relative.as_ref();
        let path = self.resolve(relative)?;
        if path.is_dir() && fs::read_dir(&path)?.next().is_some() {
            return Err(error("Nur leere Ordner können gelöscht werden"));
        }
        self.mkdir(".papierkorb")?;
        self.move_path(
            relative,
            PathBuf::from(".papierkorb").join(uuid::Uuid::new_v4().to_string()),
        )
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub name: String,
    pub template: String,
    pub levels: Vec<String>,
    pub naming_rules: bool,
    pub base_url: String,
    pub chat_model: String,
    pub embedding_model: String,
    pub api_key_ref: String,
    pub cloud_consent: bool,
    pub temperature: f32,
    pub max_tokens: u32,
    pub allow_free_chat: bool,
    pub auto_approve: bool,
    pub stable_seconds: u64,
    pub max_file_mb: u64,
    pub network_enabled: bool,
    pub network_bind: String,
    pub allowed_ips: Vec<String>,
    pub allowed_macs: Vec<String>,
    pub password_hash: String,
    pub autostart: bool,
    pub work_start_hour: u8,
    pub work_end_hour: u8,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            name: String::new(),
            template: "lehre-informatik".into(),
            levels: vec!["Lehrjahr".into(), "Modul".into(), "Kapitel".into()],
            naming_rules: true,
            base_url: "http://127.0.0.1:11434/v1".into(),
            chat_model: String::new(),
            embedding_model: String::new(),
            api_key_ref: "keyring:folder-mind".into(),
            cloud_consent: false,
            temperature: 0.2,
            max_tokens: 1500,
            allow_free_chat: true,
            auto_approve: false,
            stable_seconds: 30,
            max_file_mb: 100,
            network_enabled: false,
            network_bind: "0.0.0.0:8485".into(),
            allowed_ips: vec![],
            allowed_macs: vec![],
            password_hash: String::new(),
            autostart: false,
            work_start_hour: 0,
            work_end_hour: 24,
        }
    }
}
impl Config {
    pub fn save(&self, guard: &Guard) -> Result<()> {
        guard.write(
            ".folder-mind/config.toml",
            toml::to_string_pretty(self).map_err(error)?.as_bytes(),
        )
    }
    pub fn load(guard: &Guard) -> Result<Self> {
        toml::from_str(&fs::read_to_string(
            guard.resolve(".folder-mind/config.toml")?,
        )?)
        .map_err(error)
    }
}
pub fn initialize(guard: &Guard, config: &Config) -> Result<()> {
    for dir in [
        "unterlagen",
        "extrakte",
        "wissen",
        "chats",
        "protokolle",
        "backups",
        ".folder-mind/index",
    ] {
        guard.mkdir(dir)?;
    }
    let example = match config.template.as_str() {
        "schule" => "mathematik/algebra",
        "lehre-allgemein" => "1-lehrjahr/fach/kapitel-01",
        "frei" => "meine-unterlagen",
        _ => "1-lehrjahr/modul-117-netzwerk/kapitel-01-grundlagen",
    };
    guard.mkdir(PathBuf::from("unterlagen").join(example))?;
    config.save(guard)
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Chunk {
    pub id: String,
    pub path: String,
    pub hash: String,
    pub location: String,
    pub text: String,
    pub extracted_at: u64,
    pub extractor: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Document {
    pub path: String,
    pub hash: String,
    pub size: u64,
    pub status: String,
    pub error: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn path_attacks_and_safe_writes() {
        let t = tempfile::tempdir().unwrap();
        let g = Guard::new(t.path()).unwrap();
        for p in [
            "../outside",
            "/etc/passwd",
            "C:\\Windows",
            "a/../../b",
            "a\\..\\b",
            "x:stream",
            "NUL.txt",
            "COM1",
            "trailing.",
            "trailing ",
        ] {
            assert!(g.resolve(p).is_err(), "{p}");
        }
        g.mkdir("unterlagen/mit leerzeichen").unwrap();
        g.write("unterlagen/mit leerzeichen/ä.md", b"Text").unwrap();
        assert!(
            g.resolve("unterlagen/mit leerzeichen/ä.md")
                .unwrap()
                .exists()
        );
        assert!(g.upload("unterlagen/mit leerzeichen/ä.md", b"neu").is_err());
        assert!(g.trash("unterlagen/mit leerzeichen/ä.md", false).is_err());
        g.trash("unterlagen/mit leerzeichen/ä.md", true).unwrap();
    }
    #[cfg(unix)]
    #[test]
    fn links_are_rejected() {
        let t = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        let g = Guard::new(t.path()).unwrap();
        std::os::unix::fs::symlink(outside.path(), t.path().join("link")).unwrap();
        assert!(g.write("link/escape", b"x").is_err());
        assert!(!outside.path().join("escape").exists());
        fs::write(outside.path().join("original"), b"secret").unwrap();
        fs::hard_link(outside.path().join("original"), t.path().join("hard")).unwrap();
        assert!(g.write("hard", b"x").is_err());
    }
    #[test]
    fn prefix_collision_and_move() {
        let t = tempfile::tempdir().unwrap();
        let g = Guard::new(t.path()).unwrap();
        g.mkdir("a").unwrap();
        g.mkdir("b").unwrap();
        g.upload("a/x", b"x").unwrap();
        assert!(g.move_path("a/x", "../escape").is_err());
        g.move_path("a/x", "b/x").unwrap();
        assert!(g.trash("b", true).is_err());
    }
    #[cfg(windows)]
    #[test]
    fn junction_is_rejected() {
        use std::process::Command;
        let t = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        let g = Guard::new(t.path()).unwrap();
        let status = Command::new("cmd")
            .args(["/C", "mklink", "/J"])
            .arg(t.path().join("junction"))
            .arg(outside.path())
            .status()
            .unwrap();
        assert!(status.success());
        assert!(g.write("junction/escape", b"x").is_err());
    }
}
