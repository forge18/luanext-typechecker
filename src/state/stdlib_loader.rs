//! Standard library parsing for the type checker
//!
//! This module provides functionality to parse TypedLua standard library
//! definition files into AST programs. The caller is responsible for
//! processing the statements (e.g., type checking, populating symbol tables).

use crate::config::LuaVersion;
use crate::diagnostics::CollectingDiagnosticHandler;
use std::sync::Arc;
use typedlua_parser::ast::Program;
use typedlua_parser::lexer::Lexer;
use typedlua_parser::parser::Parser;
use typedlua_parser::string_interner::{CommonIdentifiers, StringInterner};

/// Parses the standard library definition files for the specified Lua version.
///
/// This function reads and parses all stdlib files for the given Lua version,
/// returning the parsed AST programs. The caller is responsible for processing
/// these programs (e.g., type checking statements, populating symbol tables).
///
/// # Separation of Concerns
///
/// This function focuses solely on parsing stdlib files into AST. It does not:
/// - Perform type checking
/// - Populate symbol tables
/// - Register types or symbols
///
/// This separation allows the stdlib parser to be:
/// - Independently testable
/// - Reusable in different contexts (e.g., LSP, static analysis tools)
/// - Free from coupling to type checker internals
///
/// # Arguments
///
/// * `target_version` - The Lua version to parse stdlib for (5.1, 5.2, 5.3, or 5.4)
/// * `interner` - String interner for parsing identifiers
/// * `common` - Common identifiers for parsing
///
/// # Returns
///
/// Returns `Ok(Vec<Program>)` containing the parsed stdlib programs, or an error
/// message if parsing failed.
///
/// # Example
///
/// ```rust,ignore
/// use typedlua_typechecker::state::stdlib_loader;
/// use typedlua_typechecker::config::LuaVersion;
///
/// let (interner, common) = StringInterner::new_with_common_identifiers();
/// let programs = stdlib_loader::parse_stdlib_files(
///     LuaVersion::Lua54,
///     &interner,
///     &common
/// )?;
///
/// // Process the programs as needed
/// for mut program in programs {
///     for statement in &mut program.statements {
///         // Check statement, register types, etc.
///     }
/// }
/// ```
pub fn parse_stdlib_files(
    target_version: LuaVersion,
    interner: &StringInterner,
    common: &CommonIdentifiers,
) -> Result<Vec<Program>, String> {
    use crate::stdlib;

    let stdlib_files = stdlib::get_all_stdlib(target_version);
    let mut programs = Vec::with_capacity(stdlib_files.len());

    for (filename, source) in stdlib_files {
        let handler = Arc::new(CollectingDiagnosticHandler::new());
        let mut lexer = Lexer::new(source, handler.clone(), interner);
        let tokens = lexer
            .tokenize()
            .map_err(|e| format!("Failed to lex {}: {:?}", filename, e))?;

        let mut parser = Parser::new(tokens, handler.clone(), interner, common);
        let program = parser
            .parse()
            .map_err(|e| format!("Failed to parse {}: {:?}", filename, e))?;

        programs.push(program);
    }

    Ok(programs)
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::LuaVersion;
    use typedlua_parser::string_interner::StringInterner;

    #[test]
    fn test_parse_stdlib_lua51() {
        let (interner, common) = StringInterner::new_with_common_identifiers();

        let result = parse_stdlib_files(LuaVersion::Lua51, &interner, &common);

        assert!(result.is_ok());
        let programs = result.unwrap();
        assert!(!programs.is_empty(), "Should parse at least one stdlib file");

        // Verify programs have statements
        let total_statements: usize = programs.iter().map(|p| p.statements.len()).sum();
        assert!(
            total_statements > 0,
            "Stdlib should contain type definitions"
        );
    }

    #[test]
    fn test_parse_stdlib_all_versions() {
        let versions = vec![
            LuaVersion::Lua51,
            LuaVersion::Lua52,
            LuaVersion::Lua53,
            LuaVersion::Lua54,
        ];

        for version in versions {
            let (interner, common) = StringInterner::new_with_common_identifiers();

            let result = parse_stdlib_files(version, &interner, &common);

            assert!(
                result.is_ok(),
                "Failed to parse stdlib for version {:?}",
                version
            );

            let programs = result.unwrap();
            assert!(
                !programs.is_empty(),
                "Should parse at least one stdlib file for {:?}",
                version
            );
        }
    }

    #[test]
    fn test_parse_stdlib_returns_valid_programs() {
        let (interner, common) = StringInterner::new_with_common_identifiers();
        let programs = parse_stdlib_files(LuaVersion::Lua54, &interner, &common).unwrap();

        // Verify each program is valid
        for program in programs {
            // Programs should have statements (stdlib definitions)
            assert!(
                !program.statements.is_empty(),
                "Each stdlib file should have declarations"
            );
        }
    }
}
