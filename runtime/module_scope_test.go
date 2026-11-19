package main

import (
	"fmt"
	"go/ast"
	"go/parser"
	"go/token"
	"os"
	"path/filepath"
	"strings"
	"testing"

	"rust-sdk/internal/dagger"
	"rust-sdk/internal/metadata"
)

func TestModuleDependenciesRequestCarriesOnlyResolvedClients(t *testing.T) {
	request := moduleDependenciesRequestDocument(
		"workspace",
		"workspace/.dagger/modules/gate",
		[]moduleClient{
			{Name: "greeter", Local: true, SourceRootSubpath: ".dagger/modules/greeter"},
			{Name: "fetcher", Reference: "github.com/acme/fetcher@v1.2.0"},
		},
	)
	encoded, err := metadata.CanonicalJSON(request)
	if err != nil {
		t.Fatalf("encode request: %v", err)
	}
	// The same document, byte for byte, is the one the Rust request model decodes
	// in crates/dagger-sdk-engine/tests/module_dependencies.rs.
	want := `{
  "request": {
    "clients": [
      {
        "kind": "local",
        "name": "greeter",
        "source_root_subpath": ".dagger/modules/greeter"
      },
      {
        "kind": "git",
        "name": "fetcher",
        "source": "github.com/acme/fetcher@v1.2.0"
      }
    ],
    "format_version": 1,
    "module_root": "workspace/.dagger/modules/gate",
    "workspace_root": "workspace"
  },
  "request_kind": "write-module-dependencies"
}
`
	if string(encoded) != want {
		t.Fatalf("request document is\n%s\nwant\n%s", encoded, want)
	}

	// No clients is a request too: the list is complete, so it empties the module's
	// dependencies.
	empty := moduleDependenciesRequestDocument("workspace", "workspace", nil)
	clients, ok := empty["request"].(map[string]any)["clients"].([]any)
	if !ok || clients == nil || len(clients) != 0 {
		t.Fatalf("a scope without clients must send an empty list, got %#v", empty["request"])
	}
}

func TestClientIsWrittenAsAPathOnlyWhenTheWorkspaceHoldsIt(t *testing.T) {
	const reference = "github.com/acme/workspace/modules/greeter@main"
	for _, test := range []struct {
		name      string
		kind      dagger.ModuleSourceKind
		workspace string
		err       error
		want      bool
	}{
		{name: "a local source", kind: dagger.ModuleSourceKindLocal, want: true},
		{name: "a directory source", kind: dagger.ModuleSourceKindDir, want: true},
		{name: "a git source the workspace resolves to itself", kind: dagger.ModuleSourceKindGit, workspace: reference, want: true},
		{name: "a git source the workspace resolves to another module", kind: dagger.ModuleSourceKindGit, workspace: "modules/greeter", want: false},
		{name: "a git source the workspace cannot resolve", kind: dagger.ModuleSourceKindGit, workspace: reference, err: fmt.Errorf("no module"), want: false},
	} {
		lookups := 0
		got := clientInWorkspace(test.kind, reference, func() (string, error) {
			lookups++
			return test.workspace, test.err
		})
		if got != test.want {
			t.Fatalf("%s: clientInWorkspace = %t, want %t", test.name, got, test.want)
		}
		if wantLookups := map[bool]int{true: 1, false: 0}[test.kind == dagger.ModuleSourceKindGit]; lookups != wantLookups {
			t.Fatalf("%s: workspace consulted %d times, want %d", test.name, lookups, wantLookups)
		}
	}
}

func TestWorkspaceModulePathIsAbsoluteFromTheWorkspaceRoot(t *testing.T) {
	for subpath, want := range map[string]string{
		".dagger/modules/greeter":  "/.dagger/modules/greeter",
		"/modules/greeter/":        "/modules/greeter",
		"./modules/greeter":        "/modules/greeter",
		"":                         "/.",
		"/":                        "/.",
		".":                        "/.",
		"modules/../other/greeter": "/modules/../other/greeter",
	} {
		if got := workspaceModulePath(subpath); got != want {
			t.Fatalf("workspaceModulePath(%q) = %q, want %q", subpath, got, want)
		}
	}
}

func TestNewModuleManifestIsUnindented(t *testing.T) {
	manifest := moduleManifest("gate", "1.0.0-beta.27", "github.com/tokeira/dagger-rust-sdk/runtime@v0.9.0")
	want := "name = \"gate\"\n" +
		"engineVersion = \"v1.0.0-beta.27\"\n\n" +
		"[runtime]\n" +
		"source = \"github.com/tokeira/dagger-rust-sdk/runtime@v0.9.0\"\n"
	if manifest != want {
		t.Fatalf("moduleManifest rendered\n%s\nwant\n%s", manifest, want)
	}
	for _, line := range strings.Split(manifest, "\n") {
		if strings.TrimLeft(line, " \t") != line {
			t.Fatalf("new module configuration indents %q", line)
		}
	}
}

// A client is read at its own naming. Setting a target's engine version renames
// its declarations by that version's casing rules, so no adapter path may do it.
func TestAdapterNeverSetsAnEngineVersionOnAModuleSource(t *testing.T) {
	sources, err := filepath.Glob("*.go")
	if err != nil {
		t.Fatalf("list adapter sources: %v", err)
	}
	checked := 0
	for _, name := range sources {
		if strings.HasSuffix(name, "_test.go") || name == "dagger.gen.go" {
			continue
		}
		file, err := parser.ParseFile(token.NewFileSet(), name, nil, parser.AllErrors)
		if err != nil {
			t.Fatalf("parse %s: %v", name, err)
		}
		checked++
		if count := selectorCount(file, "WithEngineVersion"); count != 0 {
			t.Fatalf("%s calls WithEngineVersion %d times", name, count)
		}
	}
	if checked < 4 {
		t.Fatalf("checked %d adapter sources; the adapter has at least four", checked)
	}
}

// The module branch of GenerateScope hands every scope's client list to Rust and
// refuses none of it.
func TestGenerateScopeWritesAModuleScopesClientsAsDependencies(t *testing.T) {
	file, err := parser.ParseFile(token.NewFileSet(), "main.go", nil, parser.AllErrors)
	if err != nil {
		t.Fatalf("parse adapter source: %v", err)
	}
	generateScope := findFunction(t, file, "GenerateScope")
	if count := selectorCount(generateScope, "moduleScopeDependencies"); count != 1 {
		t.Fatalf("GenerateScope calls moduleScopeDependencies %d times; want exactly once", count)
	}
	ast.Inspect(generateScope, func(node ast.Node) bool {
		call, ok := node.(*ast.CallExpr)
		if !ok {
			return true
		}
		if name, ok := call.Fun.(*ast.Ident); ok && name.Name == "len" && identifierCount(call, "clients") != 0 {
			t.Fatalf("GenerateScope still branches on the number of clients")
		}
		return true
	})
	source, err := os.ReadFile("main.go")
	if err != nil {
		t.Fatalf("read adapter source: %v", err)
	}
	if strings.Contains(string(source), "recorded for module scope") {
		t.Fatalf("GenerateScope still refuses the clients of a module scope")
	}
}
