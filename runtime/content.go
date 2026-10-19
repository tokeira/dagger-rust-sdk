package main

import (
	"context"
	"encoding/json"
	"fmt"
	"regexp"
	"strings"

	"rust-sdk/internal/dagger"
)

const (
	// rustSDKBuildImage matches the digest-pinned builder the engine-packaged content
	// uses, so a self-built content root compiles the same helper from the same inputs.
	rustSDKBuildImage = "rust:1.99.1-bookworm@sha256:e12566d5abc17a0da741628782cdb504b68b8961f5bfcf677936bf3e8b32de29"
	// releaseRepository publishes the SDK's crates.io releases, each from its tag.
	// Forks carry the same tags, so only this repository's tags name a release.
	releaseRepository = "https://github.com/tokeira/dagger-rust-sdk"
	// clientGenerationSourcePath is the client-generation metadata that sealed
	// content carries at clientGenerationPath.
	clientGenerationSourcePath = "crates/dagger-codegen/assets/client-generation.json"
)

var fullRevision = regexp.MustCompile(`^[0-9a-f]{40}$`)

type checkedTarget struct {
	DaggerRepository string `json:"dagger_repository"`
	DaggerRevision   string `json:"dagger_revision"`
	EngineVersion    string `json:"engine_version"`
	RustSDKVersion   string `json:"rust_sdk_version"`
	RustVersion      string `json:"rust_version"`
	SchemaDigest     string `json:"schema_digest"`
}

// sdkDependency is the SDK crate source that sealed content records for the
// projects it generates: a crates.io release, or a repository at one commit.
type sdkDependency struct {
	// Kind is "registry" or "git".
	Kind string
	// Value is the exact release version, or the full commit.
	Value string
	// Repository is the credential-free HTTPS repository of a Git dependency.
	Repository string
}

// runtimeReference returns the reference the engine loaded this runtime from. It
// is read only when the runtime seals its own content.
type runtimeReference func(context.Context) (string, error)

// resolved returns this SDK with a complete content root. An engine that packages
// the Rust SDK passes one with dist/ already sealed. Installed as an ordinary
// module, the runtime cannot take its workspace from a default path: with a
// workspace bound into the call, engines resolve default paths against that
// workspace rather than this module's source. Its module configuration instead
// includes the SDK files it needs, which the engine loads with the runtime from the
// same commit, and the runtime seals the content from those.
func (sdk *RustSDK) resolved(ctx context.Context, reference runtimeReference) (*RustSDK, error) {
	packaged, err := sdk.packaged(ctx)
	if err != nil {
		return nil, err
	}
	if packaged {
		return sdk, nil
	}
	workspace := dag.CurrentModule().ContextDirectory()
	var target checkedTarget
	if err := readJSON(ctx, workspace.File("codegen/target.json"), &target); err != nil {
		return nil, fmt.Errorf("read Rust SDK checked target: %w", err)
	}
	if target.DaggerRepository == "" || !fullRevision.MatchString(target.DaggerRevision) ||
		target.RustVersion == "" || target.RustSDKVersion == "" || target.SchemaDigest == "" {
		return nil, fmt.Errorf("Rust SDK checked target is incomplete")
	}
	source, err := reference(ctx)
	if err != nil {
		return nil, err
	}
	dependency, err := installedDependency(ctx, source, target.RustSDKVersion)
	if err != nil {
		return nil, err
	}
	content, err := sealWorkspaceContent(ctx, workspace, target, dependency)
	if err != nil {
		return nil, err
	}
	return &RustSDK{SDKSourceDir: content}, nil
}

// packaged reports whether the engine passed a complete content root.
func (sdk *RustSDK) packaged(ctx context.Context) (bool, error) {
	if sdk.SDKSourceDir == nil {
		return false, nil
	}
	packaged, err := sdk.SDKSourceDir.Exists(ctx, engineDescriptorPath)
	if err != nil {
		return false, fmt.Errorf("inspect Rust SDK content root: %w", err)
	}
	return packaged, nil
}

// installedDependency resolves the reference the runtime was loaded from. The
// engine resolved the same reference, under the same workspace lock, to load the
// runtime, so it names the commit whose files the content is built from.
func installedDependency(ctx context.Context, reference string, sdkVersion string) (sdkDependency, error) {
	if !isRemoteModuleReference(reference) {
		return sdkDependency{}, fmt.Errorf("the Rust SDK runtime is loaded from the local path %q; install it from a Git reference", reference)
	}
	source := dag.ModuleSource(reference, dagger.ModuleSourceOpts{RequireKind: dagger.ModuleSourceKindGit})
	version, err := source.Version(ctx)
	if err != nil {
		return sdkDependency{}, fmt.Errorf("resolve the Rust SDK runtime reference %q: %w", reference, err)
	}
	commit, err := source.Commit(ctx)
	if err != nil {
		return sdkDependency{}, fmt.Errorf("resolve the Rust SDK runtime reference %q: %w", reference, err)
	}
	cloneRef, err := source.CloneRef(ctx)
	if err != nil {
		return sdkDependency{}, fmt.Errorf("resolve the Rust SDK runtime reference %q: %w", reference, err)
	}
	return generatedDependency(version, commit, cloneRef, sdkVersion)
}

