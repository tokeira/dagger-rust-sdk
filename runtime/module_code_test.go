package main

import (
	"go/parser"
	"go/token"
	"reflect"
	"strings"
	"testing"
)

func TestTheRemovedListNamesFilesFromTheModuleSourceRoot(t *testing.T) {
	for _, test := range []struct {
		root    string
		removed []string
		listed  []string
	}{
		{
			root: "workspace/services/app",
			removed: []string{
				"workspace/services/app/src/dagger_generated/greeter.rs",
				"workspace/services/app/src/dagger_generated/greeter_style.rs",
			},
			listed: []string{"src/dagger_generated/greeter.rs", "src/dagger_generated/greeter_style.rs"},
		},
		{
			root:    "workspace",
			removed: []string{"workspace/src/dagger_generated/greeter.rs"},
			listed:  []string{"src/dagger_generated/greeter.rs"},
		},
	} {
		list, err := encodeRemovedModuleCode(test.removed, test.root)
		if err != nil {
			t.Fatalf("encode the removals of %s: %v", test.root, err)
		}
		decoded, err := decodeRemovedModuleCode([]byte(list))
		if err != nil {
			t.Fatalf("decode the removals of %s: %v", test.root, err)
		}
		if !reflect.DeepEqual(decoded, test.listed) {
			t.Fatalf("the removals of %s were listed as %#v, want %#v", test.root, decoded, test.listed)
		}
	}
}

func TestARemovedPathOutsideTheModuleIsRefused(t *testing.T) {
	for _, removed := range []string{
		"workspace/other/src/lib.rs",
		"workspace/application/src/lib.rs",
		"workspace/app",
		"workspace/app/",
		"src/lib.rs",
	} {
		if _, err := encodeRemovedModuleCode([]string{removed}, "workspace/app"); err == nil {
			t.Fatalf("removed path %q was listed for the module at workspace/app", removed)
		}
	}
}

func TestNoListRemovesNothing(t *testing.T) {
	removed, err := decodeRemovedModuleCode(nil)
	if err != nil || removed != nil {
		t.Fatalf("a generation without a list removed %#v (%v)", removed, err)
	}
}

func TestADamagedListIsRefused(t *testing.T) {
	for _, list := range []string{
		`not a list`,
		`{}`,
		`null`,
		`[]`,
		`[1]`,
		`[""]`,
		`["."]`,
		`["../escape.rs"]`,
		`["/absolute.rs"]`,
		`["src/../escape.rs"]`,
		`["./src/lib.rs"]`,
		`["src/dagger_generated/"]`,
		`["src\\lib.rs"]`,
		`[".dagger/rust/removed-paths.json"]`,
	} {
		if removed, err := decodeRemovedModuleCode([]byte(list)); err == nil {
			t.Fatalf("list %s was accepted as %#v", list, removed)
		}
	}
}

func TestTheRemovedListGoesFromCodegenToGenerateScope(t *testing.T) {
	files := token.NewFileSet()
	adapter, err := parser.ParseFile(files, "main.go", nil, parser.AllErrors)
	if err != nil {
		t.Fatalf("parse adapter: %v", err)
	}
	if got := identifierCount(findFunction(t, adapter, "Codegen"), "withRemovedModuleCode"); got != 1 {
		t.Fatalf("Codegen must list a module's removals through withRemovedModuleCode once, got %d", got)
	}
	if got := identifierCount(findFunction(t, adapter, "GenerateScope"), "withGeneratedModuleCode"); got != 1 {
		t.Fatalf("GenerateScope must apply a module's generated code through withGeneratedModuleCode once, got %d", got)
	}
	code, err := parser.ParseFile(files, "module_code.go", nil, parser.AllErrors)
	if err != nil {
		t.Fatalf("parse module code: %v", err)
	}
	for _, imported := range code.Imports {
		if strings.Contains(imported.Path.Value, "crypto/") || strings.Contains(imported.Path.Value, "metadata") {
			t.Fatalf("module code removal imports %s: Rust decides what is removed, and the adapter compares no digests", imported.Path.Value)
		}
	}
}
