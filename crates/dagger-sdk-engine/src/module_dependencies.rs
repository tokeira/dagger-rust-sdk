//! A module's dependency list, written from the clients recorded for its scope.
//!
//! The engine records a client for a module's scope in the workspace configuration
//! and hands the complete list to generation. This module makes that list the
//! module's `[[dependencies]]` in `dagger-module.toml`, by the rules of Dagger's
//! manifest helper (`withLegacyRuntimeDependencies`, `prepareOutput` and
//! `tomlContents` in `main.dang` of github.com/dagger/sdk-helpers at v1.0.12):
//!
//! - a client is written as a table with its `name` and `source`, without
//!   indentation and without a pin;
//! - a dependency already in the manifest with the same source is kept for that
//!   client, with its name and other settings, and loses its pin;
//! - every other dependency is removed;
//! - the tables are ordered by the clients' names and follow `[runtime]`.
//!
//! Unlike the helper, which renders the whole manifest again from its own model,
//! everything outside the dependency tables is kept as it is written.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;

use serde::{Deserialize, Serialize};
use toml_edit::{ArrayOfTables, DocumentMut, Item, Key, Table, Value, value};

use crate::diagnostic::{EngineDiagnostic, EngineDiagnosticCode};
use crate::{
    ExecutionResult, ExecutionResultKind, FormatVersion, OperationRoot, RelativeOperationPath,
};

/// File name of a module's configuration, beneath the module root.
pub const MODULE_MANIFEST: &str = "dagger-module.toml";

pub(crate) const LEGACY_MODULE_MANIFEST: &str = "dagger.json";
const DEPENDENCIES: &str = "dependencies";
const COORDINATE: &str = "module-dependencies";

/// Complete input to rewriting one module's dependency list.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModuleDependenciesRequest {
    /// Wire-format revision.
    pub format_version: FormatVersion,
    /// The workspace root beneath the operation root. A local client's path is
    /// relative to it.
    pub workspace_root: RelativeOperationPath,
    /// The module's directory, at or beneath the workspace root.
    pub module_root: RelativeOperationPath,
    /// Every client recorded for the module's scope. The list is complete: a
    /// dependency that matches none of them is removed.
    pub clients: Vec<ModuleClient>,
}

/// One client of a module's scope, as the adapter resolved it.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum ModuleClient {
    /// A module in the workspace, written as a path relative to the module.
    Local {
        /// The target module's original name.
        name: String,
        /// The target's source root, as a path from the workspace root.
        source_root_subpath: String,
    },
    /// A module at a Git reference, written as that reference.
    Git {
        /// The target module's original name.
        name: String,
        /// The reference, as the engine spells it.
        source: String,
    },
}

impl ModuleClient {
    fn name(&self) -> &str {
        match self {
            Self::Local { name, .. } | Self::Git { name, .. } => name,
        }
    }
}

/// Rewrites the dependency list of the module at the request's root.
///
/// The result names the manifest when its bytes changed and no path when they did
/// not. A refusal writes nothing.
pub fn execute_module_dependencies(
    root: &OperationRoot,
    request: &ModuleDependenciesRequest,
) -> Result<ExecutionResult, EngineDiagnostic> {
    let module_directory = module_directory(&request.workspace_root, &request.module_root)?;
    let manifest_path = join(&request.module_root, MODULE_MANIFEST)?;
    let mut touched_paths = BTreeSet::new();
    if root.exists(&manifest_path) {
        let manifest = String::from_utf8(root.read(&manifest_path)?).map_err(|_| {
            EngineDiagnostic::new(
                EngineDiagnosticCode::OperationInputInvalid,
                Some(manifest_path.as_str()),
                "module configuration is not UTF-8",
            )
        })?;
        let planned = plan_module_dependencies(&manifest, &module_directory, &request.clients)?;
        if planned != manifest {
            let parent = root.ensure_parent(&manifest_path)?;
            fs::write(parent.join(MODULE_MANIFEST), planned).map_err(|_| {
                EngineDiagnostic::new(
                    EngineDiagnosticCode::PublicationFailed,
                    Some(manifest_path.as_str()),
                    "module configuration could not be written",
                )
            })?;
            touched_paths.insert(manifest_path);
        }
    } else {
        refuse_legacy_manifest(root, request, &module_directory)?;
    }
    Ok(ExecutionResult {
        format_version: FormatVersion,
        kind: ExecutionResultKind::ModuleDependencies,
        output_root: request.module_root.clone(),
        touched_paths,
        removed_paths: BTreeSet::new(),
        operation_manifest: None,
        vcs_generated: BTreeSet::new(),
        vcs_ignored: BTreeSet::new(),
        client_plan: None,
    })
}

