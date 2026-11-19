//! The SDK dependency of a Cargo package that has module clients.
//!
//! Generated clients are written for one release of the SDK, so the package must
//! depend on exactly that release: `version = "=X.Y.Z"` from the registry, or the
//! repository at one commit. This plans the one edit that makes it so, in the
//! package's `[dependencies]`, and keeps everything else in the manifest as its
//! author wrote it:
//!
//! - A dependency on the SDK package keeps its key, and so the name the package's
//!   code reaches the SDK by, and its other fields such as `features`. Only the
//!   fields that say where the SDK comes from, and which release, are replaced.
//! - A package without one gains `dagger-sdk`.
//! - A dependency inherited from a Cargo workspace is not this package's to edit.
//!   It is accepted when the workspace states exactly the release, and is otherwise
//!   refused with the entry to write there. A workspace whose manifest generation
//!   cannot read is refused with the entry to write in the package instead.
//!
//! `Cargo.lock` is never read or written: Cargo brings it up to date on the
//! author's next build.

use std::str::FromStr as _;

use toml_edit::{DocumentMut, InlineTable, Item, Table, TableLike, Value, value};

use crate::diagnostic::{EngineDiagnostic, EngineDiagnosticCode};
use crate::{
    PublishedSdkDependency, RelativeOperationPath, SDK_DEPENDENCY_KEY, SDK_LIBRARY, SDK_PACKAGE,
};

/// The fields of a dependency that say where a package comes from and which release.
const SOURCE_FIELDS: &[&str] = &[
    "version",
    "git",
    "rev",
    "branch",
    "tag",
    "path",
    "registry",
    "registry-index",
];

/// A Cargo manifest above the package, which may be its workspace's.
pub(crate) struct AncestorManifest<'a> {
    /// The manifest's path as the package's author reads it, such as `../Cargo.toml`.
    pub(crate) display: String,
    /// The manifest's operation path.
    pub(crate) path: &'a RelativeOperationPath,
    /// The manifest's bytes.
    pub(crate) bytes: &'a [u8],
}

/// The package's SDK dependency after planning.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct PackageDependency {
    /// The complete manifest to write, when the edit changes it.
    pub(crate) rendered: Option<Vec<u8>>,
    /// The name the package's code reaches the SDK by.
    pub(crate) alias: String,
}