// generatedDependency names the SDK dependency of the projects a runtime generates,
// from the version, commit, and repository its reference resolved to. The tag of
// the release the content carries, in the repository that publishes releases,
// selects that release from crates.io. Any other reference selects the commit it
// resolved to, in the repository it names.
func generatedDependency(version, commit, cloneRef, sdkVersion string) (sdkDependency, error) {
	repository := cloneRef
	if !strings.Contains(repository, "://") {
		repository = "https://" + repository
	}
	if !strings.HasPrefix(repository, "https://") || strings.Contains(repository, "@") {
		return sdkDependency{}, fmt.Errorf("the Rust SDK runtime's repository %q is not a credential-free HTTPS repository; install the runtime from one", cloneRef)
	}
	if strings.TrimSuffix(repository, ".git") == releaseRepository && version == "v"+sdkVersion {
		return sdkDependency{Kind: "registry", Value: sdkVersion}, nil
	}
	if !fullRevision.MatchString(commit) {
		return sdkDependency{}, fmt.Errorf("the Rust SDK runtime's reference resolved to %q rather than a full commit", commit)
	}
	return sdkDependency{Kind: "git", Value: commit, Repository: repository}, nil
}

func sealWorkspaceContent(
	ctx context.Context,
	workspace *dagger.Directory,
	target checkedTarget,
	dependency sdkDependency,
) (*dagger.Directory, error) {
	platform, err := dag.DefaultPlatform(ctx)
	if err != nil {
		return nil, fmt.Errorf("resolve engine platform: %w", err)
	}
	rustTarget, err := rustTargetTriple(string(platform))
	if err != nil {
		return nil, err
	}
	rustfmtPath := fmt.Sprintf("/usr/local/rustup/toolchains/%s-%s/bin/rustfmt", target.RustVersion, rustTarget)

	rustWorkspace := workspace.Filter(dagger.DirectoryFilterOpts{
		Include: []string{
			"Cargo.toml", "Cargo.lock", "rust-toolchain.toml",
			"codegen/target.json", "codegen/schema.json", "codegen/generated.json",
			"crates/**/Cargo.toml", "crates/**/src/**/*.rs", "crates/**/assets/**",
		},
	})
	buildCtr := dag.Container(dagger.ContainerOpts{Platform: platform}).
		From(rustSDKBuildImage).
		WithDirectory("/src", rustWorkspace).
		WithWorkdir("/src").
		WithExec([]string{"rustup", "component", "add", "rustfmt", "--toolchain", target.RustVersion}).
		WithExec([]string{
			"cargo", "build", "--release", "--locked", "--package", "dagger-sdk-engine", "--bin", "dagger-rust-engine",
		})

	runtimeSource := workspace.Directory(sdkRuntimePath).Filter(dagger.DirectoryFilterOpts{
		Include: []string{
			"dagger-module.toml", "go.mod", "go.sum", "*.go", "internal/**/*.go",
		},
		Exclude: []string{"**/*_test.go"},
	})
	rootfs := dag.Directory().
		WithDirectory(sdkRuntimePath, runtimeSource).
		WithFile(engineToolPath, buildCtr.File("/src/target/release/dagger-rust-engine"), dagger.DirectoryWithFileOpts{Permissions: 0o755}).
		WithFile(formatterToolPath, buildCtr.File(rustfmtPath), dagger.DirectoryWithFileOpts{Permissions: 0o755}).
		WithFile(clientGenerationPath, workspace.File(clientGenerationSourcePath)).
		WithFile(runtimePolicyPath, workspace.File("runtime/assets/runtime-policy.json")).
		WithFile("LICENSE", workspace.File("LICENSE")).
		WithFile("NOTICE", workspace.File("NOTICE"))

	dependencyArgs := []string{"--dependency-kind", dependency.Kind, "--dependency-value", dependency.Value}
	if dependency.Kind == "git" {
		dependencyArgs = append(dependencyArgs, "--dependency-repository", dependency.Repository)
	}
	sealed := buildCtr.
		// A scratch mount makes the package tool's writes available as a clean
		// directory output without retaining builder layers.
		WithMountedDirectory("/content", rootfs).
		WithExec(append([]string{
			"/src/target/release/dagger-rust-engine", "package-content",
			"--root", "/content",
			"--repository", "https://" + target.DaggerRepository,
			"--dagger-revision", target.DaggerRevision,
			"--engine-version", strings.TrimPrefix(target.EngineVersion, "v"),
			"--rust-sdk-version", target.RustSDKVersion,
			"--rust-toolchain", target.RustVersion,
			"--core-schema-digest", target.SchemaDigest,
		}, dependencyArgs...))
	digest, err := sealed.Stdout(ctx)
	if err != nil {
		return nil, fmt.Errorf("seal Rust SDK content: %w", err)
	}
	if !strings.HasPrefix(strings.TrimSpace(digest), "sha256:") {
		return nil, fmt.Errorf("Rust SDK descriptor identity is absent or malformed")
	}
	return sealed.Directory("/content"), nil
}

func rustTargetTriple(platform string) (string, error) {
	switch {
	case strings.HasPrefix(platform, "linux/amd64"):
		return "x86_64-unknown-linux-gnu", nil
	case strings.HasPrefix(platform, "linux/arm64"):
		return "aarch64-unknown-linux-gnu", nil
	default:
		return "", fmt.Errorf("Rust SDK content does not support engine platform %q", platform)
	}
}

func readJSON(ctx context.Context, file *dagger.File, value any) error {
	contents, err := file.Contents(ctx)
	if err != nil {
		return err
	}
	return json.Unmarshal([]byte(contents), value)
}
