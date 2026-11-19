//! Pure engine operation compiler and bounded production renderer seams.
//!
//! The facade validates one complete visible schema, constructs one immutable semantic
//! plan, and invokes exactly one renderer. Filesystem access, Cargo execution, engine
//! sessions, publication, and completeness transitions remain outside this crate.

mod dependency;
mod library;
mod model;
mod module;
mod module_client;
mod render;
mod renderers;
mod visible;

pub use model::{
    CandidateArtifact, CandidateArtifactKind, CargoBinaryTarget, ContentDomain,
    ModuleAuthoringInput, ModuleProjectionInput, OperationKind, OperationPlan,
    OperationProjectionRequest, PostWorkPlan, PublishedSdkDependency, RelativeOperationPath,
};
pub use module_client::{ModuleClientRenderInput, render_module_client};
pub use renderers::{
    EntrypointRenderInput, LibraryRenderInput, ModuleRenderInput, OperationRenderer,
    PreparedOperationRequest, RendererOutput, dispatch_prepared_operation, project_operation_with,
};
pub use visible::{VisibleSchemaPlan, project_visible_schema};

use crate::diagnostic::DiagnosticSet;

/// The production renderer set: the library, module and entrypoint renderers this
/// crate ships.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ProductionRenderers;

impl OperationRenderer for ProductionRenderers {
    fn render_library(
        &self,
        input: LibraryRenderInput<'_>,
    ) -> Result<RendererOutput, DiagnosticSet> {
        library::render(input)
    }

    fn render_module(&self, input: ModuleRenderInput<'_>) -> Result<RendererOutput, DiagnosticSet> {
        module::render(input)
    }

    fn render_entrypoint(
        &self,
        input: EntrypointRenderInput<'_>,
    ) -> Result<RendererOutput, DiagnosticSet> {
        module::render_entrypoint(input)
    }
}

/// Projects and renders one operation through the production renderer set.
pub fn project_operation(
    request: OperationProjectionRequest<'_>,
) -> Result<OperationPlan, DiagnosticSet> {
    project_operation_with(&ProductionRenderers, request)
}
