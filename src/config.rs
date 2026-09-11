use std::{
    collections::BTreeMap,
    env,
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, bail, Context, Result};
use atomicwrites::{AllowOverwrite, AtomicFile};
use fs2::FileExt;
use path_clean::PathClean;

pub type Workplaces = BTreeMap<String, String>;

pub struct ConfigStore {
    path: PathBuf,
}

impl ConfigStore {
    pub fn discover() -> Result<Self> {
        if let Some(path) = env::var_os("GOTO_CONFIG").filter(|value| !value.is_empty()) {
            return Ok(Self::new(path));
        }

        let base = platform_config_dir().context(
            "could not determine the user configuration directory; set GOTO_CONFIG explicitly",
        )?;
        Ok(Self::new(base.join("goto").join("workplaces.json")))
    }

    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn load(&self) -> Result<Workplaces> {
        load_file(&self.path)
    }

    pub fn save_inferred(
        &self,
        path: Option<&Path>,
        name: Option<&str>,
    ) -> Result<(String, String)> {
        let normalized = normalize_directory(path)?;
        let inferred;
        let chosen_name = match name {
            Some(value) => value,
            None => {
                inferred = infer_name(&normalized)?;
                inferred.as_str()
            }
        };
        validate_key(chosen_name)?;

        let value = normalized
            .to_str()
            .context("the directory path is not valid UTF-8 and cannot be stored in JSON")?;
        if value.contains(['\r', '\n']) {
            bail!("directory paths containing line breaks are not supported");
        }

        let _lock = self.exclusive_lock()?;
        let mut workplaces = self.load()?;
        if let Some(existing) = find_key(&workplaces, chosen_name) {
            bail!("a workplace named '{existing}' already exists");
        }

        workplaces.insert(chosen_name.to_owned(), value.to_owned());
        self.write(&workplaces)?;
        Ok((chosen_name.to_owned(), value.to_owned()))
    }

    pub fn resolve(&self, name: &str) -> Result<PathBuf> {
        validate_key(name)?;
        let workplaces = self.load()?;
        let stored_name = find_key(&workplaces, name)
            .ok_or_else(|| anyhow!("no workplace named '{name}' is saved"))?;
        let path = PathBuf::from(&workplaces[stored_name]);
        ensure_existing_directory(&path).with_context(|| {
            format!(
                "the saved workplace '{stored_name}' points to '{}'",
                path.display()
            )
        })?;
        Ok(path)
    }

    pub fn remove(&self, name: &str) -> Result<(String, String)> {
        validate_key(name)?;
        let _lock = self.exclusive_lock()?;
        let mut workplaces = self.load()?;
        let stored_name = find_key(&workplaces, name)
            .cloned()
            .ok_or_else(|| anyhow!("no workplace named '{name}' is saved"))?;
        let path = workplaces
            .remove(&stored_name)
            .expect("the key was found immediately before removal");
        self.write(&workplaces)?;
        Ok((stored_name, path))
    }

    fn exclusive_lock(&self) -> Result<ConfigLock> {
        let parent = self
            .path
            .parent()
            .context("the configuration file must have a parent directory")?;
        fs::create_dir_all(parent).with_context(|| {
            format!(
                "could not create configuration directory '{}'",
                parent.display()
            )
        })?;

        let lock_path = parent.join("workplaces.lock");
        let file = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(&lock_path)
            .with_context(|| format!("could not open lock file '{}'", lock_path.display()))?;
        file.lock_exclusive()
            .with_context(|| format!("could not lock '{}'", lock_path.display()))?;
        Ok(ConfigLock(file))
    }

    fn write(&self, workplaces: &Workplaces) -> Result<()> {
        let mut bytes = serde_json::to_vec_pretty(workplaces)
            .context("could not serialize the workplace configuration")?;
        bytes.push(b'\n');

        AtomicFile::new(&self.path, AllowOverwrite)
            .write(|file| {
                file.write_all(&bytes)?;
                file.sync_all()
            })
            .map_err(|error| {
                anyhow!(
                    "could not atomically write '{}': {error}",
                    self.path.display()
                )
            })
    }
}

struct ConfigLock(File);

impl Drop for ConfigLock {
    fn drop(&mut self) {
        let _ = FileExt::unlock(&self.0);
    }
}