/// A module configured only by `dagger.json` is left alone while its dependency
/// list already is its client list, which is when both are empty. Any other state
/// needs an edit this SDK does not make to that file, so the refusal advises the
/// migration that Dagger's guide describes. The module is named by its path from the
/// workspace root, which `dagger module migrate` reads wherever it runs.
fn refuse_legacy_manifest(
    root: &OperationRoot,
    request: &ModuleDependenciesRequest,
    module_directory: &str,
) -> Result<(), EngineDiagnostic> {
    let legacy_path = join(&request.module_root, LEGACY_MODULE_MANIFEST)?;
    if !root.exists(&legacy_path) {
        return Err(EngineDiagnostic::new(
            EngineDiagnosticCode::OperationInputInvalid,
            Some(request.module_root.as_str()),
            format!("the module has no {MODULE_MANIFEST}"),
        ));
    }
    let legacy: serde_json::Value =
        serde_json::from_slice(&root.read(&legacy_path)?).map_err(|_| {
            EngineDiagnostic::new(
                EngineDiagnosticCode::OperationInputInvalid,
                Some(legacy_path.as_str()),
                "module configuration is not JSON",
            )
        })?;
    let has_dependencies = legacy
        .get(DEPENDENCIES)
        .and_then(serde_json::Value::as_array)
        .is_some_and(|dependencies| !dependencies.is_empty());
    if request.clients.is_empty() && !has_dependencies {
        return Ok(());
    }
    let module = match module_directory {
        "." => "/".to_owned(),
        directory => format!("/{directory}"),
    };
    Err(legacy_manifest_refusal(
        &legacy_path,
        &format!("the module is configured by {LEGACY_MODULE_MANIFEST}"),
        &module,
        "`dagger generate`",
    ))
}

/// The refusal of a module configured only by `dagger.json` whose dependencies
/// generation would write. `configured` says which module that is, `module` is its
/// directory as `dagger module migrate` takes it, and `then` is what to run after
/// the migration.
pub(crate) fn legacy_manifest_refusal(
    legacy_path: &RelativeOperationPath,
    configured: &str,
    module: &str,
    then: &str,
) -> EngineDiagnostic {
    EngineDiagnostic::new(
        EngineDiagnosticCode::OperationInputInvalid,
        Some(legacy_path.as_str()),
        format!(
            "{configured}, where the Rust SDK does not write dependencies; run `dagger module \
             migrate {module}`, which moves its configuration to {MODULE_MANIFEST} and records \
             its dependencies as clients, then {then}; a coding agent passes `-y` to apply \
             each, or `--no-apply` to preview the migration"
        ),
    )
}

/// Returns a module configuration whose dependencies are exactly the clients.
///
/// `module_directory` is the module's directory as a path from the workspace root,
/// `.` for the root itself. The result equals `manifest` byte for byte when the
/// dependency tables already have the form this function writes.
pub fn plan_module_dependencies(
    manifest: &str,
    module_directory: &str,
    clients: &[ModuleClient],
) -> Result<String, EngineDiagnostic> {
    let module_directory = workspace_components(module_directory)?;
    let wanted = wanted_dependencies(&module_directory, clients)?;
    let mut document = manifest.parse::<DocumentMut>().map_err(|_| {
        refusal(format!(
            "{MODULE_MANIFEST} is not valid TOML; its dependencies cannot be written"
        ))
    })?;

    let anchor = anchor_position(&document);
    let mut existing = existing_dependencies(&mut document)?
        .into_iter()
        .map(Some)
        .collect::<Vec<_>>();
    let mut tables = Vec::with_capacity(wanted.len());
    for dependency in &wanted {
        // The helper compares sources alone (`sameDependencySource`), so a
        // dependency renamed by hand is still recognised.
        let adopted = existing
            .iter_mut()
            .find(|candidate| {
                candidate.as_ref().is_some_and(|table| {
                    string_field(table, "source").is_some_and(|source| {
                        normalize_path(source) == normalize_path(&dependency.source)
                    })
                })
            })
            .and_then(Option::take);
        tables.push(dependency_table(dependency, adopted));
    }
    refuse_shared_names(&wanted, &tables)?;

    // Tables are written in the order of their positions. Scaling every position
    // leaves a gap after the anchor that holds the dependency tables, with any
    // tables nested in them, without disturbing the order of the rest.
    let scale = tables.iter().map(count_tables).sum::<isize>() + 1;
    scale_positions(document.as_table_mut(), scale);
    let mut next = anchor * scale;
    let mut dependencies = ArrayOfTables::new();
    for mut table in tables {
        assign_positions(&mut table, &mut next);
        dependencies.push(table);
    }
    if !dependencies.is_empty() {
        document.insert(DEPENDENCIES, Item::ArrayOfTables(dependencies));
    }
    Ok(document.to_string())
}

