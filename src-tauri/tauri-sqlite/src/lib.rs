use std::{
    fs::create_dir_all,
    path::{Path, PathBuf},
    str::FromStr,
};

use anyhow::Result;
use sqlx::{sqlite::SqliteConnectOptions, SqlitePool};
use tauri::{
    async_runtime::block_on,
    plugin::{self, TauriPlugin},
    AppHandle, Manager, Runtime,
};

async fn connect_sqlite<R: Runtime, P: AsRef<Path>>(
    app: &AppHandle<R>,
    path: P,
) -> Result<SqlitePool> {
    let app_dir = app
        .path_resolver()
        .app_dir()
        .ok_or_else(|| anyhow::anyhow!("App can not read/write SQLite files."))?;
    create_dir_all(&app_dir)?;
    let sqlite_path = app_dir.join(path.as_ref());
    let filename = sqlite_path
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("Invalid SQLite path: {:?}", sqlite_path))?;

    let options =
        SqliteConnectOptions::from_str(&format!("sqlite:{}", filename))?.create_if_missing(true);
    let pool = SqlitePool::connect_with(options)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to open or create {:?}: {:?}", filename, e))?;
    Ok(pool)
}

pub struct SqlitePluginBuilder {
    path: PathBuf,
}

impl SqlitePluginBuilder {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
        }
    }
    pub fn build<R: Runtime>(self) -> TauriPlugin<R> {
        plugin::Builder::new("sqlite")
            .setup(move |app_handle| {
                let pool = block_on(connect_sqlite(app_handle, self.path))?;
                app_handle.manage(pool);
                Ok(())
            })
            .build()
    }
}