/// Plans the package's dependency on exactly `dependency`.
///
/// `ancestors` are the Cargo manifests in the directories above the package,
/// nearest first. They are read only when the package inherits the dependency.
///
/// # Errors
///
/// Returns a diagnostic when the manifest is not a package's, when its dependency
/// tables cannot be edited, and when an inherited dependency does not state exactly
/// the release or comes from a workspace manifest that is not among `ancestors`.
pub(crate) fn plan_package_dependency(
    manifest_path: &RelativeOperationPath,
    manifest: &[u8],
    ancestors: &[AncestorManifest<'_>],
    dependency: &PublishedSdkDependency,
) -> Result<PackageDependency, EngineDiagnostic> {
    let mut document = parse(manifest_path, manifest)?;
    if document
        .get("package")
        .and_then(Item::as_table_like)
        .is_none()
    {
        return Err(invalid(
            manifest_path,
            "Cargo.toml has no [package]; module clients are generated into a Cargo package",
        ));
    }
    // A workspace manifest matters only to an inherited dependency, so one that
    // cannot be read fails generation only for a package that inherits. Cargo's
    // workspace is the one `package.workspace` names, else the nearest manifest
    // above the package with `[workspace]`.
    let inherits = document
        .get("dependencies")
        .and_then(Item::as_table_like)
        .is_some_and(|dependencies| dependencies.iter().any(|(_, item)| is_inherited(item)));
    let pointer = document
        .get("package")
        .and_then(Item::as_table_like)
        .and_then(|package| package.get("workspace"))
        .and_then(Item::as_str)
        .map(str::to_owned);
    let workspace = if !inherits {
        None
    } else if document.get("workspace").is_some() {
        Some((
            "Cargo.toml".to_owned(),
            manifest_path.clone(),
            document.clone(),
        ))
    } else if let Some(pointer) = &pointer {
        let named = named_manifest(manifest_path, pointer);
        ancestors
            .iter()
            .find(|ancestor| Some(ancestor.path.as_str()) == named.as_deref())
            .map(|ancestor| {
                parse(ancestor.path, ancestor.bytes)
                    .map(|document| (ancestor.display.clone(), ancestor.path.clone(), document))
            })
            .transpose()?
            .filter(|(_, _, document)| document.get("workspace").is_some())
    } else {
        ancestors
            .iter()
            .map(|ancestor| {
                parse(ancestor.path, ancestor.bytes)
                    .map(|document| (ancestor.display.clone(), ancestor.path.clone(), document))
            })
            .find(|parsed| {
                parsed
                    .as_ref()
                    .map_or(true, |(_, _, document)| document.get("workspace").is_some())
            })
            .transpose()?
    };
    let workspace_dependencies = workspace.as_ref().and_then(|(_, _, document)| {
        document
            .get("workspace")
            .and_then(Item::as_table_like)
            .and_then(|workspace| workspace.get("dependencies"))
            .and_then(Item::as_table_like)
    });

    if !document.contains_key("dependencies") {
        document.insert("dependencies", Item::Table(Table::new()));
    }
    let dependencies = document
        .get_mut("dependencies")
        .and_then(Item::as_table_like_mut)
        .ok_or_else(|| invalid(manifest_path, "[dependencies] of Cargo.toml is not a table"))?;

    let found = dependencies.iter().find_map(|(key, item)| {
        let inherited = is_inherited(item);
        let declared = if inherited {
            workspace_dependencies.and_then(|dependencies| dependencies.get(key))
        } else {
            Some(item)
        };
        match declared.map(package_field) {
            Some(Some(package)) => {
                (package == SDK_PACKAGE).then(|| (key.to_owned(), inherited, true))
            }
            Some(None) => (key == SDK_PACKAGE).then(|| (key.to_owned(), inherited, false)),
            // An inherited entry that no workspace manifest declares is the SDK's
            // when its key is one of the SDK's names; it is refused below.
            None => (key == SDK_PACKAGE || key == SDK_DEPENDENCY_KEY)
                .then(|| (key.to_owned(), inherited, key != SDK_PACKAGE)),
        }
    });

    let Some((key, inherited, renamed)) = found else {
        if dependencies.contains_key(SDK_DEPENDENCY_KEY) {
            return Err(EngineDiagnostic::new(
                EngineDiagnosticCode::SdkDependencyConflict,
                Some(manifest_path.as_str()),
                format!(
                    "the `{SDK_DEPENDENCY_KEY}` dependency of Cargo.toml is not the `{SDK_PACKAGE}` package; generated clients need the SDK, and add it under that name"
                ),
            ));
        }
        dependencies.insert(
            SDK_DEPENDENCY_KEY,
            requirement_item(SDK_DEPENDENCY_KEY, dependency),
        );
        return Ok(PackageDependency {
            rendered: Some(document.to_string().into_bytes()),
            alias: alias(SDK_DEPENDENCY_KEY, true),
        });
    };
    let alias = alias(&key, renamed);

    if inherited {
        let Some((display, path, _)) = workspace.as_ref() else {
            let source = if pointer.is_some() {
                "the Cargo workspace that `package.workspace` names, whose manifest is not above the package in the Dagger workspace"
            } else {
                "a Cargo workspace, but no Cargo.toml above the package in the Dagger workspace has [workspace]"
            };
            return Err(EngineDiagnostic::new(
                EngineDiagnosticCode::SdkDependencyConflict,
                Some(manifest_path.as_str()),
                format!(
                    "the package inherits `{key}` from {source}; write {} in the package's own [dependencies] instead",
                    entry_to_write(&key, dependency)
                ),
            ));
        };
        let exact = workspace_dependencies
            .and_then(|dependencies| dependencies.get(&key))
            .is_some_and(|entry| is_exact(&key, entry, dependency));
        if !exact {
            return Err(EngineDiagnostic::new(
                EngineDiagnosticCode::SdkDependencyConflict,
                Some(path.as_str()),
                format!(
                    "the package inherits `{key}` from its Cargo workspace, which does not state the SDK release these clients are generated for; write {} under [workspace.dependencies] in {display}",
                    entry_to_write(&key, dependency)
                ),
            ));
        }
        return Ok(PackageDependency {
            rendered: None,
            alias,
        });
    }

    let item = dependencies
        .get_mut(&key)
        .expect("the dependency was found under this key");
    if is_exact(&key, item, dependency) {
        return Ok(PackageDependency {
            rendered: None,
            alias,
        });
    }
    match item.as_table_like_mut() {
        Some(table) => {
            for field in SOURCE_FIELDS {
                table.remove(field);
            }
            for (field, requirement) in requirement_fields(dependency) {
                table.insert(field, value(requirement));
            }
        }
        // A bare version string, which only the package's own name can carry.
        None => *item = requirement_item(&key, dependency),
    }
    // An inline table keeps the space before its closing brace on its last value,
    // which is no longer the last.
    if let Some(table) = item.as_inline_table_mut() {
        table.fmt();
    }
    Ok(PackageDependency {
        rendered: Some(document.to_string().into_bytes()),
        alias,
    })
}

fn parse(path: &RelativeOperationPath, bytes: &[u8]) -> Result<DocumentMut, EngineDiagnostic> {
    let source =
        std::str::from_utf8(bytes).map_err(|_| invalid(path, "Cargo.toml is not UTF-8"))?;
    DocumentMut::from_str(source).map_err(|_| invalid(path, "Cargo.toml is not valid TOML"))
}

// The operation path of the manifest of the Cargo workspace that `package.workspace`
// names, a directory relative to the package. `None` when the pointer is absolute or
// leaves the operation root's first directory, where the adapter mounts the workspace.
fn named_manifest(manifest_path: &RelativeOperationPath, pointer: &str) -> Option<String> {
    if pointer.starts_with('/') || pointer.contains('\\') {
        return None;
    }
    let mut components = manifest_path.as_str().split('/').collect::<Vec<_>>();
    components.pop();
    for component in pointer.split('/') {
        match component {
            "" | "." => {}
            ".." => {
                components.pop();
                if components.is_empty() {
                    return None;
                }
            }
            name => components.push(name),
        }
    }
    components.push("Cargo.toml");
    Some(components.join("/"))
}

fn is_inherited(item: &Item) -> bool {
    item.as_table_like()
        .and_then(|table| table.get("workspace"))
        .and_then(Item::as_bool)
        == Some(true)
}

fn package_field(item: &Item) -> Option<&str> {
    item.as_table_like()?.get("package")?.as_str()
}

// Cargo names a dependency in code by its key when the entry has a `package`
// field, and by the package's library otherwise.
fn alias(key: &str, renamed: bool) -> String {
    if renamed {
        key.replace('-', "_")
    } else {
        SDK_LIBRARY.to_owned()
    }
}

fn requirement_fields(dependency: &PublishedSdkDependency) -> Vec<(&'static str, String)> {
    match dependency {
        PublishedSdkDependency::Registry { exact_version, .. } => {
            vec![("version", format!("={exact_version}"))]
        }
        PublishedSdkDependency::Git { url, revision, .. } => {
            vec![("git", url.to_string()), ("rev", revision.to_string())]
        }
    }
}

fn requirement_item(key: &str, dependency: &PublishedSdkDependency) -> Item {
    if key == SDK_PACKAGE
        && let PublishedSdkDependency::Registry { exact_version, .. } = dependency
    {
        return value(format!("={exact_version}"));
    }
    let mut table = InlineTable::new();
    if key != SDK_PACKAGE {
        table.insert("package", Value::from(SDK_PACKAGE));
    }
    for (field, requirement) in requirement_fields(dependency) {
        table.insert(field, Value::from(requirement));
    }
    Item::Value(Value::InlineTable(table))
}

// Exactly the release: its source and requirement, and no other source field.
fn is_exact(key: &str, item: &Item, dependency: &PublishedSdkDependency) -> bool {
    if let Some(version) = item.as_str() {
        return key == SDK_PACKAGE
            && matches!(
                dependency,
                PublishedSdkDependency::Registry { exact_version, .. }
                    if version == format!("={exact_version}")
            );
    }
    let Some(table) = item.as_table_like() else {
        return false;
    };
    let required = requirement_fields(dependency);
    package_matches(key, table)
        && required
            .iter()
            .all(|(field, requirement)| field_str(table, field) == Some(requirement.as_str()))
        && SOURCE_FIELDS.iter().all(|field| {
            table.get(field).is_none() || required.iter().any(|(required, _)| required == field)
        })
}

fn package_matches(key: &str, table: &dyn TableLike) -> bool {
    match field_str(table, "package") {
        Some(package) => package == SDK_PACKAGE,
        None => key == SDK_PACKAGE,
    }
}

fn field_str<'a>(table: &'a dyn TableLike, field: &str) -> Option<&'a str> {
    table.get(field).and_then(Item::as_str)
}

