package metadata

import "testing"

func TestModuleDependenciesResultNamesOnlyTheModuleConfiguration(t *testing.T) {
	const moduleRoot = "workspace/.dagger/modules/gate"
	const written = `{
  "format_version": 1,
  "kind": "module-dependencies",
  "operation_manifest": null,
  "output_root": "workspace/.dagger/modules/gate",
  "touched_paths": [
    "workspace/.dagger/modules/gate/dagger-module.toml"
  ],
  "vcs_generated": [],
  "vcs_ignored": []
}
`
	result, err := DecodeExecutionResult([]byte(written), "module-dependencies")
	if err != nil {
		t.Fatalf("decode result: %v", err)
	}
	manifest, changed, err := WrittenModuleManifest(result, moduleRoot)
	if err != nil || !changed || manifest != moduleRoot+"/dagger-module.toml" {
		t.Fatalf("WrittenModuleManifest = %q, %t, %v", manifest, changed, err)
	}

	unchanged := result
	unchanged.TouchedPaths = nil
	if manifest, changed, err := WrittenModuleManifest(unchanged, moduleRoot); err != nil || changed || manifest != "" {
		t.Fatalf("an unchanged module reported %q, %t, %v", manifest, changed, err)
	}

	manifestPath := moduleRoot + "/dagger-module.toml"
	for name, mutate := range map[string]func(*ExecutionResult){
		"another result kind":  func(result *ExecutionResult) { result.Kind = "initialization" },
		"another module":       func(result *ExecutionResult) { result.OutputRoot = "workspace/.dagger/modules/other" },
		"another file":         func(result *ExecutionResult) { result.TouchedPaths = []string{moduleRoot + "/Cargo.toml"} },
		"the workspace config": func(result *ExecutionResult) { result.TouchedPaths = []string{"workspace/dagger.toml"} },
		"a second file": func(result *ExecutionResult) {
			result.TouchedPaths = []string{manifestPath, moduleRoot + "/dagger.json"}
		},
		"an operation manifest": func(result *ExecutionResult) { result.OperationManifest = &manifestPath },
		"a generated VCS path":  func(result *ExecutionResult) { result.VCSGenerated = []string{manifestPath} },
		"an ignored VCS path":   func(result *ExecutionResult) { result.VCSIgnored = []string{manifestPath} },
	} {
		mutated := result
		mutate(&mutated)
		if _, _, err := WrittenModuleManifest(mutated, moduleRoot); err == nil {
			t.Fatalf("a result with %s was accepted", name)
		}
	}
}
