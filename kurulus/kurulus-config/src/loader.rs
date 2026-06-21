use bevy::prelude::*;
use bevy::asset::io::Reader;
use bevy::asset::{AssetLoader, LoadContext};
use kurulus_core::resources::config::*;
use std::marker::PhantomData;

/// Generic RON asset loader for config files
pub struct RonAssetLoader<T: bevy::asset::Asset + for<'de> serde::Deserialize<'de>> {
    _marker: PhantomData<T>,
}

impl<T: bevy::asset::Asset + for<'de> serde::Deserialize<'de>> Default for RonAssetLoader<T> {
    fn default() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<T: bevy::asset::Asset + for<'de> serde::Deserialize<'de>> AssetLoader for RonAssetLoader<T> {
    type Asset = T;
    type Settings = ();
    type Error = RonLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        _load_context: &mut LoadContext<'_>,
    ) -> Result<T, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let content = String::from_utf8(bytes)?;
        let config = ron::from_str(&content)?;
        Ok(config)
    }

    fn extensions(&self) -> &[&str] {
        &["ron"]
    }
}

/// Register all config asset types with the app
pub fn register_config_assets(app: &mut App) {
    app.init_asset::<UnitConfig>()
        .init_asset::<BuildingConfig>()
        .init_asset::<HeroConfig>()
        .init_asset::<MissionDef>()
        .register_asset_loader::<RonAssetLoader<UnitConfig>>(RonAssetLoader::default())
        .register_asset_loader::<RonAssetLoader<BuildingConfig>>(RonAssetLoader::default())
        .register_asset_loader::<RonAssetLoader<HeroConfig>>(RonAssetLoader::default())
        .register_asset_loader::<RonAssetLoader<MissionDef>>(RonAssetLoader::default());
}

// --- Error type ---

#[derive(Debug)]
pub enum RonLoaderError {
    Io(std::io::Error),
    Utf8(std::string::FromUtf8Error),
    Ron(ron::error::SpannedError),
}

impl std::fmt::Display for RonLoaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RonLoaderError::Io(e) => write!(f, "IO error: {}", e),
            RonLoaderError::Utf8(e) => write!(f, "UTF-8 error: {}", e),
            RonLoaderError::Ron(e) => write!(f, "RON error: {}", e),
        }
    }
}

impl std::error::Error for RonLoaderError {}

impl From<std::io::Error> for RonLoaderError {
    fn from(e: std::io::Error) -> Self {
        RonLoaderError::Io(e)
    }
}

impl From<std::string::FromUtf8Error> for RonLoaderError {
    fn from(e: std::string::FromUtf8Error) -> Self {
        RonLoaderError::Utf8(e)
    }
}

impl From<ron::error::SpannedError> for RonLoaderError {
    fn from(e: ron::error::SpannedError) -> Self {
        RonLoaderError::Ron(e)
    }
}
