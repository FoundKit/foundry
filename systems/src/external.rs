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
    let mut seen_slugs = std::collections::HashSet::new();

    let external_dirs = get_external_search_paths();

    for dir_path in external_dirs {
        if !dir_path.exists() {
            continue;
        }

        // Case 1: dir_path itself contains subsystem.json
        let direct_manifest = dir_path.join("subsystem.json");
        if direct_manifest.exists() {
            if let Some(m) = try_load_manifest_module(&dir_path, &direct_manifest)
                .filter(|m| seen_slugs.insert(m.slug().to_string()))
            {
                modules.push(m);
            }
            continue;
        }

        // Case 2: dir_path is a parent directory containing multiple subsystem subfolders
        if dir_path.is_dir() {
            let entries = match fs::read_dir(&dir_path) {
                Ok(e) => e,
                Err(err) => {
                    warn!(
                        "Failed to read external subsystem dir {:?}: {}",
                        dir_path, err
                    );
                    continue;
                }
            };

            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }
                let manifest_path = path.join("subsystem.json");
                if !manifest_path.exists() {
                    continue;
                }
                if let Some(m) = try_load_manifest_module(&path, &manifest_path)
                    .filter(|m| seen_slugs.insert(m.slug().to_string()))
                {
                    modules.push(m);
                }
            }
        }
    }

    modules
}

fn try_load_manifest_module(
    root_dir: &std::path::Path,
    manifest_path: &std::path::Path,
) -> Option<Box<dyn SubsystemModule>> {
    match fs::read_to_string(manifest_path) {
        Ok(content) => match serde_json::from_str::<ExternalSubsystemManifest>(&content) {
            Ok(manifest) => {
                info!(
                    "Loaded external standalone subsystem: {} ({}) from {:?}",
                    manifest.display_name, manifest.slug, root_dir
                );
                Some(Box::new(ExternalSubsystemModule {
                    manifest,
                    root_dir: root_dir.to_path_buf(),
                }))
            }
            Err(e) => {
                warn!("Failed to parse manifest at {:?}: {}", manifest_path, e);
                None
            }
        },
        Err(e) => {
            warn!("Failed to read manifest file at {:?}: {}", manifest_path, e);
            None
        }
    }
}

fn get_external_search_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Ok(env_path) = env::var("FOUNDRY_SYSTEMS_DIR") {
        for p in env_path.split([';', ':']) {
            if !p.trim().is_empty() {
                paths.push(PathBuf::from(p.trim()));
            }
        }
    }
    // Default search paths in current working directory
    paths.push(PathBuf::from("./external_systems"));
    paths.push(PathBuf::from("../external_systems"));
    paths
}
