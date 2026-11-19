package metadata

import "fmt"

const moduleManifestName = "dagger-module.toml"

// WrittenModuleManifest returns the operation path of the module configuration
// that a module-dependencies result rewrote, and false when the result left the
// module as it was. The result may name that one file and nothing else.
func WrittenModuleManifest(result ExecutionResult, moduleRoot string) (string, bool, error) {
	if result.Kind != "module-dependencies" || result.OutputRoot != moduleRoot ||
		result.OperationManifest != nil || len(result.VCSGenerated) != 0 || len(result.VCSIgnored) != 0 {
		return "", false, fmt.Errorf("module dependencies result differs from the requested operation")
	}
	switch len(result.TouchedPaths) {
	case 0:
		return "", false, nil
	case 1:
		manifest := moduleRoot + "/" + moduleManifestName
		if result.TouchedPaths[0] == manifest {
			return manifest, true, nil
		}
	}
	return "", false, fmt.Errorf("module dependencies result names a path other than the module configuration")
}
