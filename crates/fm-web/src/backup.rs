use super::*;
use std::{
    collections::{BTreeMap, BTreeSet},
    io::{Cursor, Read, Write},
};
use zip::{ZipArchive, ZipWriter, write::SimpleFileOptions};
const LIMIT: u64 = 512 * 1024 * 1024;
#[derive(serde::Serialize, Deserialize)]
struct Manifest {
    version: u32,
    files: BTreeMap<String, String>,
}
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/backup", get(list).post(create))
        .route("/api/backup/preview", post(preview))
        .route("/api/backup/restore", post(restore))
        .route("/api/backup/download", get(download))
        .route("/api/data/change", post(change_data))
}
fn archive_path(name: &str) -> Result<String> {
    if !name.ends_with(".zip")
        || Path::new(name).components().count() != 1
        || name.contains(['/', '\\'])
    {
        return Err(error("Ungültiger Sicherungsname"));
    }
    Ok(format!("backups/{name}"))
}
fn allowed_path(p: &str) -> bool {
    [
        "unterlagen/",
        "wissen/",
        "chats/",
        "extrakte/",
        "input/",
        "eingangsarchive/",
    ]
    .iter()
    .any(|prefix| p.starts_with(prefix))
        || p == ".folder-mind/config.toml"
        || p == ".folder-mind/folder-mind.sqlite"
}
pub(crate) fn make(l: &Library) -> Result<String> {
    let _busy = l.processing.lock().map_err(error)?;
    make_locked(l)
}
fn make_locked(l: &Library) -> Result<String> {
    let snapshot = format!("backups/.snapshot-{}.sqlite", uuid::Uuid::new_v4());
    l.catalog.snapshot(&l.guard, &snapshot)?;
    let mut files = Vec::new();
    for dir in [
        "unterlagen",
        "wissen",
        "chats",
        "extrakte",
        "input",
        "eingangsarchive",
    ] {
        walk(&l.guard, dir, &mut files)?;
    }
    files.push(".folder-mind/config.toml".into());
    files.push(".folder-mind/folder-mind.sqlite".into());
    if files.len() > 9999 {
        return Err(error(
            "Die Sicherung enthält mehr als 9999 Dateien. Bitte den Bestand auf mehrere Datenordner verteilen.",
        ));
    }
    let mut manifest = Manifest {
        version: 1,
        files: BTreeMap::new(),
    };
    let mut archive = ZipWriter::new(Cursor::new(Vec::new()));
    let mut total = 0u64;
    for path in files {
        let source = if path == ".folder-mind/folder-mind.sqlite" {
            snapshot.as_str()
        } else {
            &path
        };
        let source = l.guard.resolve(source)?;
        let size = std::fs::metadata(&source)?.len();
        if size > LIMIT - total {
            return Err(error(
                "Die Sicherung überschreitet die Grenze von 512 MB. Unterlagen bitte auf mehrere Datenordner verteilen.",
            ));
        }
        let mut bytes = Vec::new();
        std::fs::File::open(source)?
            .take(LIMIT - total + 1)
            .read_to_end(&mut bytes)?;
        if bytes.len() as u64 > LIMIT - total {
            return Err(error(
                "Eine Datei hat während der Sicherung die Grössengrenze überschritten",
            ));
        }
        total += bytes.len() as u64;
        manifest.files.insert(path.clone(), fm_core::hash(&bytes));
        archive
            .start_file(
                path,
                SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated),
            )
            .map_err(error)?;
        archive.write_all(&bytes)?;
    }
    archive
        .start_file("manifest.json", SimpleFileOptions::default())
        .map_err(error)?;
    archive.write_all(&serde_json::to_vec_pretty(&manifest).map_err(error)?)?;
    let bytes = archive.finish().map_err(error)?.into_inner();
    let name = format!(
        "folder-mind-{}-{}.zip",
        fm_core::now(),
        uuid::Uuid::new_v4()
    );
    l.guard.upload(archive_path(&name)?, &bytes)?;
    l.guard.trash(snapshot, true)?;
    Ok(name)
}
fn checked(l: &Library, name: &str) -> Result<(String, BTreeMap<String, Vec<u8>>)> {
    let bytes = std::fs::read(l.guard.resolve(archive_path(name)?)?)?;
    if bytes.len() as u64 > LIMIT {
        return Err(error("Sicherungsdatei ist zu gross"));
    }
    let hash = fm_core::hash(&bytes);
    let mut zip = ZipArchive::new(Cursor::new(bytes)).map_err(error)?;
    if zip.len() > 10000 {
        return Err(error("Zu viele Dateien in der Sicherung"));
    }
    let mut entries = BTreeMap::new();
    let mut folded = BTreeSet::new();
    let mut total = 0u64;
    for i in 0..zip.len() {
        let mut f = zip.by_index(i).map_err(error)?;
        let p = f.name().to_string();
        if f.is_dir() {
            continue;
        }
        if f.unix_mode().is_some_and(|m| m & 0o170000 == 0o120000) {
            return Err(error("Sicherung enthält einen symbolischen Link"));
        }
        if p != "manifest.json" && !allowed_path(&p) {
            return Err(error("Sicherung enthält einen nicht erlaubten Pfad"));
        }
        l.guard.resolve(&p)?;
        if !folded.insert(p.to_lowercase()) {
            return Err(error("Sicherung enthält doppelte Dateipfade"));
        }
        if f.size() > LIMIT - total {
            return Err(error(
                "Entpackte Sicherung überschreitet die Grenze von 512 MB",
            ));
        }
        let mut content = Vec::new();
        f.by_ref()
            .take(LIMIT - total + 1)
            .read_to_end(&mut content)?;
        if content.len() as u64 > LIMIT - total {
            return Err(error("Sicherungsdatei ist zu gross"));
        }
        total += content.len() as u64;
        entries.insert(p, content);
    }
    let manifest: Manifest = serde_json::from_slice(
        &entries
            .remove("manifest.json")
            .ok_or_else(|| error("Prüfmanifest fehlt"))?,
    )
    .map_err(error)?;
    if manifest.version != 1 || manifest.files.len() != entries.len() {
        return Err(error("Ungültiges Sicherungsmanifest"));
    }
    for (p, bytes) in &entries {
        if manifest.files.get(p) != Some(&fm_core::hash(bytes)) {
            return Err(error("Prüfsumme der Sicherung stimmt nicht"));
        }
    }
    if !entries.contains_key(".folder-mind/folder-mind.sqlite")
        || !entries.contains_key(".folder-mind/config.toml")
    {
        return Err(error("Konfiguration oder Datenbank fehlt in der Sicherung"));
    }
    let _: Config = Config::from_toml(
        std::str::from_utf8(entries.get(".folder-mind/config.toml").unwrap()).map_err(error)?,
    )?;
    Ok((hash, entries))
}
async fn list(State(s): State<AppState>) -> ApiResult<Json<Value>> {
    let l = s.lib()?;
    let mut files = Vec::new();
    for e in std::fs::read_dir(l.guard.resolve("backups")?)
        .map_err(fm_core::Error::from)?
        .flatten()
    {
        if e.path().extension().is_some_and(|s| s == "zip") {
            let name = e.file_name().to_string_lossy().to_string();
            let p = l.guard.resolve(archive_path(&name)?)?;
            files.push(json!({"name":name,"size":std::fs::metadata(p).map_err(fm_core::Error::from)?.len()}));
        }
    }
    Ok(Json(json!({"files":files})))
}
async fn create(State(s): State<AppState>) -> ApiResult<Json<Value>> {
    let l = s.lib()?;
    let name = tokio::task::spawn_blocking(move || make(&l))
        .await
        .map_err(error)??;
    Ok(Json(json!({"ok":true,"name":name})))
}
#[derive(Deserialize)]
struct BackupInput {
    name: String,
    hash: Option<String>,
    #[serde(default)]
    confirmed: bool,
}
async fn preview(State(s): State<AppState>, Json(p): Json<BackupInput>) -> ApiResult<Json<Value>> {
    let l = s.lib()?;
    let (hash, entries) = checked(&l, &p.name)?;
    let files = entries
        .iter()
        .map(|(path, bytes)| {
            Ok(json!({"path":path,"size":bytes.len(),"exists":l.guard.resolve(path)?.exists()}))
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(Json(
        json!({"hash":hash,"files":files,"message":"Vorhandene Dateien werden ersetzt. Zusätzliche Dateien bleiben erhalten. Netzwerkfreigabe und Cloud-Zustimmung werden deaktiviert. Vorher wird eine neue Sicherung angelegt."}),
    ))
}
async fn restore(State(s): State<AppState>, Json(p): Json<BackupInput>) -> ApiResult<Json<Value>> {
    if !p.confirmed {
        return Err(error("Wiederherstellung muss nach dem Probelauf bestätigt werden").into());
    }
    let l = s.lib()?;
    let lib = l.clone();
    let expected = p
        .hash
        .ok_or_else(|| error("Bitte zuerst den Probelauf ausführen"))?;
    let safety = tokio::task::spawn_blocking(move || -> Result<String> {
        let _busy = lib.processing.lock().map_err(error)?;
        let (hash, entries) = checked(&lib, &p.name)?;
        if hash != expected {
            return Err(error("Sicherung wurde nach dem Probelauf verändert"));
        }
        let safety = make_locked(&lib)?;
        let mut config: Config = Config::from_toml(
            std::str::from_utf8(&entries[".folder-mind/config.toml"]).map_err(error)?,
        )?;
        config.network_enabled = false;
        config.cloud_consent = false;
        config.autostart = false;
        config.api_key_ref = lib.config.read().map_err(error)?.api_key_ref.clone();
        let stage = format!("backups/.restore-{}.sqlite", uuid::Uuid::new_v4());
        lib.guard
            .upload(&stage, &entries[".folder-mind/folder-mind.sqlite"])?;
        lib.catalog.restore(&lib.guard, &stage)?;
        lib.guard.trash(stage, true)?;
        for (path, bytes) in entries {
            if path.starts_with(".folder-mind/") {
                continue;
            }
            if let Some(parent) = Path::new(&path).parent() {
                lib.guard.mkdir(parent)?;
            }
            lib.guard.write(&path, &bytes)?;
        }
        config.save(&lib.guard)?;
        fm_access::set_autostart(false)?;
        *lib.config.write().map_err(error)? = config;
        lib.index.rebuild(&lib.catalog.chunks()?)?;
        Ok(safety)
    })
    .await
    .map_err(error)??;
    s.sessions.lock().map_err(error)?.clear();
    Ok(Json(json!({"ok":true,"safety_backup":safety})))
}
#[derive(Deserialize)]
struct Download {
    name: String,
}
async fn download(State(s): State<AppState>, Query(p): Query<Download>) -> ApiResult<Response> {
    let l = s.lib()?;
    let path = archive_path(&p.name)?;
    let bytes = std::fs::read(l.guard.resolve(path)?).map_err(fm_core::Error::from)?;
    Ok((
        [
            (header::CONTENT_TYPE, "application/zip"),
            (header::CONTENT_DISPOSITION, "attachment"),
        ],
        bytes,
    )
        .into_response())
}
#[derive(Deserialize)]
struct ChangeData {
    path: String,
    #[serde(default)]
    take_data: bool,
    #[serde(default)]
    confirmed: bool,
}
async fn change_data(
    State(s): State<AppState>,
    Json(p): Json<ChangeData>,
) -> ApiResult<Json<Value>> {
    if !p.confirmed {
        return Err(error("Datenordnerwechsel bitte bestätigen").into());
    }
    let l = s.lib()?;
    let target = Guard::new(&p.path)?;
    if target.root().starts_with(l.guard.root()) || l.guard.root().starts_with(target.root()) {
        return Err(error("Alter und neuer Datenordner dürfen nicht ineinander liegen").into());
    }
    let _busy = l.processing.lock().map_err(error)?;
    let mut config = l.config.read().map_err(error)?.clone();
    config.network_enabled = false;
    if p.take_data {
        if std::fs::read_dir(target.root())
            .map_err(fm_core::Error::from)?
            .next()
            .is_some()
        {
            return Err(
                error("Zum Mitnehmen der Daten bitte einen leeren Zielordner wählen").into(),
            );
        }
        fm_core::initialize(&target, &config)?;
        let mut files = Vec::new();
        for dir in [
            "unterlagen",
            "extrakte",
            "wissen",
            "chats",
            "protokolle",
            "backups",
            "input",
            "eingangsarchive",
        ] {
            walk(&l.guard, dir, &mut files)?;
        }
        for path in files {
            if let Some(parent) = Path::new(&path).parent() {
                target.mkdir(parent)?;
            }
            target.upload(
                &path,
                &std::fs::read(l.guard.resolve(&path)?).map_err(fm_core::Error::from)?,
            )?;
        }
        l.catalog
            .snapshot(&target, ".folder-mind/folder-mind.sqlite")?;
        config.save(&target)?;
    } else if !target.resolve(".folder-mind/config.toml")?.exists() {
        fm_core::initialize(&target, &config)?;
    }
    let new = Arc::new(Library::open(target)?);
    {
        let mut c = new.config.write().map_err(error)?;
        c.network_enabled = false;
        c.save(&new.guard)?;
    }
    std::fs::write(&s.pointer, new.guard.root().to_string_lossy().as_bytes())
        .map_err(fm_core::Error::from)?;
    *s.library.write().map_err(error)? = Some(new);
    s.sessions.lock().map_err(error)?.clear();
    Ok(Json(json!({"ok":true})))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn zip_slip_is_blocked() {
        let t = tempfile::tempdir().unwrap();
        let g = Guard::new(t.path()).unwrap();
        fm_core::initialize(&g, &Config::default()).unwrap();
        let l = Library::open(g).unwrap();
        for path in [
            "../escape",
            "unterlagen/../../escape",
            ".folder-mind/tls-key.pem",
            "C:/escape",
        ] {
            let mut z = ZipWriter::new(Cursor::new(Vec::new()));
            z.start_file(path, SimpleFileOptions::default()).unwrap();
            z.write_all(b"bad").unwrap();
            let bytes = z.finish().unwrap().into_inner();
            l.guard.write("backups/test.zip", &bytes).unwrap();
            assert!(checked(&l, "test.zip").is_err(), "{path}");
        }
    }
    #[test]
    fn round_trip_manifest_and_tamper() {
        let t = tempfile::tempdir().unwrap();
        let g = Guard::new(t.path()).unwrap();
        fm_core::initialize(&g, &Config::default()).unwrap();
        g.upload("unterlagen/test.txt", b"Originaltext").unwrap();
        let l = Library::open(g).unwrap();
        let name = make(&l).unwrap();
        let (_, files) = checked(&l, &name).unwrap();
        assert_eq!(files["unterlagen/test.txt"], b"Originaltext");
        assert!(!files.keys().any(|p| p.contains("tls-key")));
        l.guard
            .write(archive_path(&name).unwrap(), b"kaputt")
            .unwrap();
        assert!(checked(&l, &name).is_err());
    }
}
