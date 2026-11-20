// Package main exposes the module-backed ABI adapter for the built-in Rust SDK.
package main

import (
	"context"
	"fmt"
	"path"
	"sort"
	"strings"

	"rust-sdk/internal/dagger"
	"rust-sdk/internal/metadata"
)

const (
	sdkRuntimePath          = "runtime"
	engineToolPath          = "dist/dagger-rust-engine"
	formatterToolPath       = "dist/rustfmt"
	engineDescriptorPath    = "dist/engine-source.json"
	runtimePolicyPath       = "dist/runtime-policy.json"
	operationRequestPath    = "/var/lib/dagger/rust/request.json"
	operationSchemaPath     = "/var/lib/dagger/rust/schema.json"
	operationSchemasPath    = "/var/lib/dagger/rust/schemas"
	operationProjectPath    = "/var/lib/dagger/rust/project"
	operationResultPath     = "/var/lib/dagger/rust/result.json"
	runtimePolicyMountPath  = "/usr/local/share/dagger/rust/runtime-policy.json"
	runtimePlanPath         = "/var/lib/dagger/rust/runtime-plan.json"
	runtimeProvenancePath   = "/var/lib/dagger/rust/runtime-provenance.json"
	runtimeDiagnosticPath   = "/var/lib/dagger/rust/runtime-diagnostic.json"
	operationDescriptorPath = "/usr/local/share/dagger/rust/engine-source.json"
	operationToolPath       = "/usr/local/bin/dagger-rust-engine"
	operationWorkspaceRoot  = "workspace"
	// packageClientsPath is where a Cargo package's module clients are generated,
	// beneath the package.
	packageClientsPath = "src/dagger_clients"
)

// RustSDK is immutable packaged state. Operation methods build fresh Dagger object
// graphs from this source instead of retaining mutable container or project handles.
type RustSDK struct {
	SDKSourceDir *dagger.Directory // +private
}

// New constructs the built-in adapter from the complete packaged SDK content root.
func New(
	// Complete engine-packaged Rust SDK content, including runtime and dist metadata.
	// Absent when the runtime is installed as an ordinary module; operations then
	// seal the content from the runtime's own commit.
	// +optional
	sdkSourceDir *dagger.Directory,
) (*RustSDK, error) {
	return &RustSDK{SDKSourceDir: sdkSourceDir}, nil
}

// FindClientRoot returns the nearest Cargo package root at or above the
// workspace's current location, or null when no Cargo package contains it.
func (sdk *RustSDK) FindClientRoot(ctx context.Context, ws *dagger.Workspace) (*string, error) {
	cwd, err := workspaceCwd(ctx, ws)
	if err != nil {
		return nil, err
	}
	// findUp stats a host-backed workspace, so a manifest that exists only in
	// pending workspace edits is visible solely through the current directory.
	present, err := workspaceHasFile(ctx, ws, "Cargo.toml")
	if err != nil {
		return nil, err
	}
	if present {
		return &cwd, nil
	}
	found, err := ws.FindUp(ctx, "Cargo.toml")
	if err != nil {
		return nil, fmt.Errorf("find enclosing Cargo package: %w", err)
	}
	if found == "" {
		return nil, nil
	}
	root := normalizeWorkspacePath(path.Dir("/" + strings.TrimPrefix(found, "/")))
	return &root, nil
}

// GenerateScope reconciles the Rust SDK state of one workspace scope.
//
// A scope that is not a module is a Cargo package with module clients: the engine
// schedules it only to generate the clients recorded for it, and passes the scope
// that FindClientRoot returned. Each client is generated into the package.
//
// A module scope is scaffolded and given its module configuration on first use.
// Its dependencies are then made exactly the clients recorded for it, and its
// generated code is refreshed through the engine's module code generation, which
// calls back into Codegen with the module's visible schema.
func (sdk *RustSDK) GenerateScope(
	ctx context.Context,
	ws *dagger.Workspace,
	isModule bool,
	name string,
	clients []*dagger.ModuleSource,
) (*dagger.Workspace, error) {
	scope, err := workspaceCwd(ctx, ws)
	if err != nil {
		return nil, err
	}
	if !isConfinedWorkspacePath(scope) {
		return nil, fmt.Errorf("SDK scope is not confined to the workspace")
	}
	// The engine passes each scope its recorded clients (withSDKModuleClient in
	// core/schema/workspace_sdk_module.go of dagger/dagger at v1.0.0-beta.27). A Cargo
	// package that is not a module has them generated; a module has them as dependencies.
	if !isModule {
		return sdk.generatedPackageClients(ctx, ws, scope, clients)
	}

	configured := ws
	hasManifest, err := workspaceHasAnyFile(ctx, ws, "dagger-module.toml", "dagger.json")
	if err != nil {
		return nil, err
	}
	if !hasManifest {
		// A new module names the runtime this SDK module was installed from. An
		// existing one names its own, which is the reference Codegen resolves too.
		runtime, err := installedRuntimeSource(ctx, ws)
		if err != nil {
			return nil, err
		}
		sdk, err = sdk.resolved(ctx, func(context.Context) (string, error) { return runtime, nil })
		if err != nil {
			return nil, err
		}
		configured, err = sdk.initializedModuleScope(ctx, ws, scope, name, runtime)
		if err != nil {
			return nil, err
		}
	} else {
		sdk, err = sdk.resolved(ctx, moduleRuntimeSource(scopeModuleSource(ws, scope)))
		if err != nil {
			return nil, err
		}
	}
	configured, err = sdk.moduleScopeDependencies(ctx, configured, scope, clients)
	if err != nil {
		return nil, err
	}
	generated, err := generatedModuleCode(ctx, configured, scope)
	if err != nil {
		return nil, err
	}
	return withGeneratedModuleCode(ctx, configured, generated)
}

