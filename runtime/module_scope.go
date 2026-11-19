package main

import (
	"context"
	"fmt"
	"strings"

	"rust-sdk/internal/dagger"
	"rust-sdk/internal/metadata"
)

const (
	moduleManifestName       = "dagger-module.toml"
	legacyModuleManifestName = "dagger.json"
)

// moduleClient is one client of a module scope as the adapter resolved it: the
// target's original name, and either its path in the workspace or its Git
// reference. Rust turns the list into the module's dependencies.
type moduleClient struct {
	Name              string
	Local             bool
	SourceRootSubpath string
	Reference         string
}

// scopeModuleSource loads the module in a scope from the workspace's own files.
func scopeModuleSource(ws *dagger.Workspace, scope string) *dagger.ModuleSource {
	sourcePath := "/"
	if scope != "." {
		sourcePath = "/" + scope
	}
	return ws.WithWorkdir(".").ModuleSource(sourcePath)
}

// moduleScopeDependencies returns the workspace with the scope's module
// configuration rewritten so that its dependencies are exactly the clients recorded
// for the scope. The list is complete: with no clients, every dependency goes.
// Dagger's Go SDK does the same for a module it generates (generatedManifest in
// go-sdk.dang of github.com/dagger/go-sdk at bb23e8d), through
// withLegacyRuntimeDependencies of github.com/dagger/sdk-helpers at v1.0.12. The
// rules are Rust's; the adapter resolves each client and applies the file Rust
// returns.
func (sdk *RustSDK) moduleScopeDependencies(
	ctx context.Context,
	ws *dagger.Workspace,
	scope string,
	clients []*dagger.ModuleSource,
) (*dagger.Workspace, error) {
	resolved := make([]moduleClient, 0, len(clients))
	for _, client := range clients {
		next, err := resolvedModuleClient(ctx, ws, client)
		if err != nil {
			return nil, err
		}
		resolved = append(resolved, next)
	}
	workspaceRoot, err := rebasePath(".")
	if err != nil {
		return nil, fmt.Errorf("workspace path: %w", err)
	}
	moduleRoot, err := rebasePath(scope)
	if err != nil {
		return nil, fmt.Errorf("module path: %w", err)
	}
	// The current directory shows files that exist only as pending workspace edits,
	// which a newly scaffolded module's configuration is.
	manifests := ws.Directory(".", dagger.WorkspaceDirectoryOpts{
		Include: []string{moduleManifestName, legacyModuleManifestName},
	})
	project := manifests
	if scope != "." {
		project = dag.Directory().WithDirectory(scope, manifests)
	}
	execution, err := sdk.executeOperation(
		ctx,
		moduleDependenciesRequestDocument(workspaceRoot, moduleRoot, resolved),
		nil,
		nil,
		project,
		"module-dependencies",
	)
	if err != nil {
		return nil, err
	}
	written, changed, err := metadata.WrittenModuleManifest(execution.result, moduleRoot)
	if err != nil {
		return nil, err
	}
	if !changed {
		return ws, nil
	}
	// Workspace.withFile cannot take permissions on the target release: the engine
	// fails to pass them on to the directory (v1.0.0-beta.27). The contents go
	// through withNewFile, as a new module's configuration does.
	contents, err := execution.project.File(written).Contents(ctx)
	if err != nil {
		return nil, fmt.Errorf("read rewritten module configuration: %w", err)
	}
	return ws.WithNewFile(moduleManifestName, contents, dagger.WorkspaceWithNewFileOpts{Permissions: 0o644}), nil
}

// resolvedModuleClient reads what Rust needs of one client. It reads the client's
// own source only: nothing here loads the client's schema or sets its engine
// version, so the dependency is the module at the version it declares.
func resolvedModuleClient(
	ctx context.Context,
	ws *dagger.Workspace,
	client *dagger.ModuleSource,
) (moduleClient, error) {
	name, err := client.ModuleOriginalName(ctx)
	if err != nil {
		return moduleClient{}, fmt.Errorf("read client module name: %w", err)
	}
	kind, err := client.Kind(ctx)
	if err != nil {
		return moduleClient{}, fmt.Errorf("read client %q source kind: %w", name, err)
	}
	subpath, err := client.SourceRootSubpath(ctx)
	if err != nil {
		return moduleClient{}, fmt.Errorf("read client %q source path: %w", name, err)
	}
	reference := ""
	if kind == dagger.ModuleSourceKindGit {
		reference, err = client.AsString(ctx)
		if err != nil {
			return moduleClient{}, fmt.Errorf("read client %q reference: %w", name, err)
		}
	}
	local := clientInWorkspace(kind, reference, func() (string, error) {
		return ws.ModuleSource(workspaceModulePath(subpath)).AsString(ctx)
	})
	if local {
		return moduleClient{Name: name, Local: true, SourceRootSubpath: subpath}, nil
	}
	return moduleClient{Name: name, Reference: reference}, nil
}

// clientInWorkspace reports whether a client is written as a path. It follows
// inWorkspace of github.com/dagger/sdk-helpers at v1.0.12: a client that is not a
// Git source is in the workspace, and so is a Git source that the workspace
// resolves, at the client's path, to the same reference. A workspace that is
// itself a Git checkout loads its own modules as Git sources, and a pinned
// reference to one would change with every commit. A lookup that fails counts as
// a different module.
func clientInWorkspace(
	kind dagger.ModuleSourceKind,
	reference string,
	workspaceReference func() (string, error),
) bool {
	if kind != dagger.ModuleSourceKindGit {
		return true
	}
	resolved, err := workspaceReference()
	return err == nil && resolved == reference
}

// workspaceModulePath spells a client's source root as a path from the workspace
// root, as inWorkspace does with the helper's normalizePath.
func workspaceModulePath(sourceRootSubpath string) string {
	trimmed := strings.TrimSuffix(strings.TrimPrefix(strings.TrimPrefix(sourceRootSubpath, "/"), "./"), "/")
	if trimmed == "" {
		trimmed = "."
	}
	return "/" + trimmed
}

// moduleDependenciesRequestDocument is data only: Rust owns the dependency rules,
// from the relative path of a local client to what happens to the dependencies
// already written.
func moduleDependenciesRequestDocument(
	workspaceRoot string,
	moduleRoot string,
	clients []moduleClient,
) map[string]any {
	documents := make([]any, 0, len(clients))
	for _, client := range clients {
		if client.Local {
			documents = append(documents, map[string]any{
				"kind":                "local",
				"name":                client.Name,
				"source_root_subpath": client.SourceRootSubpath,
			})
			continue
		}
		documents = append(documents, map[string]any{
			"kind":   "git",
			"name":   client.Name,
			"source": client.Reference,
		})
	}
	return map[string]any{
		"request_kind": "write-module-dependencies",
		"request": map[string]any{
			"format_version": 1,
			"workspace_root": workspaceRoot,
			"module_root":    moduleRoot,
			"clients":        documents,
		},
	}
}
