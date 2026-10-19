package main

import (
	"encoding/json"
	"fmt"
	"os"
	"reflect"
	"strings"
	"testing"

	"rust-sdk/internal/dagger"
	"rust-sdk/internal/metadata"
)

func TestGenerationRequestKeepsAuthoringSemanticsInRust(t *testing.T) {
	descriptor := metadata.EngineSource{
		FormatVersion:    1,
		Repository:       "https://github.com/dagger/dagger",
		DaggerRevision:   strings.Repeat("a", 40),
		EngineVersion:    "1.0.0-beta.11.rust.5",
		RustSDKVersion:   "1.0.0-beta.11.rust.5",
		RustToolchain:    "1.99.1",
		CoreSchemaDigest: "sha256:" + strings.Repeat("b", 64),
		SDKDependency: metadata.PublishedSDKDependency{
			Source:       "registry",
			Registry:     "crates-io",
			Package:      "tokeira-dagger-sdk",
			ExactVersion: "1.0.0-beta.11.rust.5",
		},
	}
	request := generationRequestDocument(
		descriptor,
		"generate-module",
		scopedModuleIdentity{
			Name:          "fixture",
			OriginalName:  "Fixture",
			SourceSubpath: "workspace/module",
			SourceDigest:  metadata.DigestModuleSource("fixture"),
			ConfigFormat:  "current",
		},
		[]byte(`{"__schema":{"types":[]}}`),
		"workspace/module",
	)
	body := request["request"].(map[string]any)
	for _, forbidden := range []string{
		"entrypoint_type_defs",
		"source_snapshot",
		"descriptor",
		"registration",
		"codec",
		"dispatch",
		"defaults",
		"state",
		"metadata_policy",
		"evidence",
	} {
		if _, exists := body[forbidden]; exists {
			t.Fatalf("Go request unexpectedly owns Rust semantic field %q", forbidden)
		}
	}
	if body["operation"] != "generate-module" || body["output_root"] != "workspace/module" {
		t.Fatalf("closed operation selectors were not forwarded exactly: %#v", body)
	}
}

func TestProperty19ModernLegacyGenerationSemanticallyConverge(t *testing.T) {
	descriptor := metadata.EngineSource{
		FormatVersion: 1, Repository: "https://github.com/dagger/dagger",
		DaggerRevision: strings.Repeat("a", 40), EngineVersion: "1.0.0-beta.11.rust.5",
		RustSDKVersion: "1.0.0-beta.11.rust.5", RustToolchain: "1.99.1",
		CoreSchemaDigest: "sha256:" + strings.Repeat("b", 64),
		SDKDependency: metadata.PublishedSDKDependency{
			Source: "registry", Registry: "crates-io", Package: "tokeira-dagger-sdk", ExactVersion: "1.0.0-beta.11.rust.5",
		},
	}
	for seed := 0; seed < 256; seed++ {
		pin := ""
		if seed%2 == 0 {
			pin = strings.Repeat(fmt.Sprintf("%x", seed%16), 40)
		}
		identity := scopedModuleIdentity{
			Name: fmt.Sprintf("module-%d", seed), OriginalName: fmt.Sprintf("Module %d", seed),
			SourceSubpath: fmt.Sprintf("workspace/module-%d", seed),
			SourceDigest:  metadata.DigestModuleSource(fmt.Sprintf("source-%d", seed)),
			ConfigFormat:  "current", ResolvedPin: pin,
		}
		schema := []byte(fmt.Sprintf(`{"fixture":%d}`, seed))
		modern := generationRequestDocument(descriptor, "generate-client", identity, schema, "workspace/client")
		identity.ConfigFormat = "legacy"
		legacy := generationRequestDocument(descriptor, "generate-client", identity, schema, "workspace/client")
		if reflect.DeepEqual(modern, legacy) {
			t.Fatalf("adapter variants did not exercise their control-path distinction for seed %d", seed)
		}
		if !reflect.DeepEqual(clientGenerationSemantics(modern), clientGenerationSemantics(legacy)) {
			t.Fatalf("equivalent adapters diverged for seed %d", seed)
		}
	}
}

