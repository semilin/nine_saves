use anyhow::{anyhow, Context, Result};
use directories::BaseDirs;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

const NS_ID: &str = "1809540";

fn save_directory(base_dirs: &BaseDirs) -> Result<PathBuf> {
    let path = if cfg!(target_os = "windows") {
        let mut path = base_dirs.home_dir().to_owned();
        path.extend(&["AppData", "LocalLow", "RedCandleGames", "NineSols"]);
        path
    } else if cfg!(target_os = "macos") {
        todo!("find out macos save directory")
    } else if cfg!(target_os = "linux") {
        let mut path = base_dirs.data_dir().to_owned();
        path.extend(&[
            "Steam",
            "steamapps",
            "compatdata",
            NS_ID,
            "pfx",
            "drive_c",
            "users",
            "steamuser",
            "AppData",
            "LocalLow",
            "RedCandleGames",
            "NineSols",
        ]);
        path
    } else {
        panic!("OS not supported.")
    };

    match path.exists() {
        true => Ok(path),
        false => Err(anyhow!("Could not find Nine Sols save directory at {}. Please report this bug along with the path that your saves are actually stored.", path.display()))
    }
}

fn saves_from_dir(dir: &Path) -> Result<Vec<Save>> {
    Ok(fs::read_dir(dir)
        .context("couldn't read external saves directory")?
        .filter_map(|x| match x {
            Ok(x) => Some(x),
            Err(_) => None,
        })
        .filter_map(|p| {
            let name = p.file_name().into_string();
            match name {
                Ok(n) => Some((n, p.path().to_owned())),
                Err(_) => None,
            }
        })
        .filter_map(|(name, path)| match path.is_dir() {
            true => Some(Save {
                name,
                path,
                nrp_backup: false,
                info: None,
            }),
            false => None,
        })
        .map(|s| {
            s.clone()
                .with_decrypted_info()
                .with_context(|| format!("couldn't decrypt info for save {:?}", &s))
                .unwrap()
        })
        .collect())
}

fn data_dir(base_dirs: &BaseDirs) -> Result<PathBuf> {
    Ok(base_dirs.data_dir().join("nine_saves"))
}

#[derive(Clone, Debug)]
pub struct Save {
    pub name: String,
    pub path: PathBuf,
    pub nrp_backup: bool,
    pub info: Option<SaveInfo>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SaveInfo {
    pub level: u8,
    #[serde(rename(deserialize = "playTime"))]
    pub playtime: f64,
    pub gold: u32,
    #[serde(rename(deserialize = "gameMode"))]
    pub gamemode: u8,
    #[serde(rename(deserialize = "atSceneGuid"))]
    pub atsceneguid: String,
}

#[derive(Debug, Default)]
pub struct SavesData {
    pub game_slots_dir: PathBuf,
    pub external_saves_dir: PathBuf,
    pub backups_dir: PathBuf,
    pub slots: Vec<Save>,
    pub saves: Vec<Save>,
    pub backups: Vec<Save>,
}

impl Save {
    pub fn with_decrypted_info(self) -> Result<Self> {
        let info = SaveInfo::decrypt_from(&self)?;
        Ok(Save {
            info: Some(info),
            ..self
        })
    }
    pub fn copy(&self, destination: &Path) -> Result<()> {
        fs::create_dir_all(destination).with_context(|| {
            format!("couldn't create destination directory ({:?})", destination)
        })?;
        for entry in fs::read_dir(&self.path)? {
            let file = entry?;
            let new_path = destination.join(
                file.path()
                    .file_name()
                    .with_context(|| format!("couldn't get filename of entry {:?}", file))?,
            );
            fs::copy(file.path(), new_path)?;
        }
        Ok(())
    }
    pub fn delete(&self) -> Result<()> {
        for entry in fs::read_dir(&self.path)? {
            let file = entry?;
            fs::remove_file(file.path())?;
        }
        Ok(())
    }
}

impl SavesData {
    pub fn refresh(&mut self) -> Result<()> {
        let re = Regex::new("saveslot([0-3])(_BeforeNoReturnPoint)?$")?;
        self.slots = fs::read_dir(&self.game_slots_dir)?
            .filter_map(|x| match x {
                Ok(x) => Some(x),
                Err(_) => None,
            })
            .filter_map(|p| {
                let name = p.file_name().into_string();
                match name {
                    Ok(n) => Some((n, p.path().to_owned())),
                    Err(_) => None,
                }
            })
            .filter_map(|p| {
                if let Some(caps) = re.captures(&p.0) {
                    match caps.get(1) {
                        Some(num) => {
                            let num: u8 = num.as_str().parse().unwrap();
                            match caps.get(2) {
                                Some(_) => Some(Save {
                                    name: format!("Slot {} (Before NRP)", num + 1),
                                    path: p.1,
                                    nrp_backup: true,
                                    info: None,
                                }),
                                None => Some(Save {
                                    name: format!("Slot {}", num + 1),
                                    path: p.1,
                                    nrp_backup: false,
                                    info: None,
                                }),
                            }
                        }
                        None => None,
                    }
                } else {
                    None
                }
            })
            .map(|s| s.with_decrypted_info().unwrap())
            .collect();
        fs::create_dir_all(&self.external_saves_dir)?;
        self.saves = saves_from_dir(&self.external_saves_dir)?;
        fs::create_dir_all(&self.backups_dir)?;
        self.backups = saves_from_dir(&self.backups_dir)?;

        self.slots.sort_by(|a, b| a.name.cmp(&b.name));
        self.saves.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(())
    }

    pub fn backup_and_overwrite(&self, source: &Save, destination: &Save) -> Result<()> {
        let backup_dst = &self
            .backups_dir
            .join(format!("{}_{}", self.backups.len(), &source.name));
        destination
            .copy(backup_dst)
            .with_context(|| format!("failed to back up save {}", destination.name))?;
        destination
            .delete()
            .with_context(|| format!("failed to delete save {}", destination.name))?;
        source
            .copy(&destination.path)
            .with_context(|| format!("failed to copy {} to {:?}", source.name, destination.path))?;
        Ok(())
    }

    pub fn new() -> Result<Self> {
        let base_dirs = BaseDirs::new().context("couldn't get base directories for OS")?;
        Ok(Self {
            game_slots_dir: save_directory(&base_dirs)?,
            external_saves_dir: data_dir(&base_dirs)?.join("saves"),
            backups_dir: data_dir(&base_dirs)?.join("backups"),
            slots: vec![],
            saves: vec![],
            backups: vec![],
        })
    }
}
