use hf_hub::{api::tokio::Api, Repo, RepoType};
use std::path::PathBuf;

pub async fn download_model_files() -> Result<(PathBuf, PathBuf), Box<dyn std::error::Error>> {
    let api = Api::new()?;
    let repo = api.repo(Repo::with_revision(
        "google/vit-base-patch16-224".to_string(),
        RepoType::Model,
        "main".to_string(),
    ));

    // Télécharge le fichier du modèle
    let model_path = repo.get("pytorch_model.bin").await?;
    println!("Fichier du modèle téléchargé: {:?}", model_path);

    // Télécharge le fichier de configuration
    let config_path = repo.get("config.json").await?;
    println!("Fichier de configuration téléchargé: {:?}", config_path);

    Ok((model_path, config_path))
}
