//! IN1's finite bootstrap/preparation/first-declaration admissions.
use super::*;
use crate::{depth_record::acquisition, support_matrix};
use std::collections::BTreeSet;

pub(crate) struct Checkpoint {
    registry: AgentRegistry,
    view: view::View,
    root: PathBuf,
    head: String,
    pub(crate) publication_changed: bool,
}
impl Checkpoint {
    /// Only a fully checked checkpoint can temporarily supply its candidate registry to read-only
    /// reporting checks. This does not alter the route's captured registry or scratch binding.
    pub(crate) fn with_registry<T>(
        &self,
        root: &Path,
        check: impl FnOnce() -> Result<T, String>,
    ) -> Result<T, String> {
        self.recheck(root)?;
        let root = self.root.clone();
        CANDIDATE.with(|slot| {
            let mut slot = slot.borrow_mut();
            if slot.is_some() {
                return Err("bootstrap registry already bound".to_string());
            }
            *slot = Some(CandidateBinding {
                root,
                registry: self.registry.clone(),
                view: None,
            });
            Ok(())
        })?;
        let _guard = CandidateRegistry(std::marker::PhantomData);
        let result = check();
        self.recheck(&self.root)?;
        result
    }
    /// Read-only filesystem engines run against committed candidate bytes, with their registry
    /// bound to that private checkout. The live checkout and materialization are both verified.
    pub(crate) fn with_view<T>(
        &self,
        root: &Path,
        check: impl FnOnce(&Path) -> Result<T, String>,
    ) -> Result<T, String> {
        self.recheck(root)?;
        self.view.verify(self.view.root())?;
        CANDIDATE.with(|slot| {
            let mut slot = slot.borrow_mut();
            if slot.is_some() {
                return Err("bootstrap registry already bound".to_string());
            }
            *slot = Some(CandidateBinding {
                root: self.view.root().to_owned(),
                registry: self.registry.clone(),
                view: None,
            });
            Ok(())
        })?;
        let _guard = CandidateRegistry(std::marker::PhantomData);
        let result = check(self.view.root());
        self.view.verify(self.view.root())?;
        self.recheck(root)?;
        result
    }
    fn recheck(&self, root: &Path) -> Result<(), String> {
        if root.canonicalize().map_err(|e| e.to_string())? != self.root
            || super::super::git(root, &["rev-parse", "HEAD"])? != self.head
            || !super::super::git(root, &["status", "--porcelain=v1", "--untracked-files=all"])?
                .is_empty()
        {
            return Err("bootstrap checkpoint inputs changed".into());
        }
        self.view.verify(root)
    }
}

