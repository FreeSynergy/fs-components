//! An icon theme by the freedesktop Icon Theme Specification: find the file of an icon name.
//!
//! Pattern: Strategy (the lookup of the specification) — `index.theme` is read, the theme and what it `Inherits` are
//! searched in order (and `hicolor` last), in each directory the size is matched or the closest one taken. The
//! specification is the primary source; where this code does less, it says so below: only `svg` and `png` are looked
//! for (no `xpm`), `.icon` files with embedded text are not read, and the cache `icon-theme.cache` is not used.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// How a directory of a theme holds sizes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    Fixed,
    Scalable,
    Threshold,
}

#[derive(Clone, Debug)]
struct Dir {
    path: String,
    size: u32,
    scale: u32,
    kind: Kind,
    min: u32,
    max: u32,
    threshold: u32,
}

impl Dir {
    /// `DirectoryMatchesSize` of the specification.
    fn matches(&self, size: u32, scale: u32) -> bool {
        if self.scale != scale {
            return false;
        }
        match self.kind {
            Kind::Fixed => self.size == size,
            Kind::Scalable => (self.min..=self.max).contains(&size),
            Kind::Threshold => {
                self.size.saturating_sub(self.threshold) <= size
                    && size <= self.size + self.threshold
            }
        }
    }

    /// `DirectorySizeDistance` of the specification.
    fn distance(&self, size: u32, scale: u32) -> u32 {
        let want = i64::from(size) * i64::from(scale);
        let d = |v: u32| i64::from(v) * i64::from(self.scale);
        let out = match self.kind {
            Kind::Fixed => (d(self.size) - want).abs(),
            Kind::Scalable | Kind::Threshold => {
                let (lo, hi) = if self.kind == Kind::Scalable {
                    (d(self.min), d(self.max))
                } else {
                    (
                        d(self.size.saturating_sub(self.threshold)),
                        d(self.size + self.threshold),
                    )
                };
                if want < lo {
                    lo - want
                } else if want > hi {
                    want - hi
                } else {
                    0
                }
            }
        };
        u32::try_from(out).unwrap_or(u32::MAX)
    }
}

#[derive(Clone, Debug)]
struct Theme {
    name: String,
    dirs: Vec<Dir>,
    inherits: Vec<String>,
}

/// An icon theme with what it inherits, over the base directories where themes live.
#[derive(Clone, Debug)]
pub struct IconTheme {
    bases: Vec<PathBuf>,
    chain: Vec<Theme>,
}

/// The base directories of the specification: `$HOME/.icons`, `$XDG_DATA_HOME/icons` (default
/// `~/.local/share/icons`), `$XDG_DATA_DIRS/icons` (default `/usr/local/share:/usr/share`), `/usr/share/pixmaps`.
pub fn default_base_dirs() -> Vec<PathBuf> {
    let home = std::env::var_os("HOME").map(PathBuf::from);
    let mut out = Vec::new();
    if let Some(h) = &home {
        out.push(h.join(".icons"));
    }
    if let Some(d) = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| home.map(|h| h.join(".local/share")))
    {
        out.push(d.join("icons"));
    }
    let dirs =
        std::env::var("XDG_DATA_DIRS").unwrap_or_else(|_| "/usr/local/share:/usr/share".into());
    out.extend(
        dirs.split(':')
            .filter(|d| !d.is_empty())
            .map(|d| Path::new(d).join("icons")),
    );
    out.push(PathBuf::from("/usr/share/pixmaps"));
    out
}

/// The sections of an `index.theme`: section name to its keys.
fn parse(text: &str) -> HashMap<String, HashMap<String, String>> {
    let mut out: HashMap<String, HashMap<String, String>> = HashMap::new();
    let mut at = String::new();
    for line in text.lines().map(str::trim) {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(name) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
            at = name.to_string();
        } else if let Some((k, v)) = line.split_once('=') {
            out.entry(at.clone())
                .or_default()
                .insert(k.trim().to_string(), v.trim().to_string());
        }
    }
    out
}