// packageClient is one module a Cargo package has a client for, as the Rust
// operation is told of it.
type packageClient struct {
	// Name is the module's name as its author wrote it.
	Name string
	// Address is where the engine serves the module from.
	Address string
	// Pin is the commit a git address is pinned to, empty for a local module.
	Pin string
	// SchemaPath names the client's schema among those mounted for the operation.
	SchemaPath string
	// SchemaDigest is the digest of the schema's exact bytes.
	SchemaDigest string
}

// generatedPackageClients generates the scope's module clients into its Cargo
// package and returns the workspace with what the Rust operation wrote and removed.
//
// A client's schema is read through clientSchemaIntrospectionJson alone, with the
// SDK's target release as the client's version: Core is then described as the
// generated bindings' SDK types expect, and the module keeps the names of the
// engine version it declares. Making the module declare another version instead
// would rename its declarations by that version's rules.
func (sdk *RustSDK) generatedPackageClients(
	ctx context.Context,
	ws *dagger.Workspace,
	scope string,
	clients []*dagger.ModuleSource,
) (*dagger.Workspace, error) {
	runtime, err := installedRuntimeSource(ctx, ws)
	if err != nil {
		return nil, err
	}
	sdk, err = sdk.resolved(ctx, func(context.Context) (string, error) { return runtime, nil })
	if err != nil {
		return nil, err
	}
	descriptor, err := sdk.descriptorMetadata(ctx)
	if err != nil {
		return nil, err
	}
	clientRoot, err := rebasePath(scope)
	if err != nil {
		return nil, fmt.Errorf("client root: %w", err)
	}
	schemas := dag.Directory()
	documents := make([]packageClient, 0, len(clients))
	for index, client := range clients {
		document, err := packageClientIdentity(ctx, ws, client)
		if err != nil {
			return nil, err
		}
		schema := client.ClientSchemaIntrospectionJSON(dagger.ModuleSourceClientSchemaIntrospectionJSONOpts{
			ClientVersion: "v" + descriptor.EngineVersion,
		})
		contents, err := schema.Contents(ctx)
		if err != nil {
			return nil, fmt.Errorf("read the client schema of module %q: %w", document.Name, err)
		}
		document.SchemaPath = fmt.Sprintf("%d.json", index)
		document.SchemaDigest = metadata.DigestBytes([]byte(contents))
		schemas = schemas.WithFile(document.SchemaPath, schema)
		documents = append(documents, document)
	}
	request := packageClientsRequestDocument(descriptor, clientRoot, documents)
	// Workspace.directory reads ignored files unless asked to filter them, so the
	// operation sees every file already in the clients directory, tracked or not,
	// and its ownership checks cover them all.
	project := ws.Directory("/", dagger.WorkspaceDirectoryOpts{Include: packageClientInputs(scope)})
	execution, err := sdk.executeOperation(ctx, request, nil, schemas, project, "package-clients")
	if err != nil {
		return nil, err
	}
	updated := ws
	for _, touched := range execution.result.TouchedPaths {
		target, err := packageClientChange(clientRoot, touched, false)
		if err != nil {
			return nil, err
		}
		updated = updated.WithFile(target, execution.project.File(touched))
	}
	for _, removed := range execution.result.RemovedPaths {
		target, err := packageClientChange(clientRoot, removed, true)
		if err != nil {
			return nil, err
		}
		updated = updated.WithoutFile(target)
	}
	return updated, nil
}

