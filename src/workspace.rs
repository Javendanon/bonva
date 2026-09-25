use crate::data;
use anyhow::{Context, Result, bail, ensure};
use fs2::FileExt;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::{
    collections::BTreeMap,
    fs::{self, File, OpenOptions},
    path::{Path, PathBuf},
    time::{Instant, SystemTime, UNIX_EPOCH},
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entry {
    pub sha256: String,
    pub mode: u32,
}
pub type Inventory = BTreeMap<String, Entry>;
pub fn inventory(root: &Path) -> Result<Inventory> {
    let mut out = BTreeMap::new();
    walk(root, root, &mut out)?;
    Ok(out)
}
fn walk(root: &Path, dir: &Path, out: &mut Inventory) -> Result<()> {
    for item in fs::read_dir(dir)? {
        let item = item?;
        let path = item.path();
        let name = item.file_name();
        let name = name.to_str().context("Non-UTF8 path unsupported")?;
        if ["_build", ".elixir_ls", ".DS_Store"].contains(&name) || (dir == root && name == ".git")
        {
            continue;
        }
        let metadata = fs::symlink_metadata(&path)?;
        ensure!(
            !metadata.file_type().is_symlink(),
            "Unsupported symlink: {}",
            path.display()
        );
        if metadata.is_dir() {
            walk(root, &path, out)?;
        } else if metadata.is_file() {
            #[cfg(unix)]
            let mode = metadata.permissions().mode() & 0o777;
            #[cfg(not(unix))]
            let mode = 0;
            out.insert(
                path.strip_prefix(root)?
                    .to_str()
                    .context("Path encoding")?
                    .to_string(),
                Entry {
                    sha256: data::digest(&path)?,
                    mode,
                },
            );
        } else {
            bail!("Unsupported special file: {}", path.display());
        }
    }
    Ok(())
}
pub fn hashes(inv: &Inventory) -> Value {
    json!(
        inv.iter()
            .map(|(p, e)| (p.clone(), e.sha256.clone()))
            .collect::<BTreeMap<_, _>>()
    )
}
pub fn input_inventory(inv: &Inventory, generated: &[String]) -> Inventory {
    inv.iter()
        .filter(|(p, _)| !generated.iter().any(|d| p.starts_with(&format!("{d}/"))))
        .map(|(p, e)| (p.clone(), e.clone()))
        .collect()
}
pub fn environment() -> BTreeMap<String, String> {
    let mut env: BTreeMap<_, _> = std::env::vars().collect();
    for k in ["MIX_BUILD_PATH", "MIX_DEPS_PATH", "MIX_EXS", "PHX_SERVER"] {
        env.remove(k);
    }
    env.insert("MIX_ENV".into(), "test".into());
    env.insert("NO_COLOR".into(), "1".into());
    env.insert("TERM".into(), "dumb".into());
    env
}
pub fn environment_hash(env: &BTreeMap<String, String>) -> String {
    let stable: BTreeMap<_, _> = env
        .iter()
        .filter(|(k, _)| {
            !matches!(
                k.as_str(),
                "PWD" | "OLDPWD" | "SHLVL" | "_" | "QUALITY_TEST_RESULT"
            )
        })
        .collect();
    data::identity(&json!(stable))
}

pub struct Workspace {
    pub project: PathBuf,
    pub logs: PathBuf,
    pub info: Value,
    lock: File,
    lane: PathBuf,
}
impl Workspace {
    pub fn open(
        original: &Path,
        cache: &Path,
        inputs: &Inventory,
        fingerprint: &str,
    ) -> Result<Self> {
        let start = Instant::now();
        fs::create_dir_all(cache)?;
        let cache = cache.canonicalize()?;
        ensure!(
            !cache.starts_with(original) && !original.starts_with(&cache),
            "Cache and original must be disjoint"
        );
        let namespace = cache.join(data::hash(
            original.to_str().context("Project path")?.as_bytes(),
        ));
        fs::create_dir_all(&namespace)?;
        let lock = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(namespace.join("lock"))?;
        lock.try_lock_exclusive()
            .context("Workspace is in use by another evaluator")?;
        let lane = namespace.join(fingerprint);
        let reused = lane.join("complete.json").is_file();
        fs::create_dir_all(&lane)?;
        let marker = lane.join("owner.json");
        let expected = json!({"owner":"agent-quality","version":1,"original":original,"fingerprint":fingerprint});
        if marker.exists() {
            ensure!(
                data::read(&marker)? == expected,
                "Invalid workspace ownership"
            );
        } else {
            ensure!(!lane.join("project").exists(), "Unowned workspace");
            fs::write(&marker, serde_json::to_vec(&expected)?)?;
        }
        let project = lane.join("project");
        fs::create_dir_all(&project)?;
        // A run interrupted before its completion marker may have left untrusted build output.
        if !reused && project.join("_build").exists() {
            fs::remove_dir_all(project.join("_build"))?;
        }
        if lane.join("complete.json").exists() {
            fs::remove_file(lane.join("complete.json"))?;
        }
        let actual = inventory(&project)?;
        let mut removed = 0;
        let mut copied = 0;
        for p in actual.keys().filter(|p| !inputs.contains_key(*p)) {
            fs::remove_file(project.join(p))?;
            removed += 1;
        }
        prune_empty(&project)?;
        for (p, entry) in inputs {
            if actual.get(p) != Some(entry) {
                let destination = project.join(p);
                if destination.is_dir() {
                    fs::remove_dir_all(&destination)?;
                }
                fs::create_dir_all(destination.parent().unwrap())?;
                fs::copy(original.join(p), &destination)?;
                copied += 1;
            }
        }
        ensure!(
            inventory(&project)? == *inputs,
            "Source changed while synchronizing workspace"
        );
        let nonce = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
        let logs = lane
            .join("runs")
            .join(format!("{}-{nonce}", std::process::id()));
        fs::create_dir_all(&logs)?;
        Ok(Self {
            project,
            logs,
            info: json!({"reused":reused,"copied_files":copied,"removed_files":removed,"fingerprint":fingerprint,"prepare_ms":start.elapsed().as_secs_f64()*1000.0}),
            lock,
            lane,
        })
    }
    pub fn complete(&self, clean: bool) -> Result<()> {
        if clean {
            fs::write(self.lane.join("complete.json"), b"{}")?;
        }
        Ok(())
    }
}
impl Drop for Workspace {
    fn drop(&mut self) {
        let _ = FileExt::unlock(&self.lock);
    }
}
fn prune_empty(dir: &Path) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let p = entry?.path();
        if p.file_name().is_some_and(|n| n == "_build") {
            continue;
        }
        if p.is_dir() {
            prune_empty(&p)?;
            if fs::read_dir(&p)?.next().is_none() {
                fs::remove_dir(&p)?;
            }
        }
    }
    Ok(())
}

pub fn fingerprint(
    inputs: &Inventory,
    env: &BTreeMap<String, String>,
    runtime: &str,
    adapter: &Value,
    execution: &Value,
) -> String {
    let build_inputs: BTreeMap<_, _> = inputs
        .iter()
        .filter(|(p, _)| {
            p.starts_with("deps/")
                || p.starts_with("config/")
                || matches!(p.as_str(), "mix.exs" | "mix.lock")
        })
        .collect();
    data::identity(
        &json!({"workspace_version":1,"build_inputs":build_inputs,"environment":environment_hash(env),"runtime":runtime,"adapter":adapter,"execution":execution}),
    )
}