func clientGenerationSemantics(document map[string]any) map[string]any {
	encoded, err := json.Marshal(document)
	if err != nil {
		panic(err)
	}
	var normalized map[string]any
	if err := json.Unmarshal(encoded, &normalized); err != nil {
		panic(err)
	}
	request := normalized["request"].(map[string]any)
	module := request["module"].(map[string]any)
	delete(module, "config_format")
	return normalized
}

func TestCheckedRuntimeAPIImplementsTheSDKModuleInterface(t *testing.T) {
	entrypoint, err := os.ReadFile("dagger.gen.go")
	if err != nil {
		t.Fatalf("read checked module entrypoint: %v", err)
	}
	for _, token := range []string{
		`case "FindClientRoot":`,
		`dag.Function("FindClientRoot",`,
		`case "GenerateScope":`,
		`dag.Function("GenerateScope",`,
		`WithArg("isModule", dag.TypeDef().WithKind(dagger.TypeDefKindBooleanKind)`,
		`WithArg("clients", dag.TypeDef().WithListOf(dag.TypeDef().WithObject("ModuleSource"))`,
	} {
		if count := strings.Count(string(entrypoint), token); count != 1 {
			t.Fatalf("checked runtime API contains %q %d times; want exactly once", token, count)
		}
	}
	// Engines that drive generation through generateScope never call the
	// workspace initializers or generator functions of the earlier interface.
	for _, retired := range []string{"InitModule", "InitClient", "GenerateModules", "GenerateClients"} {
		if strings.Contains(string(entrypoint), `"`+retired+`"`) {
			t.Fatalf("checked runtime API still registers %s", retired)
		}
	}
}

func TestRuntimeConstructorTakesNoWorkspaceDefaultPath(t *testing.T) {
	source, err := os.ReadFile("main.go")
	if err != nil {
		t.Fatalf("read runtime source: %v", err)
	}
	entrypoint, err := os.ReadFile("dagger.gen.go")
	if err != nil {
		t.Fatalf("read checked module entrypoint: %v", err)
	}
	// With a workspace bound into the call, engines resolve a default path against
	// that workspace, so the content root comes from the files the runtime's module
	// configuration includes.
	for token, body := range map[string][]byte{
		`+defaultPath`: source,
		`DefaultPath:`: entrypoint,
	} {
		if strings.Contains(string(body), token) {
			t.Fatalf("runtime constructor declares a default path through %q", token)
		}
	}
}

func TestNewModuleManifestQuotesEveryValue(t *testing.T) {
	got := moduleManifest("gate", "v1.0.0-beta.19+80699ac6", `github.com/acme/sdk/"rust"\runtime@`+strings.Repeat("a", 40)+"\x01")
	want := "name = \"gate\"\n" +
		"engineVersion = \"v1.0.0-beta.19\"\n\n" +
		"[runtime]\n" +
		"  source = \"github.com/acme/sdk/\\\"rust\\\"\\\\runtime@" + strings.Repeat("a", 40) + "\\u0001\"\n"
	if got != want {
		t.Fatalf("moduleManifest rendered\n%s\nwant\n%s", got, want)
	}
}

func TestInstalledRuntimeSourceMustBeRemote(t *testing.T) {
	for _, test := range []struct {
		source string
		remote bool
	}{
		{"github.com/tokeira/dagger-rust-sdk/runtime@" + strings.Repeat("a", 40), true},
		{"https://github.com/tokeira/dagger-rust-sdk/runtime", true},
		{"git@github.com:tokeira/dagger-rust-sdk.git/runtime", true},
		{"runtime", false},
		{"./runtime", false},
		{"../runtime", false},
		{"/runtime", false},
		{".dagger/modules/rust-sdk", false},
	} {
		if got := isRemoteModuleReference(test.source); got != test.remote {
			t.Fatalf("isRemoteModuleReference(%q) = %t; want %t", test.source, got, test.remote)
		}
	}
}