/// A dependency to write: the target's original name and its source.
struct WantedDependency {
    name: String,
    source: String,
}

/// Resolves each client's source and orders the list as the helper's
/// `sortDependencies` does: by the target's original name, clients of one name in
/// the order given.
fn wanted_dependencies(
    module_directory: &[String],
    clients: &[ModuleClient],
) -> Result<Vec<WantedDependency>, EngineDiagnostic> {
    let mut wanted = Vec::with_capacity(clients.len());
    let mut sources = BTreeMap::<&str, String>::new();
    for client in clients {
        let name = client.name();
        if name.is_empty() {
            return Err(refusal("a client's module has no name"));
        }
        let source = match client {
            ModuleClient::Local {
                source_root_subpath,
                ..
            } => relative_source(module_directory, source_root_subpath)?,
            ModuleClient::Git { source, .. } => source.clone(),
        };
        validate_dependency_source(&source)?;
        // The helper keeps the last of two clients with one name. The engine loads
        // a module's dependencies by name, so one of the two would silently go.
        if let Some(first) = sources.insert(name, source.clone()) {
            return Err(refusal(format!(
                "clients `{first}` and `{source}` are both modules named `{name}`; a module's \
                 dependencies need distinct names, so remove one of the clients"
            )));
        }
        wanted.push(WantedDependency {
            name: name.to_owned(),
            source,
        });
    }
    wanted.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(wanted)
}

/// An adopted dependency keeps the name written for it, which may be the name of
/// another client's module or of another adopted dependency.
fn refuse_shared_names(
    wanted: &[WantedDependency],
    tables: &[Table],
) -> Result<(), EngineDiagnostic> {
    let mut sources = BTreeMap::<&str, &str>::new();
    for (dependency, table) in wanted.iter().zip(tables) {
        let name = string_field(table, "name").unwrap_or(&dependency.name);
        if let Some(first) = sources.insert(name, &dependency.source) {
            return Err(refusal(format!(
                "dependencies `{first}` and `{}` would both be named `{name}`; rename the \
                 dependency in {MODULE_MANIFEST} or remove one of the clients",
                dependency.source
            )));
        }
    }
    Ok(())
}

/// Takes the manifest's dependencies out of the document, in the order written.
///
/// An inline `dependencies = [...]` array is read as the tables it stands for, so
/// it is written back as tables, the only form the helper writes.
fn existing_dependencies(document: &mut DocumentMut) -> Result<Vec<Table>, EngineDiagnostic> {
    let unsupported = || {
        refusal(format!(
            "`dependencies` in {MODULE_MANIFEST} is not a list of tables; its dependencies cannot \
             be written"
        ))
    };
    match document.remove(DEPENDENCIES) {
        None => Ok(Vec::new()),
        Some(Item::ArrayOfTables(tables)) => Ok(tables.into_iter().collect()),
        Some(Item::Value(Value::Array(entries))) => entries
            .into_iter()
            .map(|entry| match entry {
                Value::InlineTable(table) => Ok(table.into_table()),
                Value::String(source) => {
                    let mut table = Table::new();
                    table.insert("source", value(source.into_value()));
                    Ok(table)
                }
                _ => Err(unsupported()),
            })
            .collect(),
        Some(_) => Err(unsupported()),
    }
}

