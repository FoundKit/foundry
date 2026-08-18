use axum::Router;
use foundry_core::{CustomAdminPageSpec, SubsystemModule};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::PathBuf;
use tower_http::services::ServeDir;
use tracing::{info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalSubsystemManifest {
    pub slug: String,
    pub display_name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub custom_pages: Vec<CustomAdminPageSpec>,
}

#[derive(Debug, Clone)]
pub struct ExternalSubsystemModule {
    pub manifest: ExternalSubsystemManifest,
    pub root_dir: PathBuf,
}

impl SubsystemModule for ExternalSubsystemModule {
    fn slug(&self) -> &'static str {
        Box::leak(self.manifest.slug.clone().into_boxed_str())
    }

    fn display_name(&self) -> &'static str {
        Box::leak(self.manifest.display_name.clone().into_boxed_str())
    }

    fn description(&self) -> &'static str {
        Box::leak(self.manifest.description.clone().into_boxed_str())
    }

    fn register_routes(&self, router: Router) -> Router {
        let custom_pages_dir = self.root_dir.join("custom_pages");
        if custom_pages_dir.exists() {
            let serve_dir = ServeDir::new(custom_pages_dir);
            router.nest_service("/custom-pages", serve_dir)
        } else {
            router
        }
    }

    fn custom_admin_pages(&self) -> Vec<CustomAdminPageSpec> {
        self.manifest.custom_pages.clone()
    }
}

/// Discover and load external standalone subsystem directories
pub fn load_external_subsystems() -> Vec<Box<dyn SubsystemModule>> {
    let mut modules: Vec<Box<dyn SubsystemModule>> = Vec::new();

    let external_dirs = get_external_search_paths();

    for dir_path in external_dirs {
        if !dir_path.exists() || !dir_path.is_dir() {
            continue;
        }

        let entries = match fs::read_dir(&dir_path) {
            Ok(e) => e,
            Err(err) => {
                warn!("Failed to read external subsystem dir {:?}: {}", dir_path, err);
                continue;
            }
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let manifest_path = path.join("subsystem.json");
                if manifest_path.exists() {
                    match fs::read_to_string(&manifest_path) {
                        Ok(content) => match serde_json::from_str::<ExternalSubsystemManifest>(&content) {
                            Ok(manifest) => {
                                info!(
                                    "Loaded external subsystem: {} ({}) from {:?}",
                                    manifest.display_name, manifest.slug, path
                                );
                                modules.push(Box::new(ExternalSubsystemModule {
                                    manifest,
                                    root_dir: path,
                                }));
                            }
                            Err(e) => {
                                warn!("Failed to parse manifest at {:?}: {}", manifest_path, e);
                            }
                        },
                        Err(e) => {
                            warn!("Failed to read manifest file at {:?}: {}", manifest_path, e);
                        }
                    }
                }
            }
        }
    }

    modules
}

fn get_external_search_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Ok(env_path) = env::var("FOUNDRY_SYSTEMS_DIR") {
        paths.push(PathBuf::from(env_path));
    }
    // Default search paths in current working directory
    paths.push(PathBuf::from("./external_systems"));
    paths.push(PathBuf::from("../external_systems"));
    paths
}