pub fn validate_key(name: &str) -> Result<()> {
    let mut characters = name.chars();
    let Some(first) = characters.next() else {
        bail!("workplace names cannot be empty");
    };
    if !first.is_ascii_alphanumeric()
        || !characters.all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '.' | '-' | '_')
        })
    {
        bail!(
            "invalid workplace name '{name}'; use ASCII letters, numbers, dots, hyphens, or underscores, starting with a letter or number"
        );
    }
    Ok(())
}

fn load_file(path: &Path) -> Result<Workplaces> {
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(BTreeMap::new()),
        Err(error) => {
            return Err(error)
                .with_context(|| format!("could not read configuration '{}'", path.display()))
        }
    };

    let workplaces: Workplaces = serde_json::from_slice(&bytes).with_context(|| {
        format!(
            "configuration '{}' is not a JSON object containing string paths",
            path.display()
        )
    })?;
    validate_loaded(&workplaces, path)?;
    Ok(workplaces)
}

fn validate_loaded(workplaces: &Workplaces, path: &Path) -> Result<()> {
    let mut seen: Vec<&str> = Vec::with_capacity(workplaces.len());
    for (name, value) in workplaces {
        validate_key(name).with_context(|| {
            format!("configuration '{}' contains an invalid key", path.display())
        })?;
        if let Some(existing) = seen
            .iter()
            .find(|existing| existing.eq_ignore_ascii_case(name))
        {
            bail!(
                "configuration '{}' contains ambiguous keys '{existing}' and '{name}'",
                path.display()
            );
        }
        if !Path::new(value).is_absolute() {
            bail!(
                "configuration '{}' contains a non-absolute path for '{name}'",
                path.display()
            );
        }
        if value.contains(['\r', '\n']) {
            bail!(
                "configuration '{}' contains a path with line breaks for '{name}'",
                path.display()
            );
        }
        seen.push(name);
    }
    Ok(())
}

fn find_key<'a>(workplaces: &'a Workplaces, name: &str) -> Option<&'a String> {
    workplaces
        .keys()
        .find(|existing| existing.eq_ignore_ascii_case(name))
}

fn normalize_directory(path: Option<&Path>) -> Result<PathBuf> {
    let source = match path {
        Some(path) => expand_tilde(path)?,
        None => env::current_dir().context("could not determine the current directory")?,
    };
    let absolute = if source.is_absolute() {
        source
    } else {
        env::current_dir()
            .context("could not determine the current directory")?
            .join(source)
    }
    .clean();
    ensure_existing_directory(&absolute)?;
    Ok(absolute)
}

fn ensure_existing_directory(path: &Path) -> Result<()> {
    let metadata = fs::metadata(path).with_context(|| {
        format!(
            "directory '{}' does not exist or is inaccessible",
            path.display()
        )
    })?;
    if !metadata.is_dir() {
        bail!("'{}' is not a directory", path.display());
    }
    Ok(())
}

fn infer_name(path: &Path) -> Result<String> {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(str::to_owned)
        .context("cannot infer a name for this directory; provide --name")
}

fn expand_tilde(path: &Path) -> Result<PathBuf> {
    let Some(value) = path.to_str() else {
        bail!("the directory path is not valid UTF-8");
    };
    if value == "~" {
        return user_home().context("could not expand '~'; set HOME or USERPROFILE");
    }

    let remainder = value
        .strip_prefix("~/")
        .or_else(|| value.strip_prefix("~\\"));
    if let Some(remainder) = remainder {
        return Ok(user_home()
            .context("could not expand '~'; set HOME or USERPROFILE")?
            .join(remainder));
    }
    Ok(path.to_owned())
}

fn platform_config_dir() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        env::var_os("APPDATA").map(PathBuf::from)
    }

    #[cfg(target_os = "macos")]
    {
        user_home().map(|home| home.join("Library").join("Application Support"))
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        env::var_os("XDG_CONFIG_HOME")
            .filter(|value| !value.is_empty())
            .map(PathBuf::from)
            .or_else(|| user_home().map(|home| home.join(".config")))
    }
}

