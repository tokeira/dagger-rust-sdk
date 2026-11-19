package main

import (
	"context"
	"fmt"
	"strings"

	"rust-sdk/internal/dagger"
)

// boundGitClient reads the pin of a Git client and binds the client as Dagger's Go
// SDK binds a unified client (workspaceModule and its use in
// cmd/dagger-go-sdk-codegen/main.go of github.com/dagger/go-sdk at bb23e8d). A
// module of the Git workspace's own repository, at the workspace's commit and
// selector, is bound as a local module, by its path and with no pin: its address
// would name that commit, which every commit changes. Any other Git client is bound
// by its reference and pin. The facts are the ones Go reads: the workspace's
// address, the commit of its Git head, and the module's clone URL.
func boundGitClient(
	ctx context.Context,
	ws *dagger.Workspace,
	client *dagger.ModuleSource,
	name string,
	reference string,
) (dagger.ModuleSourceKind, string, string, error) {
	pin, err := client.Pin(ctx)
	if err != nil {
		return "", "", "", fmt.Errorf("read the pin of module %q: %w", name, err)
	}
	cloneRef, err := client.CloneRef(ctx)
	if err != nil {
		return "", "", "", fmt.Errorf("read the repository of module %q: %w", name, err)
	}
	address, err := ws.Address(ctx)
	if err != nil {
		return "", "", "", fmt.Errorf("read the workspace address: %w", err)
	}
	// Go reads the commit of a workspace that has no Git head as empty, which no
	// pin equals.
	commit, err := ws.Git().Head().Commit(ctx)
	if err != nil {
		commit = ""
	}
	if !workspaceModule(dagger.ModuleSourceKindGit, reference, pin, cloneRef, address, commit) {
		return dagger.ModuleSourceKindGit, pin, "", nil
	}
	subpath, err := client.SourceRootSubpath(ctx)
	if err != nil {
		return "", "", "", fmt.Errorf("read the path of module %q: %w", name, err)
	}
	return dagger.ModuleSourceKindLocal, "", subpath, nil
}

// workspaceModule reports whether a Git module is part of the Git workspace that
// generates its client: in its repository, at its commit and selector. A workspace
// without a selector resolves its modules at the default branch, which its address
// does not name, so any selector at its commit matches. It is workspaceModule of
// cmd/dagger-go-sdk-codegen/main.go in github.com/dagger/go-sdk at bb23e8d, over
// the same facts.
func workspaceModule(
	kind dagger.ModuleSourceKind,
	reference, pin, cloneRef, workspaceAddress, workspaceCommit string,
) bool {
	if kind != dagger.ModuleSourceKindGit || cloneRef == "" || pin == "" || pin != workspaceCommit {
		return false
	}
	workspaceSelector, ok := gitSelector(workspaceAddress, cloneRef)
	if !ok {
		return false
	}
	moduleSelector, ok := gitSelector(reference, cloneRef)
	return ok && (workspaceSelector == "" || workspaceSelector == moduleSelector)
}

// gitSelector returns the version of a ref of the repository at cloneRef, or false
// for a ref of another repository. The ref is either in the legacy form
// (repo/subpath@version) or, since dagger v1.0.0-beta.20, in the explicit Git URL
// form (repo#version:subpath). It is Go's gitSelector, as workspaceModule is.
func gitSelector(ref, cloneRef string) (string, bool) {
	rest, ok := strings.CutPrefix(trimURLScheme(ref), trimURLScheme(cloneRef))
	if !ok || (rest != "" && rest[0] != '/' && rest[0] != '@' && rest[0] != '#') {
		return "", false
	}
	if version, explicit := strings.CutPrefix(rest, "#"); explicit {
		version, _, _ = strings.Cut(version, ":")
		return version, true
	}
	_, selector, _ := strings.Cut(rest, "@")
	return selector, true
}

// trimURLScheme drops a URL scheme: a GitHub workspace address has none, while the
// clone URL of its modules has https://.
func trimURLScheme(ref string) string {
	if _, rest, ok := strings.Cut(ref, "://"); ok {
		return rest
	}
	return ref
}