func TestGeneratedDependencyFollowsTheInstalledReference(t *testing.T) {
	commit := strings.Repeat("a", 40)
	release := sdkDependency{Kind: "registry", Value: "0.2.0"}
	atCommit := func(repository string) sdkDependency {
		return sdkDependency{Kind: "git", Value: commit, Repository: repository}
	}
	for _, test := range []struct {
		reference string
		version   string
		cloneRef  string
		want      sdkDependency
	}{
		{"the release's tag", "v0.2.0", "github.com/tokeira/dagger-rust-sdk", release},
		{"the release's tag over HTTPS", "v0.2.0", "https://github.com/tokeira/dagger-rust-sdk", release},
		{"the release's tag with .git", "v0.2.0", "github.com/tokeira/dagger-rust-sdk.git", release},
		{"another release's tag", "v0.1.0", "github.com/tokeira/dagger-rust-sdk", atCommit("https://github.com/tokeira/dagger-rust-sdk")},
		{"a commit", commit, "github.com/tokeira/dagger-rust-sdk", atCommit("https://github.com/tokeira/dagger-rust-sdk")},
		{"a branch", "main", "https://github.com/tokeira/dagger-rust-sdk", atCommit("https://github.com/tokeira/dagger-rust-sdk")},
		{"a branch with .git", "main", "github.com/tokeira/dagger-rust-sdk.git", atCommit("https://github.com/tokeira/dagger-rust-sdk.git")},
		{"a fork's copy of the release's tag", "v0.2.0", "github.com/acme/dagger-rust-sdk", atCommit("https://github.com/acme/dagger-rust-sdk")},
	} {
		got, err := generatedDependency(test.version, commit, test.cloneRef, "0.2.0")
		if err != nil || got != test.want {
			t.Fatalf("generatedDependency for %s = %+v, %v; want %+v", test.reference, got, err, test.want)
		}
	}
	for _, cloneRef := range []string{
		"git@github.com:tokeira/dagger-rust-sdk",
		"ssh://git@github.com/tokeira/dagger-rust-sdk",
		"http://github.com/tokeira/dagger-rust-sdk",
		"https://ci-bot@github.com/tokeira/dagger-rust-sdk",
	} {
		if got, err := generatedDependency("v0.2.0", commit, cloneRef, "0.2.0"); err == nil {
			t.Fatalf("generatedDependency for repository %q = %+v; want a refusal", cloneRef, got)
		}
	}
	if got, err := generatedDependency("main", "a1b2c3d", "github.com/tokeira/dagger-rust-sdk", "0.2.0"); err == nil {
		t.Fatalf("generatedDependency for an abbreviated commit = %+v; want a refusal", got)
	}
}

func TestModuleSourcePinPolicyAcceptsImmutableDirectorySnapshots(t *testing.T) {
	for _, test := range []struct {
		kind dagger.ModuleSourceKind
		want bool
	}{
		{kind: dagger.ModuleSourceKindLocal, want: false},
		{kind: dagger.ModuleSourceKindDir, want: false},
		{kind: dagger.ModuleSourceKindGit, want: true},
	} {
		got, err := moduleSourceRequiresPin(test.kind)
		if err != nil || got != test.want {
			t.Fatalf("moduleSourceRequiresPin(%q) = %t, %v; want %t", test.kind, got, err, test.want)
		}
	}
	if _, err := moduleSourceRequiresPin(dagger.ModuleSourceKind("UNKNOWN")); err == nil {
		t.Fatal("unknown module source kind unexpectedly accepted")
	}
}

func TestAdapterSourceContainsNoRustBehaviouralModel(t *testing.T) {
	source, err := os.ReadFile("main.go")
	if err != nil {
		t.Fatalf("read adapter source: %v", err)
	}
	for _, forbidden := range []string{
		"ModuleDescriptor",
		"RegistrationProjection",
		"CallEnvelope",
		"DispatchRegistry",
		"ModuleInputCodec",
		"ModuleOutputCodec",
		"dagger(function",
		"dagger(object",
	} {
		if strings.Contains(string(source), forbidden) {
			t.Fatalf("Go adapter contains Rust behavioural model token %q", forbidden)
		}
	}
}