// The entry an author writes in `[workspace.dependencies]`. A diagnostic never
// carries a URL, so a repository is named by its host and path.
fn entry_to_write(key: &str, dependency: &PublishedSdkDependency) -> String {
    let package = if key == SDK_PACKAGE {
        String::new()
    } else {
        format!("package = \"{SDK_PACKAGE}\", ")
    };
    match dependency {
        PublishedSdkDependency::Registry { exact_version, .. } if key == SDK_PACKAGE => {
            format!("`{key} = \"={exact_version}\"`")
        }
        PublishedSdkDependency::Registry { exact_version, .. } => {
            format!("`{key} = {{ {package}version = \"={exact_version}\" }}`")
        }
        PublishedSdkDependency::Git { url, revision, .. } => format!(
            "`{key} = {{ {package}git = \"<repository>\", rev = \"{revision}\" }}`, with the HTTPS URL of {} for <repository>,",
            url.as_str()
                .split_once("://")
                .map_or(url.as_str(), |(_, rest)| rest)
        ),
    }
}

fn invalid(path: &RelativeOperationPath, message: &str) -> EngineDiagnostic {
    EngineDiagnostic::new(
        EngineDiagnosticCode::CargoManifestInvalid,
        Some(path.as_str()),
        message,
    )
}

#[cfg(test)]
mod tests {
    use super::{AncestorManifest, PackageDependency, plan_package_dependency};
    use crate::diagnostic::EngineDiagnosticCode;
    use crate::{EngineDiagnostic, PublishedSdkDependency, RelativeOperationPath};