// packageClientIdentity reads a client's name, and its address and pin as Go binds them.
func packageClientIdentity(ctx context.Context, ws *dagger.Workspace, client *dagger.ModuleSource) (packageClient, error) {
	if client == nil {
		return packageClient{}, fmt.Errorf("module client not provided")
	}
	name, err := client.ModuleOriginalName(ctx)
	if err != nil {
		return packageClient{}, fmt.Errorf("read module client name: %w", err)
	}
	kind, err := client.Kind(ctx)
	if err != nil {
		return packageClient{}, fmt.Errorf("read the source kind of module %q: %w", name, err)
	}
	var reference, pin, subpath string
	switch kind {
	case dagger.ModuleSourceKindGit:
		if reference, err = client.AsString(ctx); err != nil {
			return packageClient{}, fmt.Errorf("read the reference of module %q: %w", name, err)
		}
		if kind, pin, subpath, err = boundGitClient(ctx, ws, client, name, reference); err != nil {
			return packageClient{}, err
		}
	case dagger.ModuleSourceKindLocal:
		if subpath, err = client.SourceRootSubpath(ctx); err != nil {
			return packageClient{}, fmt.Errorf("read the path of module %q: %w", name, err)
		}
	}
	address, pin, err := clientAddress(kind, reference, pin, subpath)
	if err != nil {
		return packageClient{}, fmt.Errorf("module %q: %w", name, err)
	}
	return packageClient{Name: name, Address: address, Pin: pin}, nil
}

// clientAddress returns the address and pin a generated client serves its module
// from, as Dagger's Go SDK chooses them (GenerateUnifiedClient in
// cmd/dagger-go-sdk-codegen/generator/gogenerator/generate_unified.go of
// github.com/dagger/go-sdk at bb23e8d): a git module by its reference and the
// commit it is pinned to, and a local module by its path from the workspace root.
func clientAddress(kind dagger.ModuleSourceKind, reference, pin, sourceRootSubpath string) (string, string, error) {
	switch kind {
	case dagger.ModuleSourceKindGit:
		if reference == "" {
			return "", "", fmt.Errorf("git module client has no reference")
		}
		if err := validateOptionalRevision(pin); err != nil || pin == "" {
			return "", "", fmt.Errorf("git module client is not pinned to a full commit")
		}
		return reference, pin, nil
	case dagger.ModuleSourceKindLocal:
		return "/" + strings.TrimPrefix(sourceRootSubpath, "/"), "", nil
	default:
		return "", "", fmt.Errorf("a module client is a git or a local module, not source kind %q", kind)
	}
}

// packageClientInputs lists what the Rust operation reads for a package at scope:
// the Cargo manifest and any dagger.json in its directory and each directory above
// it, where a Cargo workspace may state an inherited dependency and a dagger.json
// may make the package a module's source, and everything in its clients directory.
func packageClientInputs(scope string) []string {
	if scope == "." {
		return []string{"Cargo.toml", "dagger.json", packageClientsPath + "/**"}
	}
	inputs := []string{}
	for directory := scope; directory != "."; directory = path.Dir(directory) {
		inputs = append(inputs, path.Join(directory, "Cargo.toml"), path.Join(directory, "dagger.json"))
	}
	return append(inputs, "Cargo.toml", "dagger.json", path.Join(scope, packageClientsPath)+"/**")
}

// packageClientChange returns the workspace path of one file the Rust operation
// wrote or removed, from the workspace root. The operation writes the package's
// manifest and its clients directory, and removes only within the clients
// directory; a result that names anything else is refused, so no generated result
// can reach workspace configuration or authored source.
func packageClientChange(clientRoot, changed string, removed bool) (string, error) {
	relative, ok := strings.CutPrefix(changed, clientRoot+"/")
	inClients := ok && strings.HasPrefix(relative, packageClientsPath+"/")
	if !inClients && (removed || !ok || relative != "Cargo.toml") {
		return "", fmt.Errorf("module client generation changed %q, which is outside what it owns", changed)
	}
	stripped, err := metadata.StripOperationRoot([]string{changed}, operationWorkspaceRoot)
	if err != nil {
		return "", err
	}
	return "/" + stripped[0], nil
}

// packageClientsRequestDocument is data only: Rust owns naming, layout, ownership
// of generated files and the Cargo manifest edit.
func packageClientsRequestDocument(
	descriptor metadata.EngineSource,
	clientRoot string,
	clients []packageClient,
) map[string]any {
	documents := make([]any, 0, len(clients))
	for _, client := range clients {
		document := map[string]any{
			"name":    client.Name,
			"address": client.Address,
			"schema": map[string]any{
				"path":   client.SchemaPath,
				"digest": client.SchemaDigest,
			},
		}
		if client.Pin != "" {
			document["pin"] = client.Pin
		}
		documents = append(documents, document)
	}
	return map[string]any{
		"request_kind": "generate-package-clients",
		"request": map[string]any{
			"format_version": 1,
			"target":         targetIdentity(descriptor),
			"sdk_dependency": descriptor.SDKDependency,
			"client_root":    clientRoot,
			"clients":        documents,
		},
	}
}

