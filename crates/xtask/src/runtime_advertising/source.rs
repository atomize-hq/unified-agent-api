use quote::{quote, ToTokens};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs,
    path::{Component, Path, PathBuf},
    process::Command,
};

/// All reads in a baseline view refer to one resolved commit, never the candidate filesystem.
pub struct Source {
    root: PathBuf,
    commit: Option<String>,
    proposed: BTreeMap<String, String>,
}

impl Source {
    pub fn workspace(root: &Path) -> Result<Self, String> {
        Ok(Self {
            root: root.canonicalize().map_err(|e| e.to_string())?,
            commit: None,
            proposed: BTreeMap::new(),
        })
    }

    /// A bounded read-only candidate; it never materializes proposed files in the workspace.
    pub(crate) fn proposed(root: &Path, files: BTreeMap<String, String>) -> Result<Self, String> {
        let mut source = Self::workspace(root)?;
        for path in files.keys() {
            source.optional(path)?;
        }
        source.proposed = files;
        Ok(source)
    }

    pub fn tip(root: &Path, revision: &str) -> Result<Self, String> {
        let mut source = Self::workspace(root)?;
        let top = source.git(&["rev-parse", "--show-toplevel"])?;
        if Path::new(top.trim())
            .canonicalize()
            .map_err(|e| e.to_string())?
            != source.root
        {
            return Err("advertising baseline requires the workspace's own Git repository".into());
        }
        source.commit = Some(
            source
                .git(&[
                    "rev-parse",
                    "--verify",
                    "--end-of-options",
                    &format!("{revision}^{{commit}}"),
                ])?
                .trim()
                .into(),
        );
        Ok(source)
    }

    pub fn read(&self, path: &str) -> Result<String, String> {
        self.optional(path)?
            .ok_or_else(|| format!("missing advertising input: {path}"))
    }

    pub fn optional(&self, path: &str) -> Result<Option<String>, String> {
        self.optional_bytes(path)?
            .map(|bytes| String::from_utf8(bytes).map_err(|e| e.to_string()))
            .transpose()
    }

    /// Raw regular-file bytes from the same pinned view; binary fixtures are not lossy UTF-8.
    pub(crate) fn optional_bytes(&self, path: &str) -> Result<Option<Vec<u8>>, String> {
        if path.is_empty()
            || path.split('/').any(|part| {
                part.is_empty()
                    || part == "."
                    || part == ".."
                    || part == ".git"
                    || part.contains('\\')
            })
            || Path::new(path)
                .components()
                .any(|p| !matches!(p, Component::Normal(_)))
        {
            return Err(format!("invalid source path: {path}"));
        }
        if let Some(raw) = self.proposed.get(path) {
            return Ok(Some(raw.as_bytes().to_vec()));
        }
        if let Some(commit) = &self.commit {
            // ls-tree does not follow a symlink ancestor. Distinguish that unsupported binding
            // from an absent optional file (notably .cargo/config* and build.rs).
            let mut ancestor = PathBuf::new();
            let components: Vec<_> = Path::new(path).components().collect();
            for component in &components[..components.len() - 1] {
                ancestor.push(component);
                let parent = self.git(&[
                    "ls-tree",
                    "-z",
                    "--full-tree",
                    commit,
                    "--",
                    ancestor.to_str().ok_or("non-UTF8 ancestor")?,
                ])?;
                if parent.is_empty() {
                    return Ok(None);
                }
                if !parent.starts_with("040000 tree ") {
                    return Err(format!("{path}: non-directory Git ancestor"));
                }
            }
            let rows = self.git(&["ls-tree", "-z", "--full-tree", commit, "--", path])?;
            if rows.is_empty() {
                return Ok(None);
            }
            let row = rows
                .strip_suffix('\0')
                .ok_or("malformed Git source entry")?;
            let (meta, actual) = row.split_once('\t').ok_or("invalid Git tree row")?;
            let fields: Vec<_> = meta.split_whitespace().collect();
            if actual != path
                || fields.len() != 3
                || !matches!(fields[0], "100644" | "100755")
                || fields[1] != "blob"
            {
                return Err(format!("{path}: expected a regular source blob"));
            }
            return self.git_bytes(&["cat-file", "blob", fields[2]]).map(Some);
        }
        let mut cursor = self.root.clone();
        for component in Path::new(path).components() {
            cursor.push(component);
            match fs::symlink_metadata(&cursor) {
                Ok(meta) if meta.file_type().is_symlink() => {
                    return Err(format!("{path}: source symlink is unsupported"))
                }
                Ok(_) => {}
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
                Err(e) => return Err(format!("{path}: {e}")),
            }
        }
        if !fs::symlink_metadata(&cursor)
            .map_err(|e| e.to_string())?
            .is_file()
        {
            return Err(format!("{path}: expected a regular source file"));
        }
        fs::read(&cursor)
            .map(Some)
            .map_err(|e| format!("{path}: {e}"))
    }