    const REVISION: &str = "1e6e0d5c7ccfe1c712c692c229fe2c0b35d55a4b";
    const PACKAGE: &str = "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n";

    fn path(value: &str) -> RelativeOperationPath {
        RelativeOperationPath::parse(value).expect("fixture path must parse")
    }

    fn release() -> PublishedSdkDependency {
        PublishedSdkDependency::Registry {
            registry: "crates-io".parse().unwrap(),
            package: "tokeira-dagger-sdk".parse().unwrap(),
            exact_version: "0.9.0".parse().unwrap(),
        }
    }

    fn commit() -> PublishedSdkDependency {
        PublishedSdkDependency::Git {
            url: "https://github.com/tokeira/dagger-rust-sdk"
                .parse()
                .unwrap(),
            revision: REVISION.parse().unwrap(),
            package: "tokeira-dagger-sdk".parse().unwrap(),
        }
    }

    fn plan(
        manifest: &str,
        workspace: Option<&str>,
        dependency: &PublishedSdkDependency,
    ) -> Result<PackageDependency, EngineDiagnostic> {
        let workspace_path = path("workspace/Cargo.toml");
        let ancestors = workspace
            .map(|bytes| AncestorManifest {
                display: "../Cargo.toml".to_owned(),
                path: &workspace_path,
                bytes: bytes.as_bytes(),
            })
            .into_iter()
            .collect::<Vec<_>>();
        plan_package_dependency(
            &path("workspace/app/Cargo.toml"),
            manifest.as_bytes(),
            &ancestors,
            dependency,
        )
    }