// initializedModuleScope scaffolds a new Rust module in the scope and writes its
// module configuration, which loads the runtime from the given reference. The
// SDK-module interface leaves module configuration to the SDK, and the Rust
// initialization plan refuses to write it, so the adapter authors it.
func (sdk *RustSDK) initializedModuleScope(
	ctx context.Context,
	ws *dagger.Workspace,
	scope string,
	name string,
	runtime string,
) (*dagger.Workspace, error) {
	scaffold, err := sdk.moduleScaffold(ctx, ws, scope, name)
	if err != nil {
		return nil, err
	}
	descriptor, err := sdk.descriptorMetadata(ctx)
	if err != nil {
		return nil, err
	}
	manifest := moduleManifest(name, descriptor.EngineVersion, runtime)
	return ws.WithDirectory(".", scaffold).
		WithNewFile("dagger-module.toml", manifest, dagger.WorkspaceWithNewFileOpts{Permissions: 0o644}), nil
}

// moduleScaffold returns the scope directory after the Rust initialization plan.
// The plan sees the whole workspace so it can adopt an existing Cargo project, but
// it writes only beneath the module root.
func (sdk *RustSDK) moduleScaffold(
	ctx context.Context,
	ws *dagger.Workspace,
	scope string,
	name string,
) (*dagger.Directory, error) {
	moduleRoot, err := rebasePath(scope)
	if err != nil {
		return nil, fmt.Errorf("module path: %w", err)
	}
	descriptor, err := sdk.descriptorMetadata(ctx)
	if err != nil {
		return nil, err
	}
	request := map[string]any{
		"request_kind": "initialize-module",
		"request": map[string]any{
			"format_version": 1,
			"target":         targetIdentity(descriptor),
			"module": map[string]any{
				"name":           name,
				"original_name":  name,
				"source_subpath": moduleRoot,
				"config_format":  "current",
				"source_digest":  metadata.DigestModuleSource("initialization:" + scope),
			},
			"package_name":   name,
			"sdk_dependency": descriptor.SDKDependency,
		},
	}
	execution, err := sdk.executeOperation(ctx, request, nil, nil, ws.Directory("/"), "initialization")
	if err != nil {
		return nil, err
	}
	initialized := execution.project.Directory(operationWorkspaceRoot)
	if scope == "." {
		return initialized, nil
	}
	return initialized.Directory(scope), nil
}

// installedRuntimeSource names the reference this SDK module was installed from:
// the runtime a new module loads, and the SDK release or commit a Cargo package's
// clients depend on. A module would resolve a local path against its own
// directory, and a local path names no release or commit, so it is refused.
func installedRuntimeSource(ctx context.Context, ws *dagger.Workspace) (string, error) {
	moduleName, err := dag.CurrentModule().Name(ctx)
	if err != nil {
		return "", fmt.Errorf("read Rust SDK module name: %w", err)
	}
	source, err := ws.Module(moduleName).Source(ctx)
	if err != nil {
		return "", fmt.Errorf("read the workspace installation of %q: %w", moduleName, err)
	}
	if !isRemoteModuleReference(source) {
		return "", fmt.Errorf("Rust SDK %q is installed from a local path; install it from a Git reference to initialize modules or generate clients", moduleName)
	}
	return source, nil
}

// moduleRuntimeSource reads the runtime reference in a module's configuration,
// which the engine resolved to load this runtime for that module.
func moduleRuntimeSource(modSource *dagger.ModuleSource) runtimeReference {
	return func(ctx context.Context) (string, error) {
		config, err := modSource.SDK(ctx)
		if err != nil {
			return "", fmt.Errorf("read the module's Rust SDK runtime: %w", err)
		}
		if config == nil {
			return "", fmt.Errorf("the module configuration names no Rust SDK runtime")
		}
		source, err := config.Source(ctx)
		if err != nil {
			return "", fmt.Errorf("read the module's Rust SDK runtime: %w", err)
		}
		return source, nil
	}
}

