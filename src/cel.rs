//! CEL-backed indicator evaluation (enabled by the `cel` cargo feature).
//!
//! Mirrors the Go library's optional `celindicators` subpackage: a project
//! type may carry an [`Indicator::Cel`](crate::Indicator::Cel) whose
//! expression is evaluated against two `list<string>` variables — `files`
//! (file basenames) and `subdirs` (immediate subdirectory basenames). Any
//! evaluation that is not boolean `true`, or that errors, counts as no match
//! (the Go implementation swallows errors to `false`).
//!
//! The compiled program is validated at registration time so bad expressions
//! surface immediately. We retain the source and recompile per evaluation,
//! which keeps the registry `Send + Sync` regardless of the CEL crate's
//! internal representation; CEL indicators are rare and only evaluated for
//! directories that fail every cheaper indicator first.

use cel::{Context, Program as CelProgram, Value};

/// A validated CEL directory predicate.
#[derive(Debug, Clone)]
pub(crate) struct Program {
    source: String,
}

impl Program {
    /// Compiles and validates `expr`. Returns the compiler's error message on
    /// failure.
    pub(crate) fn compile(expr: &str) -> Result<Program, String> {
        CelProgram::compile(expr).map_err(|e| e.to_string())?;
        Ok(Program {
            source: expr.to_string(),
        })
    }

    /// Evaluates the predicate against a directory's listing. Returns `false`
    /// on any compile/runtime error or non-boolean / false result.
    pub(crate) fn eval(&self, files: &[String], subdirs: &[String]) -> bool {
        let program = match CelProgram::compile(&self.source) {
            Ok(p) => p,
            Err(_) => return false,
        };
        let mut ctx = Context::default();
        if ctx.add_variable("files", files.to_vec()).is_err() {
            return false;
        }
        if ctx.add_variable("subdirs", subdirs.to_vec()).is_err() {
            return false;
        }
        matches!(program.execute(&ctx), Ok(Value::Bool(true)))
    }
}