    fn edited(manifest: &str, dependency: &PublishedSdkDependency) -> (String, String) {
        let plan = plan(manifest, None, dependency).expect("the manifest must be editable");
        (
            String::from_utf8(plan.rendered.expect("the edit must change the manifest")).unwrap(),
            plan.alias,
        )
    }

    #[test]
    fn a_package_without_the_sdk_gains_dagger_sdk() {
        let manifest = format!("{PACKAGE}\n[dependencies]\n# kept\nserde = \"1\"\n");
        let (rendered, alias) = edited(&manifest, &release());
        assert_eq!(
            rendered,
            format!(
                "{manifest}dagger-sdk = {{ package = \"tokeira-dagger-sdk\", version = \"=0.9.0\" }}\n"
            )
        );
        assert_eq!(alias, "dagger_sdk");

        // Without a dependency table at all, and from a commit.
        let (rendered, alias) = edited(PACKAGE, &commit());
        assert_eq!(
            rendered,
            format!(
                "{PACKAGE}\n[dependencies]\ndagger-sdk = {{ package = \"tokeira-dagger-sdk\", git = \"https://github.com/tokeira/dagger-rust-sdk\", rev = \"{REVISION}\" }}\n"
            )
        );
        assert_eq!(alias, "dagger_sdk");

        // The edit is the whole edit: a second plan changes nothing.
        let again = plan(&rendered, None, &commit()).unwrap();
        assert_eq!(again.rendered, None);
    }

    #[test]
    fn an_existing_key_and_its_other_fields_are_kept() {
        let manifest = format!(
            "{PACKAGE}\n[dependencies]\nsdk = {{ package = \"tokeira-dagger-sdk\", version = \"0.8\", features = [\"a\"], default-features = false }} # mine\nserde = \"1\"\n"
        );
        let (rendered, alias) = edited(&manifest, &release());
        assert_eq!(
            rendered,
            format!(
                "{PACKAGE}\n[dependencies]\nsdk = {{ package = \"tokeira-dagger-sdk\", features = [\"a\"], default-features = false, version = \"=0.9.0\" }} # mine\nserde = \"1\"\n"
            )
        );
        assert_eq!(alias, "sdk");

        // Every field that names a source goes when the source changes.
        let manifest = format!(
            "{PACKAGE}\n[dependencies.my-sdk]\npackage = \"tokeira-dagger-sdk\"\npath = \"../sdk\"\nversion = \"0.8\"\nfeatures = [\"a\"]\n"
        );
        let (rendered, alias) = edited(&manifest, &commit());
        assert_eq!(
            rendered,
            format!(
                "{PACKAGE}\n[dependencies.my-sdk]\npackage = \"tokeira-dagger-sdk\"\nfeatures = [\"a\"]\ngit = \"https://github.com/tokeira/dagger-rust-sdk\"\nrev = \"{REVISION}\"\n"
            )
        );
        assert_eq!(alias, "my_sdk");
    }

    #[test]
    fn the_package_under_its_own_name_is_reached_as_dagger_sdk() {
        let manifest = format!("{PACKAGE}\n[dependencies]\ntokeira-dagger-sdk = \"0.8\"\n");
        let (rendered, alias) = edited(&manifest, &release());
        assert_eq!(
            rendered,
            format!("{PACKAGE}\n[dependencies]\ntokeira-dagger-sdk = \"=0.9.0\"\n")
        );
        assert_eq!(alias, "dagger_sdk");

        let (rendered, alias) = edited(&manifest, &commit());
        assert_eq!(
            rendered,
            format!(
                "{PACKAGE}\n[dependencies]\ntokeira-dagger-sdk = {{ git = \"https://github.com/tokeira/dagger-rust-sdk\", rev = \"{REVISION}\" }}\n"
            )
        );
        assert_eq!(alias, "dagger_sdk");
    }

