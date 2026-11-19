package main

import (
	"encoding/json"
	"fmt"
	"go/ast"
	"go/parser"
	"go/token"
	"io/fs"
	"os"
	"path/filepath"
	"reflect"
	"strconv"
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
		"",
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
		modern := generationRequestDocument(descriptor, "generate-client", identity, schema, "workspace/client", "")
		identity.ConfigFormat = "legacy"
		legacy := generationRequestDocument(descriptor, "generate-client", identity, schema, "workspace/client", "")
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
	got := moduleManifest("gate", "1.0.0-beta.27", `github.com/acme/sdk/"rust"\runtime@`+strings.Repeat("a", 40)+"\x01")
	want := "name = \"gate\"\n" +
		"engineVersion = \"v1.0.0-beta.27\"\n\n" +
		"[runtime]\n" +
		"  source = \"github.com/acme/sdk/\\\"rust\\\"\\\\runtime@" + strings.Repeat("a", 40) + "\\u0001\"\n"
	if got != want {
		t.Fatalf("moduleManifest rendered\n%s\nwant\n%s", got, want)
	}
}

func TestGenerationRequestForwardsOnlyAnAdmissibleEngineRelease(t *testing.T) {
	descriptor := metadata.EngineSource{
		FormatVersion: 1, Repository: "https://github.com/dagger/dagger",
		DaggerRevision: strings.Repeat("a", 40), EngineVersion: "1.0.0-beta.27",
		RustSDKVersion: "1.0.0-beta.11.rust.5", RustToolchain: "1.99.1",
		CoreSchemaDigest: "sha256:" + strings.Repeat("b", 64),
		SDKDependency: metadata.PublishedSDKDependency{
			Source: "registry", Registry: "crates-io", Package: "tokeira-dagger-sdk", ExactVersion: "1.0.0-beta.11.rust.5",
		},
	}
	identity := scopedModuleIdentity{
		Name: "fixture", OriginalName: "Fixture", SourceSubpath: "workspace/module",
		SourceDigest: metadata.DigestModuleSource("fixture"), ConfigFormat: "current",
	}
	schema := []byte(`{"__schema":{"types":[]}}`)
	stated := generationRequestDocument(descriptor, "generate-module", identity, schema, "workspace/module", "v1.0.0-beta.28+1a2b3c4d")
	if got := stated["request"].(map[string]any)["engine_version"]; got != "v1.0.0-beta.28+1a2b3c4d" {
		t.Fatalf("stated engine release was not forwarded: %#v", got)
	}
	unstated := generationRequestDocument(descriptor, "generate-module", identity, schema, "workspace/module", "")
	if _, exists := unstated["request"].(map[string]any)["engine_version"]; exists {
		t.Fatalf("an unstated engine release must be omitted from the request")
	}

	for _, test := range []struct {
		release    string
		admissible bool
	}{
		{"v1.0.0-beta.27+400b82d6", true},
		{"v1.0.0-beta.28", true},
		{"v1.0.0", true},
		{"devel", true},
		{strings.Repeat("a", maxEngineReleaseLength), true},
		{"", false},
		{strings.Repeat("a", maxEngineReleaseLength+1), false},
		{"v1.0.0 beta", false},
		{"v1.0.0\n", false},
		{"v1.0.0_dirty", false},
		{"v1.0.0-<beta>", false},
		{"v1.0.0-bêta", false},
	} {
		if got := admissibleEngineRelease(test.release); got != test.admissible {
			t.Fatalf("admissibleEngineRelease(%q) = %v, want %v", test.release, got, test.admissible)
		}
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
		{"entire://us.entire.io/gh/tokeira/dagger-rust-sdk/runtime", true},
		{"entire://forge/gh/tokeira/dagger-rust-sdk/runtime", true},
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

// The request for a package's module clients is data alone, and exactly the
// document the Rust request model decodes: the same file is a fixture of the Rust
// operation's tests, which decode it and encode it back byte for byte.
func TestPackageClientsRequestIsTheDocumentRustDecodes(t *testing.T) {
	descriptor := metadata.EngineSource{
		FormatVersion:    1,
		Repository:       "https://github.com/dagger/dagger",
		DaggerRevision:   "400b82d6f3c09ef096d2f59b3e920267b5969bf4",
		EngineVersion:    "1.0.0-beta.27",
		RustSDKVersion:   "0.8.0",
		RustToolchain:    "1.99.1",
		CoreSchemaDigest: "sha256:9511bc82fb704520cdc632da73463fc7396b1bb081e0446b61665c30764a5e7c",
		SDKDependency: metadata.PublishedSDKDependency{
			Source:       "registry",
			Registry:     "crates-io",
			Package:      "tokeira-dagger-sdk",
			ExactVersion: "0.8.0",
		},
	}
	clientRoot, err := rebasePath("services/app")
	if err != nil {
		t.Fatal(err)
	}
	request := packageClientsRequestDocument(descriptor, clientRoot, []packageClient{
		{
			Name:         "greeter",
			Address:      "/.dagger/modules/greeter",
			SchemaPath:   "0.json",
			SchemaDigest: "sha256:" + strings.Repeat("1", 64),
		},
		{
			Name:         "naming-app",
			Address:      "github.com/example/modules/naming-app@v1.2.0",
			Pin:          "1e6e0d5c7ccfe1c712c692c229fe2c0b35d55a4b",
			SchemaPath:   "1.json",
			SchemaDigest: "sha256:" + strings.Repeat("2", 64),
		},
	})
	encoded, err := metadata.CanonicalJSON(request)
	if err != nil {
		t.Fatal(err)
	}
	expected, err := os.ReadFile("../crates/dagger-sdk-engine/tests/fixtures/package-clients-request.json")
	if err != nil {
		t.Fatalf("read the shared request fixture: %v", err)
	}
	if string(encoded) != string(expected) {
		t.Fatalf("request document differs from the one Rust decodes:\n%s", encoded)
	}

	// With no client the document still lists none, so Rust removes what remains.
	empty := packageClientsRequestDocument(descriptor, clientRoot, nil)
	if clients, ok := empty["request"].(map[string]any)["clients"].([]any); !ok || len(clients) != 0 {
		t.Fatalf("an empty client set is not an empty list: %#v", empty)
	}
}

// A generated client serves its module from the address and pin Dagger's Go SDK
// gives its own: a git module's reference and commit, a local module's path from
// the workspace root with no pin. No other kind of source can be a client.
func TestClientAddressFollowsTheSourceKind(t *testing.T) {
	commit := "1e6e0d5c7ccfe1c712c692c229fe2c0b35d55a4b"
	for _, test := range []struct {
		name      string
		kind      dagger.ModuleSourceKind
		reference string
		pin       string
		subpath   string
		address   string
		wantPin   string
		refused   bool
	}{
		{name: "git", kind: dagger.ModuleSourceKindGit, reference: "github.com/example/modules/greeter@v1.2.0", pin: commit, subpath: "greeter", address: "github.com/example/modules/greeter@v1.2.0", wantPin: commit},
		{name: "local", kind: dagger.ModuleSourceKindLocal, reference: "ignored", pin: commit, subpath: ".dagger/modules/greeter", address: "/.dagger/modules/greeter"},
		{name: "local at the root", kind: dagger.ModuleSourceKindLocal, subpath: ".", address: "/."},
		{name: "local, already rooted", kind: dagger.ModuleSourceKindLocal, subpath: "/modules/greeter", address: "/modules/greeter"},
		{name: "git without a pin", kind: dagger.ModuleSourceKindGit, reference: "github.com/example/modules/greeter", refused: true},
		{name: "git pinned to a branch", kind: dagger.ModuleSourceKindGit, reference: "github.com/example/modules/greeter", pin: "main", refused: true},
		{name: "git without a reference", kind: dagger.ModuleSourceKindGit, pin: commit, refused: true},
		{name: "directory", kind: dagger.ModuleSourceKindDir, subpath: "greeter", refused: true},
		{name: "unknown", kind: dagger.ModuleSourceKind("UNKNOWN"), refused: true},
	} {
		address, pin, err := clientAddress(test.kind, test.reference, test.pin, test.subpath)
		if test.refused {
			if err == nil {
				t.Errorf("%s: accepted as %q, %q", test.name, address, pin)
			}
			continue
		}
		if err != nil || address != test.address || pin != test.wantPin {
			t.Errorf("%s: clientAddress = %q, %q, %v; want %q, %q", test.name, address, pin, err, test.address, test.wantPin)
		}
	}
}

// The operation reads the package's manifest, the manifests above it, and its
// clients directory, and nothing else of the workspace.
func TestPackageClientInputsAreTheManifestsAndTheClientsDirectory(t *testing.T) {
	for scope, want := range map[string][]string{
		".":            {"Cargo.toml", "src/dagger_clients/**"},
		"app":          {"app/Cargo.toml", "Cargo.toml", "app/src/dagger_clients/**"},
		"services/app": {"services/app/Cargo.toml", "services/Cargo.toml", "Cargo.toml", "services/app/src/dagger_clients/**"},
	} {
		if got := packageClientInputs(scope); !reflect.DeepEqual(got, want) {
			t.Errorf("packageClientInputs(%q) = %q; want %q", scope, got, want)
		}
	}
}

// What the operation reports is applied to the workspace only where generation
// owns: the package's manifest and its clients directory, and removal in the
// clients directory alone. Workspace configuration and authored source are out of
// reach whatever a result says.
func TestPackageClientChangesAreConfinedToWhatGenerationOwns(t *testing.T) {
	for _, test := range []struct {
		changed string
		removed bool
		want    string
	}{
		{changed: "workspace/app/Cargo.toml", want: "/app/Cargo.toml"},
		{changed: "workspace/app/src/dagger_clients/mod.rs", want: "/app/src/dagger_clients/mod.rs"},
		{changed: "workspace/app/src/dagger_clients/greeter/greeter.rs", removed: true, want: "/app/src/dagger_clients/greeter/greeter.rs"},
		{changed: "workspace/app/Cargo.toml", removed: true},
		{changed: "workspace/app/Cargo.lock"},
		{changed: "workspace/app/src/main.rs"},
		{changed: "workspace/app/src/dagger_clients"},
		{changed: "workspace/app/dagger.toml"},
		{changed: "workspace/dagger.toml"},
		{changed: "workspace/Cargo.toml"},
		{changed: "workspace/other/src/dagger_clients/mod.rs"},
		{changed: "workspace/application/Cargo.toml"},
	} {
		got, err := packageClientChange("workspace/app", test.changed, test.removed)
		if test.want == "" {
			if err == nil {
				t.Errorf("change to %q (removed %t) accepted as %q", test.changed, test.removed, got)
			}
			continue
		}
		if err != nil || got != test.want {
			t.Errorf("packageClientChange(%q, %t) = %q, %v; want %q", test.changed, test.removed, got, err, test.want)
		}
	}
	if got, err := packageClientChange("workspace", "workspace/src/dagger_clients/mod.rs", false); err != nil || got != "/src/dagger_clients/mod.rs" {
		t.Errorf("a package at the workspace root: %q, %v", got, err)
	}
}

// A client's schema is read through clientSchemaIntrospectionJson(clientVersion:),
// which describes Core for the client and leaves the module named by the engine
// version it declares. withEngineVersion makes the module declare another version
// instead, which renames its declarations by that version's rules: a client would
// then say NamingAppHttpFetcher where the engine serves NamingAppHTTPFetcher. So no
// runtime source calls it; only the generated bindings define it.
func TestNoRuntimeSourceMakesAModuleDeclareAnotherEngineVersion(t *testing.T) {
	checked := 0
	err := filepath.WalkDir(".", func(file string, entry fs.DirEntry, err error) error {
		if err != nil {
			return err
		}
		if entry.IsDir() {
			if filepath.ToSlash(file) == "internal/dagger" {
				return filepath.SkipDir
			}
			return nil
		}
		if !strings.HasSuffix(file, ".go") || strings.HasSuffix(file, "_test.go") || strings.HasSuffix(file, ".gen.go") {
			return nil
		}
		source, err := parser.ParseFile(token.NewFileSet(), file, nil, parser.AllErrors)
		if err != nil {
			return err
		}
		checked++
		if selectorCount(source, "WithEngineVersion") != 0 {
			t.Errorf("%s calls WithEngineVersion; read a client through ClientSchemaIntrospectionJSON's ClientVersion instead", file)
		}
		return nil
	})
	if err != nil {
		t.Fatalf("walk runtime sources: %v", err)
	}
	if checked < 3 {
		t.Fatalf("only %d runtime sources were checked", checked)
	}
	adapter, err := parser.ParseFile(token.NewFileSet(), "main.go", nil, parser.AllErrors)
	if err != nil {
		t.Fatalf("parse adapter source: %v", err)
	}
	clients := findFunction(t, adapter, "generatedPackageClients")
	if selectorCount(clients, "ClientSchemaIntrospectionJSON") != 1 || identifierCount(clients, "ClientVersion") != 1 {
		t.Fatal("a client's schema is not read through clientSchemaIntrospectionJson with a client version")
	}
	for _, other := range []string{"IntrospectionSchemaJSON", "AsModule"} {
		if selectorCount(clients, other) != 0 {
			t.Fatalf("a client's schema is also read through %s", other)
		}
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

func TestRuntimeSemanticDigestExcludesOnlyGeneratedRepresentations(t *testing.T) {
	source, err := parser.ParseFile(token.NewFileSet(), "main.go", nil, parser.AllErrors)
	if err != nil {
		t.Fatalf("parse runtime source: %v", err)
	}
	identity := findFunction(t, source, "semanticModuleSourceDigest")
	for _, generated := range []string{
		".dagger/rust", ".dagger/rust/**",
		"src/dagger_generated", "src/dagger_generated/**",
		"src/bin/dagger-module.rs",
		".gitattributes", ".gitignore",
	} {
		if got := stringLiteralCount(identity, generated); got != 1 {
			t.Fatalf("semantic module digest must exclude %q exactly once, got %d", generated, got)
		}
	}
	for _, retained := range []string{"dagger-module.toml", "dagger.json"} {
		if got := stringLiteralCount(identity, retained); got != 0 {
			t.Fatalf("semantic module digest must retain module config %q", retained)
		}
	}
	if got := identifierCount(identity, "ExcludeMetadata"); got != 1 {
		t.Fatalf("semantic module digest must exclude unstable file metadata exactly once, got %d", got)
	}
	if got := selectorCount(identity, "UpdatedConfigDirectory"); got != 1 {
		t.Fatalf("semantic module digest must overlay normalized module config exactly once, got %d", got)
	}
}

func findFunction(t *testing.T, file *ast.File, name string) *ast.FuncDecl {
	t.Helper()

	for _, declaration := range file.Decls {
		function, ok := declaration.(*ast.FuncDecl)
		if ok && function.Name.Name == name {
			return function
		}
	}
	t.Fatalf("function %s not found", name)
	return nil
}

func stringLiteralCount(node ast.Node, value string) int {
	count := 0
	ast.Inspect(node, func(node ast.Node) bool {
		literal, ok := node.(*ast.BasicLit)
		if !ok || literal.Kind != token.STRING {
			return true
		}
		decoded, err := strconv.Unquote(literal.Value)
		if err == nil && decoded == value {
			count++
		}
		return true
	})
	return count
}

func identifierCount(node ast.Node, name string) int {
	count := 0
	ast.Inspect(node, func(node ast.Node) bool {
		identifier, ok := node.(*ast.Ident)
		if ok && identifier.Name == name {
			count++
		}
		return true
	})
	return count
}

func selectorCount(node ast.Node, name string) int {
	count := 0
	ast.Inspect(node, func(node ast.Node) bool {
		selector, ok := node.(*ast.SelectorExpr)
		if ok && selector.Sel.Name == name {
			count++
		}
		return true
	})
	return count
}