// isRemoteModuleReference follows the engine's classification of a module source
// (core/gitref.FastKindCheck and core/workspace.IsLocalRef @
// 400b82d6f3c09ef096d2f59b3e920267b5969bf4): a leading path marker or a dot-free
// source is a workspace path, and a Git scheme or a host ahead of the first
// separator names a remote reference.
func isRemoteModuleReference(source string) bool {
	switch {
	case strings.HasPrefix(source, "/"), strings.HasPrefix(source, "."):
		return false
	case strings.HasPrefix(source, "http://"), strings.HasPrefix(source, "https://"),
		strings.HasPrefix(source, "git://"), strings.HasPrefix(source, "ssh://"),
		strings.HasPrefix(source, "entire://"):
		return true
	case !strings.Contains(source, "."):
		return false
	}
	host, _, _ := strings.Cut(strings.ReplaceAll(source, `\`, "/"), "/")
	if _, afterUser, scpLike := strings.Cut(host, "@"); scpLike {
		host = afterUser
	}
	if strings.Contains(host, ".") || strings.Contains(host, ":") {
		return true
	}
	// Azure DevOps Server references name a dot-free host with a _git segment.
	return strings.Contains(source, "/_git/")
}

// moduleManifest renders the configuration of a new Rust module in the layout
// Dagger's manifest helper writes, without indentation (tomlContents in main.dang
// of github.com/dagger/sdk-helpers at v1.0.12). The engine version is
// the SDK's target release rather than the running engine's: a module is served
// the API view its engineVersion names, and the Rust bindings are compiled
// against the target's view, so a module created on a later engine still gets
// the view the SDK was built for.
func moduleManifest(name string, targetRelease string, runtime string) string {
	return "name = " + tomlString(name) + "\n" +
		"engineVersion = " + tomlString("v"+targetRelease) + "\n\n" +
		"[runtime]\n" +
		"source = " + tomlString(runtime) + "\n"
}

// tomlString encodes a TOML basic string, escaping every character the format
// does not admit literally.
func tomlString(value string) string {
	var encoded strings.Builder
	encoded.WriteByte('"')
	for _, character := range value {
		switch {
		case character == '"':
			encoded.WriteString(`\"`)
		case character == '\\':
			encoded.WriteString(`\\`)
		case character < 0x20 || character == 0x7f:
			fmt.Fprintf(&encoded, `\u%04X`, character)
		default:
			encoded.WriteRune(character)
		}
	}
	encoded.WriteByte('"')
	return encoded.String()
}

// generatedModuleCode returns the scope directory after the engine's module code
// generation, without the module configuration this SDK owns in the scope.
func generatedModuleCode(ctx context.Context, ws *dagger.Workspace, scope string) (*dagger.Directory, error) {
	sourcePath := "/"
	if scope != "." {
		sourcePath = "/" + scope
	}
	generated := ws.WithWorkdir(".").ModuleSource(sourcePath).GeneratedContextDirectory()
	if scope != "." {
		generated = generated.Directory(scope)
	}
	for _, manifest := range []string{"dagger-module.toml", "dagger.json"} {
		present, err := generated.Exists(ctx, manifest)
		if err != nil {
			return nil, fmt.Errorf("inspect generated module scope: %w", err)
		}
		if present {
			generated = generated.WithoutFile(manifest)
		}
	}
	return generated, nil
}

func workspaceHasFile(ctx context.Context, ws *dagger.Workspace, name string) (bool, error) {
	present, err := ws.Directory(".", dagger.WorkspaceDirectoryOpts{Include: []string{name}}).Exists(ctx, name)
	if err != nil {
		return false, fmt.Errorf("inspect workspace scope for %s: %w", name, err)
	}
	return present, nil
}

func workspaceHasAnyFile(ctx context.Context, ws *dagger.Workspace, names ...string) (bool, error) {
	for _, name := range names {
		present, err := workspaceHasFile(ctx, ws, name)
		if err != nil || present {
			return present, err
		}
	}
	return false, nil
}

func workspaceCwd(ctx context.Context, ws *dagger.Workspace) (string, error) {
	if ws == nil {
		return "", fmt.Errorf("workspace not provided")
	}
	cwd, err := ws.Cwd(ctx)
	if err != nil {
		return "", fmt.Errorf("read workspace cwd: %w", err)
	}
	return normalizeWorkspacePath(cwd), nil
}

func normalizeWorkspacePath(candidate string) string {
	trimmed := strings.Trim(candidate, "/")
	if trimmed == "" {
		return "."
	}
	return path.Clean(trimmed)
}

func isConfinedWorkspacePath(candidate string) bool {
	if path.IsAbs(candidate) {
		return false
	}
	normalized := normalizeWorkspacePath(candidate)
	return normalized != ".." && !strings.HasPrefix(normalized, "../") && !strings.Contains(candidate, "\\")
}

// engineDescriptor returns the opaque Rust-owned descriptor. Go forwards these bytes
// to the packaged tool and deliberately does not reinterpret dependency provenance.
func (sdk *RustSDK) engineDescriptor() *dagger.File {
	return sdk.SDKSourceDir.File(engineDescriptorPath)
}

func (sdk *RustSDK) engineTool() *dagger.File {
	return sdk.SDKSourceDir.File(engineToolPath)
}