pub(super) fn check(
    root: &Path,
    parent: &str,
    before: Source,
    after: Source,
    head: &str,
    adapter: Adapter,
) -> Result<Checkpoint, String> {
    let raw = before.read(REGISTRY_RELATIVE_PATH)?;
    let old = AgentRegistry::parse_unenforced(&raw).map_err(|e| e.to_string())?;
    let new_raw = after.read(REGISTRY_RELATIVE_PATH)?;
    let registry = AgentRegistry::parse_unenforced(&new_raw).map_err(|e| e.to_string())?;
    adapter.registry_delta(&old, &registry)?;
    let id = &adapter.scaffold.agent_id;
    let creation = old.find(id).is_none() && registry.find(id).is_some();
    let declaration = registry.find(id).is_some_and(|e| e.support_depth.is_some());
    if !creation && !declaration && new_raw != raw {
        return Err("preparation must preserve exact scaffold registry bytes".into());
    }
    if declaration {
        // Only the declared table is maintainer-owned here. Preserve every other byte from P.
        let mut expected: toml_edit::DocumentMut = raw
            .parse()
            .map_err(|e: toml_edit::TomlError| e.to_string())?;
        let table =
            toml_edit::ser::to_document(registry.find(id).unwrap().support_depth.as_ref().unwrap())
                .map_err(|e| e.to_string())?;
        expected["agents"]
            .as_array_of_tables_mut()
            .ok_or("invalid parent registry")?
            .iter_mut()
            .find(|e| e["agent_id"].as_str() == Some(id.as_str()))
            .ok_or("missing parent scaffold")?
            .insert(
                "support_depth",
                toml_edit::Item::Table(table.as_table().clone()),
            );
        if new_raw != expected.to_string() {
            return Err(
                "declaration changed bytes outside its rendered support_depth table".into(),
            );
        }
    }
    let (approval_bytes, approval) = adapter.read_approval(&before)?;
    if after.optional_bytes(&adapter.approval_path())?.as_ref() != Some(&approval_bytes)
        || before.executable(&adapter.approval_path())?
        || after.executable(&adapter.approval_path())?
    {
        return Err("bootstrap checkpoint changed approval identity".into());
    }
    inputs::record_free(&before, &adapter, old.find(id).is_none())?;
    inputs::record_free(&after, &adapter, registry.find(id).is_none())?;
    let current_path = format!("{}/current.json", adapter.scaffold.manifest_root);
    if !creation && before.optional_bytes(&current_path)? != after.optional_bytes(&current_path)? {
        return Err(
            "preparation/declaration must preserve ordinary bootstrap current bytes".into(),
        );
    }
    let state_path = adapter.state_path();
    if !creation && before.optional_bytes(&state_path)? != after.optional_bytes(&state_path)? {
        return Err("preparation/declaration changed scaffold lifecycle".into());
    }
    if creation && after.optional(&state_path)?.is_none() {
        return Err("ON1 creation requires its ordinary lifecycle scaffold".into());
    }
    let statement_path = acquisition::statement_path(
        &adapter.scaffold.scaffold.onboarding_pack_prefix,
        adapter.version,
    );
    let acquired: BTreeSet<_> = adapter.acquired_paths()?.into_iter().collect();
    let publication: BTreeSet<_> = [
        support_matrix::JSON_OUTPUT_PATH,
        support_matrix::MARKDOWN_OUTPUT_PATH,
        support_matrix::AGENT_API_RUNTIME_SUPPORT_DATA_OUTPUT_PATH,
        crate::capability_matrix::DEFAULT_OUT_PATH,
    ]
    .into_iter()
    .map(str::to_owned)
    .collect();
    let root_path = &adapter.scaffold.manifest_root;
    let mut allowed: BTreeSet<String> = BTreeSet::new();
    if creation {
        allowed.extend(creation::validate(root, parent, &before, &after, &adapter)?);
    } else if declaration {
        allowed.extend([REGISTRY_RELATIVE_PATH.to_owned(), statement_path.clone()]);
    } else {
        allowed.extend(acquired);
        allowed.extend(raw_help::paths(&adapter));
        for name in [
            "RULES.json",
            "SCHEMA.json",
            "VERSION_METADATA_SCHEMA.json",
            "VALIDATOR_SPEC.md",
            "latest_validated.txt",
            "min_supported.txt",
        ] {
            allowed.insert(format!("{root_path}/{name}"));
        }
        for target in &adapter.scaffold.canonical_targets {
            for pointer in ["latest_validated", "latest_supported"] {
                allowed.insert(format!("{root_path}/pointers/{pointer}/{target}.txt"));
            }
        }
        allowed.insert(format!("{root_path}/versions/{}.json", adapter.version));
    }
    if !creation {
        allowed.extend(publication.iter().cloned());
    }
    let changed = super::super::git(
        root,
        &[
            "diff",
            "--no-renames",
            "--name-only",
            "-z",
            &before.tree()?,
            &after.tree()?,
        ],
    )?;
    let mut publication_changed = false;
    for path in changed.split('\0').filter(|p| !p.is_empty()) {
        let old_bytes = before.optional_bytes(path)?;
        let new_bytes = after.optional_bytes(path)?;
        let mode_changed = old_bytes.is_some()
            && new_bytes.is_some()
            && before.executable(path)? != after.executable(path)?;
        if old_bytes == new_bytes && !mode_changed {
            continue;
        }
        if !allowed.contains(path) || new_bytes.is_none() || after.executable(path)? {
            return Err(format!(
                "bootstrap checkpoint has an unauthorized effect: {path}"
            ));
        }
        publication_changed |= publication.contains(path);
    }
    // The candidate's acquisition must be truthful even before metadata is materialized.
    if !creation {
        inputs::acquisition(&after, &adapter, declaration)?;
        for path in raw_help::paths(&adapter) {
            if after.optional_bytes(&path)?.is_some() && after.executable(&path)? {
                return Err(format!("executable bootstrap raw help: {path}"));
            }
        }
    }
    let view = view::View::capture(root, head)?;
    inputs::metadata(view.root(), &before, &after, &adapter)?;
    if !creation {
        inputs::governing(&after, &adapter, declaration)?;
    }
    if declaration {
        // A declaration follows an integrated, truthfully snapshotted preparation checkpoint.
        let metadata = format!("{root_path}/versions/{}.json", adapter.version);
        if before.optional(&metadata)?.is_none() {
            return Err("declaration requires prior snapshotted metadata".into());
        }
        let mut planner = GitInputs::new(view.root())?;
        if planner.required(REGISTRY_RELATIVE_PATH)? != new_raw.as_bytes() {
            return Err("declaration planner registry differs from the captured candidate".into());
        }
        let plan = crate::support_depth::FreezePlan::onboarding(&mut planner, &approval, None)?;
        let obligations = plan
            .record
            .obligations
            .ok_or("declaration requires actual acquired inputs")?;
        let statement = acquisition::AcquisitionStatement::parse(
            &after.read(&statement_path)?,
            id,
            &format!("{}.toml", adapter.version),
        )?;
        if let Some(why) = statement.mismatch(&obligations.acquired_inputs) {
            return Err(why);
        }
        if !obligations.classification_required.is_empty() {
            return Err("declaration requires reviewed CORE classification rows".into());
        }
        planner.verify()?;
    }
    view.verify(view.root())?;
    view.verify(root)?;
    Ok(Checkpoint {
        registry,
        view,
        root: root.canonicalize().map_err(|e| e.to_string())?,
        head: head.to_owned(),
        publication_changed: publication_changed || declaration,
    })
}