/// Builds one dependency table in the helper's layout: `name`, then `source`, then
/// whatever else an adopted dependency carries, except its pin.
fn dependency_table(dependency: &WantedDependency, adopted: Option<Table>) -> Table {
    let mut table = Table::new();
    let Some(mut adopted) = adopted else {
        table.insert("name", value(&dependency.name));
        table.insert("source", value(&dependency.source));
        return table;
    };
    // `adoptedDependency` names a dependency only when it has no name of its own.
    match adopted.remove_entry("name") {
        Some((key, item)) if item.as_str().is_some_and(|name| !name.is_empty()) => {
            insert_kept(&mut table, key, item);
        }
        _ => {
            table.insert("name", value(&dependency.name));
        }
    }
    match adopted.remove_entry("source") {
        Some((key, item)) if item.as_str() == Some(dependency.source.as_str()) => {
            insert_kept(&mut table, key, item);
        }
        _ => {
            table.insert("source", value(&dependency.source));
        }
    }
    adopted.remove("pin");
    let others = adopted
        .iter()
        .map(|(key, _)| key.to_owned())
        .collect::<Vec<_>>();
    for other in others {
        if let Some((key, item)) = adopted.remove_entry(&other) {
            insert_kept(&mut table, key, item);
        }
    }
    // Comments above the table's header stay with it, after the one blank line
    // that separates tables.
    let comments = adopted
        .decor()
        .prefix()
        .and_then(|prefix| prefix.as_str())
        .unwrap_or_default()
        .lines()
        .filter(|line| line.trim_start().starts_with('#'))
        .map(|line| format!("{}\n", line.trim_end()))
        .collect::<String>();
    if !comments.is_empty() {
        table.decor_mut().set_prefix(format!("\n{comments}"));
    }
    if let Some(suffix) = adopted
        .decor()
        .suffix()
        .and_then(|suffix| suffix.as_str())
        .filter(|suffix| !suffix.is_empty())
    {
        table.decor_mut().set_suffix(suffix);
    }
    table
}

/// Keeps a setting as it is written, without the indentation before its key.
fn insert_kept(table: &mut Table, mut key: Key, item: Item) {
    let prefix = key
        .leaf_decor()
        .prefix()
        .and_then(|prefix| prefix.as_str())
        .unwrap_or_default()
        .trim_end_matches([' ', '\t'])
        .to_owned();
    key.leaf_decor_mut().set_prefix(prefix);
    table.insert_formatted(&key, item);
}

fn string_field<'a>(table: &'a Table, field: &str) -> Option<&'a str> {
    table
        .get(field)
        .and_then(Item::as_str)
        .filter(|value| !value.is_empty())
}

/// The position the dependency tables follow: the place of the first dependency
/// table already written, else the end of `[runtime]`, where the helper's
/// `tomlContents` puts them, else the start of the tables.
fn anchor_position(document: &DocumentMut) -> isize {
    let written = document
        .get(DEPENDENCIES)
        .and_then(Item::as_array_of_tables)
        .and_then(|tables| tables.iter().find_map(Table::position));
    if let Some(position) = written {
        return position - 1;
    }
    if let Some(position) = document
        .get("runtime")
        .and_then(Item::as_table)
        .and_then(|runtime| extreme_position(runtime, isize::max))
    {
        return position;
    }
    document
        .iter()
        .filter(|(key, _)| *key != DEPENDENCIES)
        .filter_map(|(_, item)| extreme_item_position(item, isize::min))
        .reduce(isize::min)
        .map_or(0, |position| position - 1)
}

fn extreme_position(table: &Table, pick: fn(isize, isize) -> isize) -> Option<isize> {
    table
        .iter()
        .filter_map(|(_, item)| extreme_item_position(item, pick))
        .chain(table.position())
        .reduce(pick)
}

fn extreme_item_position(item: &Item, pick: fn(isize, isize) -> isize) -> Option<isize> {
    match item {
        Item::Table(table) => extreme_position(table, pick),
        Item::ArrayOfTables(tables) => tables
            .iter()
            .filter_map(|table| extreme_position(table, pick))
            .reduce(pick),
        _ => None,
    }
}

fn count_tables(table: &Table) -> isize {
    1 + table
        .iter()
        .map(|(_, item)| match item {
            Item::Table(nested) => count_tables(nested),
            Item::ArrayOfTables(nested) => nested.iter().map(count_tables).sum(),
            _ => 0,
        })
        .sum::<isize>()
}