// Codegen compiles the engine-visible schema into the scoped module context and
// returns the Rust-owned VCS policy emitted by the same operation plan. The engine
// writes that policy into the module's own .gitattributes and .gitignore, so its
// paths are relative to the module source root rather than the context.
func (sdk *RustSDK) Codegen(
	ctx context.Context,
	modSource *dagger.ModuleSource,
	introspectionJSON *dagger.File,
) (*dagger.GeneratedCode, error) {
	sdk, resolveErr := sdk.resolved(ctx, moduleRuntimeSource(modSource))
	if resolveErr != nil {
		return nil, resolveErr
	}
	if introspectionJSON == nil {
		return nil, fmt.Errorf("introspection JSON not provided")
	}
	identity, err := moduleIdentity(ctx, modSource, "current")
	if err != nil {
		return nil, err
	}
	request, _, err := sdk.generationRequest(ctx, "generate-module", identity, introspectionJSON, identity.SourceSubpath)
	if err != nil {
		return nil, err
	}
	execution, err := sdk.executeOperation(ctx, request, introspectionJSON, nil, modSource.ContextDirectory(), "generation")
	if err != nil {
		return nil, err
	}
	generated, err := metadata.StripOperationRoot(execution.result.VCSGenerated, identity.SourceSubpath)
	if err != nil {
		return nil, err
	}
	ignored, err := metadata.StripOperationRoot(execution.result.VCSIgnored, identity.SourceSubpath)
	if err != nil {
		return nil, err
	}
	code, err := withRemovedModuleCode(ctx, execution.project.Directory(operationWorkspaceRoot), modSource, execution.result.RemovedPaths)
	if err != nil {
		return nil, err
	}
	return dag.GeneratedCode(code).
		WithVCSGeneratedPaths(generated).
		WithVCSIgnoredPaths(ignored), nil
}

type scopedModuleIdentity struct {
	Name          string
	OriginalName  string
	SourceSubpath string
	SourceDigest  string
	ConfigFormat  string
	ResolvedPin   string
}

type operationExecution struct {
	project *dagger.Directory
	result  metadata.ExecutionResult
}

// executeOperation runs one request of the packaged Rust tool over a private copy
// of project. An operation on one module is given that module's schema; module
// client generation is given a directory holding one schema for each client.
func (sdk *RustSDK) executeOperation(
	ctx context.Context,
	request any,
	schema *dagger.File,
	schemas *dagger.Directory,
	project *dagger.Directory,
	expectedKind string,
) (operationExecution, error) {
	requestBytes, err := metadata.CanonicalJSON(request)
	if err != nil {
		return operationExecution{}, fmt.Errorf("encode Rust operation request: %w", err)
	}
	policy, err := sdk.runtimePolicyMetadata(ctx)
	if err != nil {
		return operationExecution{}, err
	}
	descriptor, err := sdk.descriptorMetadata(ctx)
	if err != nil {
		return operationExecution{}, err
	}
	rustTarget, err := runtimeTarget(ctx, policy)
	if err != nil {
		return operationExecution{}, err
	}
	formatterInstallPath := path.Join(
		"/usr/local/rustup/toolchains",
		descriptor.RustToolchain+"-"+rustTarget,
		"bin/rustfmt",
	)
	container := dag.Container().From(policy.BuildImage).
		WithFile(operationToolPath, sdk.engineTool(), dagger.ContainerWithFileOpts{Permissions: 0o755}).
		// rustfmt resolves private toolchain libraries relative to its executable.
		// Retaining the rustup layout keeps that lookup valid without an ambient proxy.
		WithFile(formatterInstallPath, sdk.SDKSourceDir.File(formatterToolPath), dagger.ContainerWithFileOpts{Permissions: 0o755}).
		WithFile(operationDescriptorPath, sdk.engineDescriptor()).
		WithDirectory(operationProjectPath, dag.Directory().WithDirectory(operationWorkspaceRoot, project)).
		WithNewFile(operationRequestPath, string(requestBytes))
	args := []string{
		operationToolPath, "execute",
		"--request", operationRequestPath,
		"--descriptor", operationDescriptorPath,
		"--project", operationProjectPath,
		"--result", operationResultPath,
	}
	if schema != nil {
		container = container.WithFile(operationSchemaPath, schema)
		args = append(args, "--schema", operationSchemaPath)
	}
	if schemas != nil {
		container = container.WithDirectory(operationSchemasPath, schemas)
		args = append(args, "--schemas", operationSchemasPath)
	}
	executed, err := container.WithExec(args).Sync(ctx)
	if err != nil {
		return operationExecution{}, fmt.Errorf("execute Rust SDK operation: %w", err)
	}
	resultBytes, err := executed.File(operationResultPath).Contents(ctx)
	if err != nil {
		return operationExecution{}, fmt.Errorf("read Rust SDK operation result: %w", err)
	}
	result, err := metadata.DecodeExecutionResult([]byte(resultBytes), expectedKind)
	if err != nil {
		return operationExecution{}, err
	}
	return operationExecution{project: executed.Directory(operationProjectPath), result: result}, nil
}