    #[test]
    fn an_exact_dependency_is_left_alone() {
        for (manifest, dependency, alias) in [
            (
                "dagger-sdk = { package = \"tokeira-dagger-sdk\", version = \"=0.9.0\", features = [\"a\"] }",
                release(),
                "dagger_sdk",
            ),
            ("tokeira-dagger-sdk = \"=0.9.0\"", release(), "dagger_sdk"),
            (
                "sdk = { package = \"tokeira-dagger-sdk\", rev = \"1e6e0d5c7ccfe1c712c692c229fe2c0b35d55a4b\", git = \"https://github.com/tokeira/dagger-rust-sdk\" }",
                commit(),
                "sdk",
            ),
        ] {
            let manifest = format!("{PACKAGE}\n[dependencies]\n{manifest}\n");
            assert_eq!(
                plan(&manifest, None, &dependency).unwrap(),
                PackageDependency {
                    rendered: None,
                    alias: alias.to_owned()
                },
                "{manifest}"
            );
        }
    }

    #[test]
    fn an_inherited_dependency_is_accepted_only_when_exact() {
        let manifest = format!(
            "{PACKAGE}\n[dependencies]\ndagger-sdk = {{ workspace = true, features = [\"a\"] }}\n"
        );
        let exact = "[workspace]\nmembers = [\"app\"]\n\n[workspace.dependencies]\ndagger-sdk = { package = \"tokeira-dagger-sdk\", version = \"=0.9.0\" }\n";
        assert_eq!(
            plan(&manifest, Some(exact), &release()).unwrap(),
            PackageDependency {
                rendered: None,
                alias: "dagger_sdk".to_owned()
            }
        );

        let loose = exact.replace("=0.9.0", "0.9");
        let error = plan(&manifest, Some(&loose), &release()).unwrap_err();
        assert_eq!(error.code, EngineDiagnosticCode::SdkDependencyConflict);
        assert_eq!(error.coordinate.as_deref(), Some("workspace/Cargo.toml"));
        assert_eq!(
            error.message,
            "the package inherits `dagger-sdk` from its Cargo workspace, which does not state the SDK release these clients are generated for; write `dagger-sdk = { package = \"tokeira-dagger-sdk\", version = \"=0.9.0\" }` under [workspace.dependencies] in ../Cargo.toml"
        );

        // From a commit, the entry names the repository without its URL, which a
        // diagnostic never carries.
        let error = plan(&manifest, Some(exact), &commit()).unwrap_err();
        assert_eq!(
            error.message,
            format!(
                "the package inherits `dagger-sdk` from its Cargo workspace, which does not state the SDK release these clients are generated for; write `dagger-sdk = {{ package = \"tokeira-dagger-sdk\", git = \"<repository>\", rev = \"{REVISION}\" }}`, with the HTTPS URL of github.com/tokeira/dagger-rust-sdk for <repository>, under [workspace.dependencies] in ../Cargo.toml"
            )
        );

        // The dotted form inherits too, and the package's own name needs no `package`.
        let manifest = format!("{PACKAGE}\n[dependencies]\ntokeira-dagger-sdk.workspace = true\n");
        let workspace =
            "[workspace]\n\n[workspace.dependencies]\ntokeira-dagger-sdk = \"=0.9.0\"\n";
        assert_eq!(
            plan(&manifest, Some(workspace), &release()).unwrap().alias,
            "dagger_sdk"
        );
        let error = plan(
            &manifest,
            Some(&workspace.replace("=0.9.0", "=0.8.0")),
            &release(),
        )
        .unwrap_err();
        assert!(
            error
                .message
                .contains("write `tokeira-dagger-sdk = \"=0.9.0\"` under"),
            "{}",
            error.message
        );
    }

