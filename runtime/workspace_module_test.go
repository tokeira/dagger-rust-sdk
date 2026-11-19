package main

import (
	"go/parser"
	"go/token"
	"testing"

	"rust-sdk/internal/dagger"
)

// The cases are those of TestWorkspaceModule and TestWorkspaceModuleExplicitGitURL
// in cmd/dagger-go-sdk-codegen/source_links_test.go of github.com/dagger/go-sdk at
// bb23e8d, with the same facts and answers.
func TestWorkspaceModuleFollowsGo(t *testing.T) {
	const sha = "419c4afea78603265c40e273c455623a0291922e"
	type facts struct {
		kind                                        dagger.ModuleSourceKind
		reference, pin, cloneRef, workspace, commit string
	}
	at := func(workspace, selector, commit string) facts {
		return facts{dagger.ModuleSourceKindGit, "http://h/r.git/app@" + selector, sha, "http://h/r.git", workspace, commit}
	}
	github := func(workspace string) facts {
		return facts{dagger.ModuleSourceKindGit, "https://github.com/o/r/app@" + sha, sha, "https://github.com/o/r", workspace, sha}
	}
	explicit := func(reference, cloneRef, workspace string) facts {
		return facts{dagger.ModuleSourceKindGit, reference, sha, cloneRef, workspace, sha}
	}
	for name, test := range map[string]struct {
		facts facts
		want  bool
	}{
		"workspace at the commit":             {at("http://h/r.git@"+sha, sha, sha), true},
		"workspace without a selector":        {at("http://h/r.git", "main", sha), true},
		"workspace in a subdirectory":         {at("http://h/r.git/ci@main", "main", sha), true},
		"another selector at the same commit": {at("http://h/r.git@main", "release", sha), false},
		"another commit":                      {at("http://h/r.git@main", "main", "0000000000000000000000000000000000000000"), false},
		"another repository":                  {at("http://h/r.git-fork@"+sha, sha, sha), false},
		"workspace commit unknown":            {at("http://h/r.git@"+sha, sha, ""), false},
		"local module in a git workspace":     {facts{dagger.ModuleSourceKindLocal, "", "", "", "http://h/r.git@" + sha, sha}, false},
		"github address without a scheme":     {github("github.com/o/r@" + sha), true},
		"another host without a scheme":       {github("gitlab.com/o/r@" + sha), false},
		"explicit: github workspace":          {explicit("https://github.com/o/r#"+sha+":app", "https://github.com/o/r", "github.com/o/r@"+sha), true},
		"explicit: git server workspace":      {explicit("http://h/r.git#"+sha+":app", "http://h/r.git", "http://h/r.git@"+sha), true},
		"explicit: branch selector":           {explicit("http://h/r.git#main:app", "http://h/r.git", "http://h/r.git@main"), true},
		"explicit: another selector":          {explicit("http://h/r.git#release:app", "http://h/r.git", "http://h/r.git@main"), false},
		"explicit: another repository":        {explicit("https://github.com/o/r-fork#"+sha+":app", "https://github.com/o/r-fork", "github.com/o/r@"+sha), false},
		"explicit: form at the root":          {explicit("https://github.com/o/r#"+sha, "https://github.com/o/r", "github.com/o/r@"+sha), true},
	} {
		got := workspaceModule(test.facts.kind, test.facts.reference, test.facts.pin, test.facts.cloneRef, test.facts.workspace, test.facts.commit)
		if got != test.want {
			t.Errorf("%s: workspaceModule = %v, want %v", name, got, test.want)
		}
	}
}

// A package's Git client goes through the workspace rule before its address is
// chosen.
func TestPackageClientIdentityBindsAGitClientByTheWorkspaceRule(t *testing.T) {
	adapter, err := parser.ParseFile(token.NewFileSet(), "main.go", nil, parser.AllErrors)
	if err != nil {
		t.Fatalf("parse adapter source: %v", err)
	}
	identity := findFunction(t, adapter, "packageClientIdentity")
	if identifierCount(identity, "boundGitClient") != 1 || identifierCount(identity, "clientAddress") != 1 {
		t.Fatal("packageClientIdentity does not bind a Git client by the workspace rule before choosing its address")
	}
	if selectorCount(identity, "Pin") != 0 {
		t.Fatal("packageClientIdentity reads a Git client's pin outside the workspace rule")
	}
}