func (sdk *RustSDK) generationRequest(
	ctx context.Context,
	operation string,
	module scopedModuleIdentity,
	schema *dagger.File,
	outputRoot string,
) (map[string]any, []byte, error) {
	descriptor, err := sdk.descriptorMetadata(ctx)
	if err != nil {
		return nil, nil, err
	}
	contents, err := schema.Contents(ctx)
	if err != nil {
		return nil, nil, fmt.Errorf("read visible schema: %w", err)
	}
	schemaBytes := []byte(contents)
	// The engine's stated release only selects the advice Rust gives when it refuses
	// the visible schema, so a release that cannot be read or is not shaped like one
	// is left out rather than failing generation.
	engineRelease, err := dag.Version(ctx)
	if err != nil || !admissibleEngineRelease(engineRelease) {
		engineRelease = ""
	}
	return generationRequestDocument(descriptor, operation, module, schemaBytes, outputRoot, engineRelease), schemaBytes, nil
}

// maxEngineReleaseLength bounds the stated release Rust accepts in a request.
const maxEngineReleaseLength = 128

// admissibleEngineRelease reports whether a stated engine release has the shape
// the Rust request model accepts: ASCII letters, digits, '.', '+' and '-'.
func admissibleEngineRelease(release string) bool {
	if release == "" || len(release) > maxEngineReleaseLength {
		return false
	}
	for _, character := range []byte(release) {
		switch {
		case character >= 'a' && character <= 'z',
			character >= 'A' && character <= 'Z',
			character >= '0' && character <= '9',
			character == '.', character == '+', character == '-':
		default:
			return false
		}
	}
	return true
}

// generationRequestDocument is deliberately data-only: Rust owns source discovery,
// descriptor construction, registration, codecs, dispatch, and evidence semantics.
func generationRequestDocument(
	descriptor metadata.EngineSource,
	operation string,
	module scopedModuleIdentity,
	schemaBytes []byte,
	outputRoot string,
	engineRelease string,
) map[string]any {
	moduleDocument := map[string]any{
		"name":           module.Name,
		"original_name":  module.OriginalName,
		"source_subpath": module.SourceSubpath,
		"config_format":  module.ConfigFormat,
		"source_digest":  module.SourceDigest,
	}
	if module.ResolvedPin != "" {
		moduleDocument["resolved_pin"] = module.ResolvedPin
	}
	request := map[string]any{
		"format_version": 1,
		"operation":      operation,
		"target":         targetIdentity(descriptor),
		"visible_schema": map[string]any{
			"path":   "schema.json",
			"digest": metadata.DigestBytes(schemaBytes),
		},
		"module":         moduleDocument,
		"sdk_dependency": descriptor.SDKDependency,
		"output_root":    outputRoot,
	}
	if engineRelease != "" {
		request["engine_version"] = engineRelease
	}
	return map[string]any{
		"request_kind": "generate",
		"request":      request,
	}
}

func (sdk *RustSDK) descriptorMetadata(ctx context.Context) (metadata.EngineSource, error) {
	contents, err := sdk.engineDescriptor().Contents(ctx)
	if err != nil {
		return metadata.EngineSource{}, fmt.Errorf("read packaged engine descriptor: %w", err)
	}
	return metadata.DecodeEngineSource([]byte(contents))
}

func (sdk *RustSDK) runtimePolicyMetadata(ctx context.Context) (metadata.RuntimePolicy, error) {
	contents, err := sdk.SDKSourceDir.File(runtimePolicyPath).Contents(ctx)
	if err != nil {
		return metadata.RuntimePolicy{}, fmt.Errorf("read packaged runtime policy: %w", err)
	}
	return metadata.DecodeRuntimePolicy([]byte(contents))
}

func targetIdentity(descriptor metadata.EngineSource) map[string]any {
	return map[string]any{
		"format_version":     descriptor.FormatVersion,
		"repository":         descriptor.Repository,
		"dagger_revision":    descriptor.DaggerRevision,
		"engine_version":     descriptor.EngineVersion,
		"rust_sdk_version":   descriptor.RustSDKVersion,
		"rust_toolchain":     descriptor.RustToolchain,
		"core_schema_digest": descriptor.CoreSchemaDigest,
	}
}