fn user_home() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        env::var_os("USERPROFILE")
            .or_else(|| env::var_os("HOME"))
            .map(PathBuf::from)
    }

    #[cfg(not(target_os = "windows"))]
    {
        env::var_os("HOME").map(PathBuf::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn validates_portable_keys() {
        for valid in ["BB", "my-work", "work_2", "work.home"] {
            validate_key(valid).unwrap();
        }
        for invalid in ["", "-work", "my work", "ação", "work&home"] {
            assert!(
                validate_key(invalid).is_err(),
                "{invalid} should be invalid"
            );
        }
    }

    #[test]
    fn saves_resolves_and_removes_case_insensitively() {
        let temp = tempdir().unwrap();
        let config = ConfigStore::new(temp.path().join("config/workplaces.json"));
        let target = temp.path().join("MyWork");
        fs::create_dir(&target).unwrap();

        let saved = config.save_inferred(Some(&target), Some("BB")).unwrap();
        assert_eq!(saved.0, "BB");
        assert_eq!(config.resolve("bb").unwrap(), target.clean());
        assert!(config.save_inferred(Some(&target), Some("bB")).is_err());

        let removed = config.remove("Bb").unwrap();
        assert_eq!(removed.0, "BB");
        assert!(config.resolve("BB").is_err());
    }

    #[test]
    fn infers_name_from_directory() {
        let temp = tempdir().unwrap();
        let config = ConfigStore::new(temp.path().join("config.json"));
        let target = temp.path().join("Project_1");
        fs::create_dir(&target).unwrap();

        let (name, _) = config.save_inferred(Some(&target), None).unwrap();
        assert_eq!(name, "Project_1");
    }

    #[test]
    fn rejects_files_and_missing_directories() {
        let temp = tempdir().unwrap();
        let config = ConfigStore::new(temp.path().join("config.json"));
        let file = temp.path().join("file.txt");
        fs::write(&file, "test").unwrap();

        assert!(config.save_inferred(Some(&file), Some("file")).is_err());
        assert!(config
            .save_inferred(Some(&temp.path().join("missing")), Some("missing"))
            .is_err());
    }

    #[test]
    fn reports_a_stale_saved_directory_without_deleting_it() {
        let temp = tempdir().unwrap();
        let config = ConfigStore::new(temp.path().join("config.json"));
        let target = temp.path().join("temporary");
        fs::create_dir(&target).unwrap();
        config.save_inferred(Some(&target), Some("stale")).unwrap();
        fs::remove_dir(&target).unwrap();

        let error = config.resolve("stale").unwrap_err().to_string();
        assert!(error.contains("points to"));
        assert!(config.load().unwrap().contains_key("stale"));
    }

    #[test]
    fn expands_the_home_directory() {
        let temp = tempdir().unwrap();
        let config = ConfigStore::new(temp.path().join("config.json"));
        let expected = user_home().unwrap().clean();

        config
            .save_inferred(Some(Path::new("~")), Some("home"))
            .unwrap();
        assert_eq!(config.resolve("home").unwrap(), expected);
    }

    #[test]
    fn rejects_ambiguous_manually_edited_configuration() {
        let temp = tempdir().unwrap();
        let path = temp.path().join("config.json");
        let value = temp.path().display().to_string().replace('\\', "\\\\");
        fs::write(&path, format!(r#"{{"BB":"{value}","bb":"{value}"}}"#)).unwrap();

        let config = ConfigStore::new(path);
        assert!(config.load().is_err());
    }

    #[test]
    fn failed_duplicate_does_not_change_configuration() {
        let temp = tempdir().unwrap();
        let path = temp.path().join("config.json");
        let config = ConfigStore::new(&path);
        let first = temp.path().join("first");
        let second = temp.path().join("second");
        fs::create_dir(&first).unwrap();
        fs::create_dir(&second).unwrap();
        config.save_inferred(Some(&first), Some("work")).unwrap();
        let before = fs::read(&path).unwrap();

        assert!(config.save_inferred(Some(&second), Some("WORK")).is_err());
        assert_eq!(fs::read(path).unwrap(), before);
    }

    #[test]
    fn concurrent_saves_do_not_lose_entries() {
        let temp = tempdir().unwrap();
        let config_path = temp.path().join("config.json");
        let mut threads = Vec::new();

        for index in 0..8 {
            let config_path = config_path.clone();
            let target = temp.path().join(format!("target_{index}"));
            fs::create_dir(&target).unwrap();
            threads.push(std::thread::spawn(move || {
                ConfigStore::new(config_path)
                    .save_inferred(Some(&target), Some(&format!("work_{index}")))
                    .unwrap();
            }));
        }

        for thread in threads {
            thread.join().unwrap();
        }
        assert_eq!(ConfigStore::new(config_path).load().unwrap().len(), 8);
    }
}
