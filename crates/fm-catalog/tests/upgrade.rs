use fm_catalog::{Catalog, Collection};
use fm_core::{Agent, Config, Guard};
use rusqlite::Connection;

#[test]
fn upgrade_preserves_legacy_content_and_creates_readable_snapshot() {
    let t = tempfile::tempdir().unwrap();
    let guard = Guard::new(t.path()).unwrap();
    let config = Config {
        name: "Bisheriger Nutzer".into(),
        api_key_ref: "bestehender-credential-manager-verweis".into(),
        password_hash: "bestehender-hash".into(),
        ..Config::default()
    };
    fm_core::initialize(&guard, &config).unwrap();
    let original_config =
        std::fs::read(guard.resolve(".folder-mind/config.toml").unwrap()).unwrap();
    guard
        .upload(
            "unterlagen/mein-dokument.bin",
            b"Unveraenderte Originaldaten",
        )
        .unwrap();
    guard
        .upload("wissen/meine.notes.md", b"Eigene Notizen")
        .unwrap();
    let path = guard.resolve(".folder-mind/folder-mind.sqlite").unwrap();
    let c = Connection::open(&path).unwrap();
    c.execute_batch(include_str!("../../../migrations/001_initial.sql"))
        .unwrap();
    c.execute_batch(include_str!("../../../migrations/002_source_archive.sql"))
        .unwrap();
    c.execute_batch("INSERT INTO chats VALUES('alter-chat','Gespeichertes Gespraech'); INSERT INTO messages VALUES('alte-nachricht','alter-chat','user','Bisherige Eingabe','[]'); INSERT INTO settings VALUES('beliebiger-wert','bleibt-erhalten');").unwrap();
    drop(c);
    let catalog = Catalog::open(&guard).unwrap();
    assert_eq!(
        catalog.rows("SELECT title FROM chats").unwrap()[0][0],
        "Gespeichertes Gespraech"
    );
    assert_eq!(
        catalog.rows("SELECT content FROM messages").unwrap()[0][0],
        "Bisherige Eingabe"
    );
    assert_eq!(
        catalog
            .rows("SELECT value FROM settings WHERE key='beliebiger-wert'")
            .unwrap()[0][0],
        "bleibt-erhalten"
    );
    assert_eq!(
        std::fs::read(guard.resolve(".folder-mind/config.toml").unwrap()).unwrap(),
        original_config
    );
    assert_eq!(
        Config::load(&guard).unwrap().api_key_ref,
        config.api_key_ref
    );
    assert_eq!(
        std::fs::read(guard.resolve("wissen/meine.notes.md").unwrap()).unwrap(),
        b"Eigene Notizen"
    );
    let snapshots = std::fs::read_dir(guard.resolve("backups").unwrap())
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect::<Vec<_>>();
    assert_eq!(snapshots.len(), 2);
    let old = Connection::open(
        snapshots
            .iter()
            .find(|p| p.extension().is_some_and(|s| s == "sqlite"))
            .unwrap(),
    )
    .unwrap();
    assert_eq!(
        old.query_row("PRAGMA user_version", [], |r| r.get::<_, u32>(0))
            .unwrap(),
        2
    );
    assert_eq!(
        old.query_row("SELECT content FROM messages", [], |r| r
            .get::<_, String>(0))
            .unwrap(),
        "Bisherige Eingabe"
    );
    drop(old);
    let agent = Agent {
        id: "mein-agent".into(),
        name: "Mein gespeicherter Agent".into(),
        ..Agent::default()
    };
    catalog
        .put_entry(Collection::Agents, &agent.id, &agent)
        .unwrap();
    drop(catalog);
    let catalog = Catalog::open(&guard).unwrap();
    assert_eq!(
        catalog.entries::<Agent>(Collection::Agents).unwrap()[0].name,
        agent.name
    );
    assert_eq!(
        std::fs::read_dir(guard.resolve("backups").unwrap())
            .unwrap()
            .count(),
        2
    );
    drop(catalog);
    let c = Connection::open(&path).unwrap();
    c.execute_batch("PRAGMA user_version=999;").unwrap();
    drop(c);
    assert!(Catalog::open(&guard).is_err());
    assert_eq!(
        Connection::open(path)
            .unwrap()
            .query_row("PRAGMA user_version", [], |r| r.get::<_, u32>(0))
            .unwrap(),
        999
    );
}
