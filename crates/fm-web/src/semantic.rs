use super::*;
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/semantic/rebuild", post(rebuild))
        .route("/api/semantic/search", post(search))
}
pub async fn retrieve(s: &AppState, l: &Library, query: &str, scope: &str) -> Result<Vec<Chunk>> {
    let config = l.config.read().map_err(error)?.clone();
    if config.embedding_model.is_empty() {
        return Err(error("Bitte zuerst ein Embedding-Modell einstellen"));
    }
    let vectors = fm_index::Vectors::load(&l.guard)
        .map_err(|_| error("Bitte zuerst den semantischen Index aufbauen"))?;
    if vectors.model != config.embedding_model || vectors.base_url != config.base_url {
        return Err(error(
            "Der semantische Index gehört zu einem anderen Anbieter oder Modell. Bitte neu aufbauen.",
        ));
    }
    let _permit = s.ai_jobs.clone().acquire_owned().await.map_err(error)?;
    let key = fm_ai::load_key(&config.api_key_ref)?;
    let client = fm_ai::Client::new(config, key)?;
    let query = client.embeddings(vec![query.into()]).await?;
    let chunks = l
        .catalog
        .chunks()?
        .into_iter()
        .filter(|c| in_scope(&c.path, scope))
        .collect::<Vec<_>>();
    let ids = vectors.search(&query[0], &chunks, 20);
    Ok(ids
        .into_iter()
        .filter_map(|id| chunks.iter().find(|c| c.id == id).cloned())
        .collect())
}
async fn rebuild(State(s): State<AppState>) -> ApiResult<Json<Value>> {
    let _permit = s.ai_jobs.clone().acquire_owned().await.map_err(error)?;
    let l = s.lib()?;
    let config = l.config.read().map_err(error)?.clone();
    let client = fm_ai::Client::new(config.clone(), fm_ai::load_key(&config.api_key_ref)?)?;
    let chunks = l.catalog.chunks()?;
    let mut entries = Vec::new();
    for batch in chunks.chunks(16) {
        let vectors = client
            .embeddings(batch.iter().map(|c| c.text.clone()).collect())
            .await?;
        for (c, vector) in batch.iter().zip(vectors) {
            entries.push(fm_index::VectorEntry {
                id: c.id.clone(),
                hash: c.hash.clone(),
                vector,
            });
        }
    }
    let count = entries.len();
    fm_index::Vectors {
        model: config.embedding_model,
        base_url: config.base_url,
        entries,
    }
    .save(&l.guard)?;
    Ok(Json(json!({"ok":true,"count":count})))
}
#[derive(Deserialize)]
struct Search {
    q: String,
    #[serde(default)]
    scope: String,
}
async fn search(State(s): State<AppState>, Json(p): Json<Search>) -> ApiResult<Json<Vec<Chunk>>> {
    let l = s.lib()?;
    Ok(Json(retrieve(&s, &l, &p.q, &p.scope).await?))
}