fn scale_positions(table: &mut Table, scale: isize) {
    if let Some(position) = table.position() {
        table.set_position(Some(position * scale));
    }
    for (_, item) in table.iter_mut() {
        match item {
            Item::Table(nested) => scale_positions(nested, scale),
            Item::ArrayOfTables(nested) => {
                for table in nested.iter_mut() {
                    scale_positions(table, scale);
                }
            }
            _ => {}
        }
    }
}

fn assign_positions(table: &mut Table, next: &mut isize) {
    *next += 1;
    table.set_position(Some(*next));
    for (_, item) in table.iter_mut() {
        match item {
            Item::Table(nested) => assign_positions(nested, next),
            Item::ArrayOfTables(nested) => {
                for table in nested.iter_mut() {
                    assign_positions(table, next);
                }
            }
            _ => {}
        }
    }
}

/// The helper's `normalizePath`: one leading `/`, one leading `./` and one trailing
/// `/` are not part of a path, and nothing is `.`.
fn normalize_path(path: &str) -> &str {
    let path = path.strip_prefix('/').unwrap_or(path);
    let path = path.strip_prefix("./").unwrap_or(path);
    let path = path.strip_suffix('/').unwrap_or(path);
    if path.is_empty() { "." } else { path }
}

/// The helper's `validateDependencySource`: a source that is an absolute or a
/// drive path cannot be relative to the manifest.
fn validate_dependency_source(source: &str) -> Result<(), EngineDiagnostic> {
    let drive = source.split(':').next().unwrap_or_default();
    let drive_path = drive.len() == 1
        && drive.chars().all(|letter| letter.is_ascii_alphabetic())
        && (source.contains(":/") || source.contains(":\\"));
    if source.starts_with('/') || source.starts_with('\\') || drive_path {
        return Err(refusal(format!(
            "local dependency source must be relative to the manifest file: {source}"
        )));
    }
    Ok(())
}

/// Splits a path from the workspace root into its directories. A path that is
/// still absolute after normalization, or that leaves the workspace, is refused.
fn workspace_components(path: &str) -> Result<Vec<String>, EngineDiagnostic> {
    let normalized = normalize_path(path);
    validate_dependency_source(normalized)?;
    let mut components = Vec::new();
    for component in normalized.split('/') {
        match component {
            "" | "." => {}
            ".." => {
                return Err(refusal(format!(
                    "local dependency source must be inside the workspace: {path}"
                )));
            }
            component => components.push(component.to_owned()),
        }
    }
    Ok(components)
}

/// A local client's source: its source root as a path from the module directory.
fn relative_source(
    module_directory: &[String],
    source_root_subpath: &str,
) -> Result<String, EngineDiagnostic> {
    let target = workspace_components(source_root_subpath)?;
    let shared = module_directory
        .iter()
        .zip(&target)
        .take_while(|(left, right)| left == right)
        .count();
    let relative = std::iter::repeat_n("..", module_directory.len() - shared)
        .chain(target[shared..].iter().map(String::as_str))
        .collect::<Vec<_>>()
        .join("/");
    Ok(if relative.is_empty() {
        ".".to_owned()
    } else {
        relative
    })
}

/// The module's directory as a path from the workspace root.
fn module_directory(
    workspace_root: &RelativeOperationPath,
    module_root: &RelativeOperationPath,
) -> Result<String, EngineDiagnostic> {
    let module = module_root.as_str();
    let workspace = workspace_root.as_str();
    if module == workspace {
        return Ok(".".to_owned());
    }
    module
        .strip_prefix(workspace)
        .and_then(|rest| rest.strip_prefix('/'))
        .map(str::to_owned)
        .ok_or_else(|| {
            EngineDiagnostic::new(
                EngineDiagnosticCode::OperationInputInvalid,
                Some(module),
                "the module root is outside the workspace root",
            )
        })
}

fn join(
    root: &RelativeOperationPath,
    name: &str,
) -> Result<RelativeOperationPath, EngineDiagnostic> {
    RelativeOperationPath::parse(&format!("{root}/{name}")).map_err(|_| {
        EngineDiagnostic::new(
            EngineDiagnosticCode::OutputPathEscape,
            Some(root.as_str()),
            "module configuration path is not canonical",
        )
    })
}

fn refusal(message: impl AsRef<str>) -> EngineDiagnostic {
    EngineDiagnostic::new(
        EngineDiagnosticCode::OperationInputInvalid,
        Some(COORDINATE),
        message,
    )
}