func moduleIdentity(ctx context.Context, source *dagger.ModuleSource, configFormat string) (scopedModuleIdentity, error) {
	if source == nil {
		return scopedModuleIdentity{}, fmt.Errorf("module source not provided")
	}
	name, err := source.ModuleName(ctx)
	if err != nil {
		return scopedModuleIdentity{}, fmt.Errorf("read module name: %w", err)
	}
	originalName, err := source.ModuleOriginalName(ctx)
	if err != nil {
		return scopedModuleIdentity{}, fmt.Errorf("read original module name: %w", err)
	}
	subpath, err := source.SourceSubpath(ctx)
	if err != nil {
		return scopedModuleIdentity{}, fmt.Errorf("read module source subpath: %w", err)
	}
	rebased, err := rebasePath(subpath)
	if err != nil {
		return scopedModuleIdentity{}, fmt.Errorf("module source subpath: %w", err)
	}
	// Dagger's directory digest retains empty parent directories created solely by
	// generated paths. Hashing a canonical leaf-file inventory keeps every authored
	// input while preventing that incidental scaffolding from making a fresh manifest
	// stale. The operation manifest separately authenticates the excluded products.
	digest, err := semanticModuleSourceDigest(ctx, source, subpath)
	if err != nil {
		return scopedModuleIdentity{}, fmt.Errorf("read semantic module source digest: %w", err)
	}
	kind, err := source.Kind(ctx)
	if err != nil {
		return scopedModuleIdentity{}, fmt.Errorf("read module source kind: %w", err)
	}
	requiresPin, err := moduleSourceRequiresPin(kind)
	if err != nil {
		return scopedModuleIdentity{}, err
	}
	resolvedPin := ""
	if requiresPin {
		resolvedPin, err = source.Pin(ctx)
		if err != nil {
			return scopedModuleIdentity{}, fmt.Errorf("read resolved module pin: %w", err)
		}
		if err := validateOptionalRevision(resolvedPin); err != nil || resolvedPin == "" {
			return scopedModuleIdentity{}, fmt.Errorf("resolved remote module pin is invalid")
		}
	}
	return scopedModuleIdentity{
		Name: name, OriginalName: originalName, SourceSubpath: rebased,
		SourceDigest: digest, ConfigFormat: configFormat, ResolvedPin: resolvedPin,
	}, nil
}

// Directory sources are immutable Dagger snapshots and local sources are bound by
// the semantic leaf digest above. Only Git sources need a separately authenticated
// resolved commit in the operation request.
func moduleSourceRequiresPin(kind dagger.ModuleSourceKind) (bool, error) {
	switch kind {
	case dagger.ModuleSourceKindGit:
		return true, nil
	case dagger.ModuleSourceKindLocal, dagger.ModuleSourceKindDir:
		return false, nil
	default:
		return false, fmt.Errorf("module source kind is unsupported")
	}
}

func validateOptionalRevision(value string) error {
	if value == "" {
		return nil
	}
	if len(value) != 40 {
		return fmt.Errorf("revision must be a full lowercase commit")
	}
	for _, character := range value {
		if (character < '0' || character > '9') && (character < 'a' || character > 'f') {
			return fmt.Errorf("revision must be a full lowercase commit")
		}
	}
	return nil
}

func semanticModuleSourceDigest(
	ctx context.Context,
	source *dagger.ModuleSource,
	subpath string,
) (string, error) {
	// The engine writes its canonical config representation after codegen. Overlaying
	// that representation before enumeration keeps config semantics in the identity
	// without coupling the manifest to pre- versus post-write serialization.
	normalizedContext := source.ContextDirectory().WithDirectory("/", source.UpdatedConfigDirectory())
	semantic := normalizedContext.Directory(subpath).Filter(dagger.DirectoryFilterOpts{
		Exclude: []string{
			".dagger/rust", ".dagger/rust/**",
			"src/dagger_generated", "src/dagger_generated/**",
			"src/bin/dagger-module.rs",
			".gitattributes", ".gitignore",
		},
	})
	paths, err := semantic.Glob(ctx, "**")
	if err != nil {
		return "", fmt.Errorf("enumerate semantic module source: %w", err)
	}
	sort.Strings(paths)
	files := make([]metadata.ModuleSourceFile, 0, len(paths))
	for _, candidate := range paths {
		if strings.HasSuffix(candidate, "/") {
			continue
		}
		digest, err := semantic.File(candidate).Digest(ctx, dagger.FileDigestOpts{ExcludeMetadata: true})
		if err != nil {
			return "", fmt.Errorf("digest semantic module source path %s: %w", candidate, err)
		}
		files = append(files, metadata.ModuleSourceFile{Path: candidate, Digest: digest})
	}
	return metadata.DigestModuleSourceFiles(files)
}

// The private operation root always has a named first component because Rust rejects
// an empty capability path. Rebasing does not change caller-visible result paths.
func rebasePath(candidate string) (string, error) {
	return metadata.RebaseOperationPath(candidate)
}
