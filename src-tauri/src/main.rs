use huggingface_hub::{api::tokio::Api, Repo, RepoType};
mod model_manager;

#[tokio::main]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
async fn main() {
    let api = Api::new()?;
    let repo = api.repo(Repo::with_revision("google/vit-base-patch16-224".to_string(), RepoType::Model, "main".to_string()));

    // Télécharge le fichier du modèle
    let model_path = repo.get("pytorch_model.bin").await?;
    println!("Fichier du modèle téléchargé: {:?}", model_path);

    // Télécharge le fichier de configuration
    let config_path = repo.get("config.json").await?;
    println!("Fichier de configuration téléchargé: {:?}", config_path);

    // Télécharge le tokenizer si nécessaire (pour d'autres modèles)
    // let tokenizer_path = repo.get("tokenizer.json").await?;
    // println!("Fichier du tokenizer téléchargé: {:?}", tokenizer_path);

    Ok(())

    nakama_lib::run()
}