fn load(bases: &[PathBuf], name: &str) -> Option<Theme> {
    let text = bases
        .iter()
        .find_map(|b| std::fs::read_to_string(b.join(name).join("index.theme")).ok())?;
    let sections = parse(&text);
    let head = sections.get("Icon Theme")?;
    let list = |k: &str| -> Vec<String> {
        head.get(k)
            .map(|v| {
                v.split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect()
            })
            .unwrap_or_default()
    };
    let number =
        |s: &HashMap<String, String>, k: &str| s.get(k).and_then(|v| v.parse::<u32>().ok());
    let dirs = list("Directories")
        .into_iter()
        .filter_map(|path| {
            let s = sections.get(&path)?;
            let size = number(s, "Size")?;
            let kind = match s.get("Type").map(String::as_str) {
                Some("Fixed") => Kind::Fixed,
                Some("Scalable") => Kind::Scalable,
                _ => Kind::Threshold,
            };
            Some(Dir {
                path,
                size,
                scale: number(s, "Scale").unwrap_or(1),
                kind,
                min: number(s, "MinSize").unwrap_or(size),
                max: number(s, "MaxSize").unwrap_or(size),
                threshold: number(s, "Threshold").unwrap_or(2),
            })
        })
        .collect();
    Some(Theme {
        name: name.to_string(),
        dirs,
        inherits: list("Inherits"),
    })
}

impl IconTheme {
    /// The theme `name` and the themes it inherits, found in `bases`; `hicolor` is searched last. `None` if the
    /// theme itself is not there.
    pub fn open(bases: &[PathBuf], name: &str) -> Option<Self> {
        let mut chain: Vec<Theme> = Vec::new();
        let mut queue = vec![name.to_string()];
        while let Some(next) = queue.first().cloned() {
            queue.remove(0);
            if chain.iter().any(|t| t.name == next) {
                continue;
            }
            match load(bases, &next) {
                Some(theme) => {
                    queue.extend(theme.inherits.iter().cloned());
                    chain.push(theme);
                }
                None if next == name => return None,
                None => {}
            }
        }
        if !chain.iter().any(|t| t.name == "hicolor") {
            chain.extend(load(bases, "hicolor"));
        }
        Some(Self {
            bases: bases.to_vec(),
            chain,
        })
    }

    /// The file of the icon `name` at `size` pixels (scale 1): the first theme of the chain that has it, in it the
    /// directory that matches the size, else the closest one. A name with dashes that is not found is tried again
    /// without its last part (`edit-copy-symbolic` finds `edit-copy`), as the specification says.
    pub fn lookup(&self, name: &str, size: u32) -> Option<PathBuf> {
        let mut at = name;
        loop {
            if let Some(found) = self.find(at, size) {
                return Some(found);
            }
            at = at.rsplit_once('-')?.0;
        }
    }

    fn find(&self, name: &str, size: u32) -> Option<PathBuf> {
        for theme in &self.chain {
            let existing = |dir: &Dir| {
                self.bases.iter().find_map(|b| {
                    ["svg", "png"]
                        .iter()
                        .map(|ext| {
                            b.join(&theme.name)
                                .join(&dir.path)
                                .join(format!("{name}.{ext}"))
                        })
                        .find(|p| p.is_file())
                })
            };
            if let Some(found) = theme
                .dirs
                .iter()
                .filter(|d| d.matches(size, 1))
                .find_map(existing)
            {
                return Some(found);
            }
            let mut closest: Option<(u32, PathBuf)> = None;
            for dir in &theme.dirs {
                if let Some(path) = existing(dir) {
                    let distance = dir.distance(size, 1);
                    if closest.as_ref().is_none_or(|(d, _)| distance < *d) {
                        closest = Some((distance, path));
                    }
                }
            }
            if let Some((_, path)) = closest {
                return Some(path);
            }
        }
        None
    }
}