    #[test]
    fn an_inherited_dependency_without_its_workspace_entry_is_refused() {
        // No Cargo workspace above the package that generation can read.
        let manifest = format!("{PACKAGE}\n[dependencies]\ntokeira-dagger-sdk.workspace = true\n");
        let error = plan(&manifest, None, &release()).unwrap_err();
        assert_eq!(error.code, EngineDiagnosticCode::SdkDependencyConflict);
        assert_eq!(
            error.coordinate.as_deref(),
            Some("workspace/app/Cargo.toml")
        );
        assert_eq!(
            error.message,
            "the package inherits `tokeira-dagger-sdk` from a Cargo workspace, but no Cargo.toml above the package in the Dagger workspace has [workspace]; write `tokeira-dagger-sdk = \"=0.9.0\"` in the package's own [dependencies] instead"
        );

        // A workspace that does not declare the entry is told the entry to write.
        let manifest = format!("{PACKAGE}\n[dependencies]\ndagger-sdk = {{ workspace = true }}\n");
        let error = plan(
            &manifest,
            Some("[workspace]\nmembers = [\"app\"]\n"),
            &release(),
        )
        .unwrap_err();
        assert_eq!(error.code, EngineDiagnosticCode::SdkDependencyConflict);
        assert_eq!(error.coordinate.as_deref(), Some("workspace/Cargo.toml"));
        assert!(
            error.message.ends_with(
                "write `dagger-sdk = { package = \"tokeira-dagger-sdk\", version = \"=0.9.0\" }` under [workspace.dependencies] in ../Cargo.toml"
            ),
            "{}",
            error.message
        );
    }

    #[test]
    fn package_workspace_names_the_workspace_a_dependency_is_inherited_from() {
        let exact = "[workspace]\nmembers = [\"app\"]\n\n[workspace.dependencies]\ndagger-sdk = { package = \"tokeira-dagger-sdk\", version = \"=0.9.0\" }\n";
        let manifest = |pointer: &str| {
            format!(
                "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\nworkspace = \"{pointer}\"\n\n[dependencies]\ndagger-sdk.workspace = true\n"
            )
        };
        for pointer in ["..", "../", "./.."] {
            assert_eq!(
                plan(&manifest(pointer), Some(exact), &release())
                    .unwrap()
                    .alias,
                "dagger_sdk",
                "{pointer}"
            );
        }
        // Anywhere else is a manifest generation does not read.
        for pointer in ["../elsewhere", "../..", "/workspace"] {
            let error = plan(&manifest(pointer), Some(exact), &release()).unwrap_err();
            assert_eq!(
                error.code,
                EngineDiagnosticCode::SdkDependencyConflict,
                "{pointer}"
            );
            assert_eq!(
                error.coordinate.as_deref(),
                Some("workspace/app/Cargo.toml")
            );
            assert_eq!(
                error.message,
                "the package inherits `dagger-sdk` from the Cargo workspace that `package.workspace` names, whose manifest is not above the package in the Dagger workspace; write `dagger-sdk = { package = \"tokeira-dagger-sdk\", version = \"=0.9.0\" }` in the package's own [dependencies] instead",
                "{pointer}"
            );
        }
    }

    #[test]
    fn a_manifest_that_is_not_a_packages_is_refused() {
        let error = plan("[workspace]\nmembers = [\"app\"]\n", None, &release()).unwrap_err();
        assert_eq!(error.code, EngineDiagnosticCode::CargoManifestInvalid);
        assert_eq!(
            error.coordinate.as_deref(),
            Some("workspace/app/Cargo.toml")
        );
        assert_eq!(
            error.message,
            "Cargo.toml has no [package]; module clients are generated into a Cargo package"
        );
        assert!(plan("not = [toml", None, &release()).is_err());
    }

    #[test]
    fn dagger_sdk_naming_another_package_is_refused() {
        let manifest = format!("{PACKAGE}\n[dependencies]\ndagger-sdk = \"0.20\"\n");
        let error = plan(&manifest, None, &release()).unwrap_err();
        assert_eq!(error.code, EngineDiagnosticCode::SdkDependencyConflict);
    }
}