    /// Enumerate this revision without following directory symlinks or treating read errors as absence.
    pub fn paths(&self, prefix: &str) -> Result<Vec<String>, String> {
        self.optional(&format!("{prefix}/.source-directory-probe"))?;
        if let Some(commit) = &self.commit {
            let rows = self.git(&[
                "ls-tree",
                "-r",
                "-t",
                "-z",
                "--full-tree",
                commit,
                "--",
                prefix,
            ])?;
            return rows
                .split('\0')
                .filter(|r| !r.is_empty())
                .map(|row| {
                    row.split_once('\t')
                        .map(|(_, path)| path.to_owned())
                        .ok_or_else(|| "invalid source tree entry".into())
                })
                .collect();
        }
        fn walk(root: &Path, path: &Path, result: &mut Vec<String>) -> Result<(), String> {
            let entries = match fs::read_dir(path) {
                Ok(entries) => entries,
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
                Err(e) => return Err(format!("{}: {e}", path.display())),
            };
            for entry in entries {
                let entry = entry.map_err(|e| e.to_string())?;
                let kind = entry.file_type().map_err(|e| e.to_string())?;
                if kind.is_symlink() {
                    return Err(format!(
                        "{}: source symlink is unsupported",
                        entry.path().display()
                    ));
                }
                if !kind.is_dir() && !kind.is_file() {
                    return Err(format!(
                        "{}: unsupported source file type",
                        entry.path().display()
                    ));
                }
                let path = entry.path();
                result.push(
                    path.strip_prefix(root)
                        .map_err(|e| e.to_string())?
                        .to_str()
                        .ok_or("non-UTF8 source path")?
                        .replace('\\', "/"),
                );
                if kind.is_dir() {
                    walk(root, &path, result)?;
                }
            }
            Ok(())
        }
        let mut result = Vec::new();
        walk(&self.root, &self.root.join(prefix), &mut result)?;
        for path in self
            .proposed
            .keys()
            .filter(|p| p.starts_with(&format!("{prefix}/")))
        {
            if !result.contains(path) {
                result.push(path.clone());
            }
        }
        result.sort();
        Ok(result)
    }

    pub(crate) fn tree(&self) -> Result<String, String> {
        let commit = self.commit.as_ref().ok_or("expected an immutable source")?;
        Ok(self
            .git(&["rev-parse", &format!("{commit}^{{tree}}")])?
            .trim()
            .into())
    }

    pub(crate) fn committed_files(&self, prefix: &str) -> Result<Vec<String>, String> {
        self.optional_bytes(&format!("{prefix}/.source-directory-probe"))?;
        let commit = self.commit.as_ref().ok_or("expected an immutable source")?;
        self.git(&["ls-tree", "-r", "-z", "--full-tree", commit, "--", prefix])?
            .split('\0')
            .filter(|r| !r.is_empty())
            .map(|row| {
                let (meta, path) = row.split_once('\t').ok_or("invalid Git tree row")?;
                if !(meta.starts_with("100644 blob ") || meta.starts_with("100755 blob ")) {
                    return Err(format!("{path}: unsupported scoped source entry"));
                }
                Ok(path.to_owned())
            })
            .collect()
    }

    fn git(&self, args: &[&str]) -> Result<String, String> {
        String::from_utf8(self.git_bytes(args)?).map_err(|e| e.to_string())
    }

    fn git_bytes(&self, args: &[&str]) -> Result<Vec<u8>, String> {
        let out = Command::new("git")
            .arg("--no-replace-objects")
            .arg("-C")
            .arg(&self.root)
            .arg("--literal-pathspecs")
            .args(args)
            .output()
            .map_err(|e| e.to_string())?;
        if !out.status.success() {
            return Err(format!(
                "git {}: {}",
                args[0],
                String::from_utf8_lossy(&out.stderr)
            ));
        }
        Ok(out.stdout)
    }
}

/// Parse real Rust, then remove documentation attributes only. This is not a text search.
pub(crate) fn normalized(raw: &str) -> Result<String, String> {
    let file = syn::parse_file(raw).map_err(|e| format!("unsupported Rust source: {e}"))?;
    fn strip(stream: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
        use proc_macro2::{Delimiter, Group, TokenTree};
        let tokens: Vec<_> = stream.into_iter().collect();
        let mut result = proc_macro2::TokenStream::new();
        let mut i = 0;
        while i < tokens.len() {
            if matches!(&tokens[i],TokenTree::Punct(p) if p.as_char()=='#') {
                let mut j = i + 1;
                if matches!(tokens.get(j),Some(TokenTree::Punct(p)) if p.as_char()=='!') {
                    j += 1;
                }
                if let Some(TokenTree::Group(g)) = tokens.get(j) {
                    if g.delimiter() == Delimiter::Bracket
                        && matches!(g.stream().into_iter().next(),Some(TokenTree::Ident(id)) if id=="doc")
                    {
                        i = j + 1;
                        continue;
                    }
                }
            }
            let token = match &tokens[i] {
                TokenTree::Group(g) => {
                    TokenTree::Group(Group::new(g.delimiter(), strip(g.stream())))
                }
                t => t.clone(),
            };
            result.extend([token]);
            i += 1;
        }
        result
    }
    Ok(strip(file.into_token_stream()).to_string())
}

pub(crate) fn same<T: ToTokens>(item: &T, expected: &str, label: &str) -> Result<(), String> {
    let raw = quote!(#item).to_string();
    if normalized(&raw)? != normalized(expected)? {
        return Err(format!("{label}: unsupported source form"));
    }
    Ok(())
}

pub(crate) fn fingerprint(raw: &str) -> Result<String, String> {
    Ok(hex::encode(Sha256::digest(normalized(raw)?.as_bytes())))
}
