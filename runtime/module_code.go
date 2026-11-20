package main

import (
	"context"
	"encoding/json"
	"fmt"
	"path"
	"strings"

	"rust-sdk/internal/dagger"
)

// removedModuleCodePath is where Codegen lists the files a module's generation
// removed, from the module's source root. The list exists only between Codegen and
// GenerateScope, which takes it out of the generated code before writing that code
// into the workspace.
const removedModuleCodePath = ".dagger/rust/removed-paths.json"

// withRemovedModuleCode returns a module's generated code with the list of the files
// its generation removed, when it removed any. code is the module's context directory
// as generation left it, and removed the operation's removed paths.
//
// Rust removes a file it no longer generates only while the file has the bytes the
// operation manifest records (verify_ownership in
// crates/dagger-sdk-engine/src/publication.rs), and it does so in the directory the
// engine gives code generation. The engine hands GenerateScope that code as the files
// that differ from the module, which cannot say that a file has gone, so the list
// carries each removal.
func withRemovedModuleCode(
	ctx context.Context,
	code *dagger.Directory,
	source *dagger.ModuleSource,
	removed []string,
) (*dagger.Directory, error) {
	if len(removed) == 0 {
		return code, nil
	}
	root, err := source.SourceRootSubpath(ctx)
	if err != nil {
		return nil, fmt.Errorf("read module source root: %w", err)
	}
	operationRoot, err := rebasePath(root)
	if err != nil {
		return nil, fmt.Errorf("module source root: %w", err)
	}
	list, err := encodeRemovedModuleCode(removed, operationRoot)
	if err != nil {
		return nil, err
	}
	return code.WithNewFile(path.Join(root, removedModuleCodePath), list), nil
}

// encodeRemovedModuleCode writes the list of the operation's removed paths from the
// module's source root, given as an operation path. Rust removes only files it
// generated in the module, so a path elsewhere is refused.
func encodeRemovedModuleCode(removed []string, root string) (string, error) {
	relative := make([]string, 0, len(removed))
	for _, file := range removed {
		value, ok := strings.CutPrefix(file, root+"/")
		if !ok || value == "" {
			return "", fmt.Errorf("the Rust operation removed %q, outside the module", file)
		}
		relative = append(relative, value)
	}
	encoded, err := json.Marshal(relative)
	if err != nil {
		return "", fmt.Errorf("encode the files the Rust operation removed: %w", err)
	}
	return string(encoded), nil
}

// withGeneratedModuleCode returns the workspace with a module scope's generated code
// written into it, and without the files that generation removed. The current
// directory is the scope, which is the module's source root.
//
// Workspace.withDirectory merges and never deletes, so the removals come from the
// list that Codegen leaves in the generated code. The adapter decides nothing about a
// file's bytes: Rust kept each file its author edited and left it out of the list.
func withGeneratedModuleCode(
	ctx context.Context,
	ws *dagger.Workspace,
	generated *dagger.Directory,
) (*dagger.Workspace, error) {
	list, err := directoryFile(ctx, generated, removedModuleCodePath)
	if err != nil {
		return nil, err
	}
	removed, err := decodeRemovedModuleCode(list)
	if err != nil {
		return nil, err
	}
	if list != nil {
		generated = generated.WithoutFile(removedModuleCodePath)
	}
	updated := ws.WithDirectory(".", generated)
	for _, file := range removed {
		updated = updated.WithoutFile(file)
	}
	return updated, nil
}

// directoryFile returns the bytes of a file of a directory, or nil when the
// directory has no such file.
func directoryFile(ctx context.Context, directory *dagger.Directory, name string) ([]byte, error) {
	present, err := directory.Exists(ctx, name)
	if err != nil {
		return nil, fmt.Errorf("inspect %s: %w", name, err)
	}
	if !present {
		return nil, nil
	}
	contents, err := directory.File(name).Contents(ctx)
	if err != nil {
		return nil, fmt.Errorf("read %s: %w", name, err)
	}
	return []byte(contents), nil
}

// decodeRemovedModuleCode reads the list Codegen leaves, nil when there is none.
// Codegen writes a list only when generation removed a file, so a list that names no
// file, that is not a list of paths inside the module, or that names itself is
// refused rather than passed over, and a damaged list removes nothing.
func decodeRemovedModuleCode(list []byte) ([]string, error) {
	if list == nil {
		return nil, nil
	}
	var removed []string
	if err := json.Unmarshal(list, &removed); err != nil {
		return nil, fmt.Errorf("decode %s: %w", removedModuleCodePath, err)
	}
	if len(removed) == 0 {
		return nil, fmt.Errorf("%s names no file", removedModuleCodePath)
	}
	for _, file := range removed {
		if file == "" || file == "." || !isConfinedWorkspacePath(file) || path.Clean(file) != file {
			return nil, fmt.Errorf("%s names %q, which is not a normalized path inside the module", removedModuleCodePath, file)
		}
		if file == removedModuleCodePath {
			return nil, fmt.Errorf("%s names itself", removedModuleCodePath)
		}
	}
	return removed, nil
}
