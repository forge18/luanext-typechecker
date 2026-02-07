use crate::module_resolver::error::{ModuleError, ModuleId, ModuleKind};
use std::path::PathBuf;

#[test]
fn test_module_id_creation() {
    let path = PathBuf::from("/test/module.tl");
    let module_id = ModuleId::new(path.clone());
    assert_eq!(module_id.path(), &path);
}

#[test]
fn test_module_id_as_str() {
    let path = PathBuf::from("/test/module.tl");
    let module_id = ModuleId::new(path);
    assert_eq!(module_id.as_str(), "/test/module.tl");
}

#[test]
fn test_module_id_display() {
    let path = PathBuf::from("/test/module.tl");
    let module_id = ModuleId::new(path);
    let display = format!("{}", module_id);
    assert_eq!(display, "/test/module.tl");
}

#[test]
fn test_module_id_from_pathbuf() {
    let path = PathBuf::from("/test/module.tl");
    let module_id: ModuleId = path.into();
    assert_eq!(module_id.as_str(), "/test/module.tl");
}

#[test]
fn test_module_error_not_found() {
    let error = ModuleError::NotFound {
        source: "test_module".to_string(),
        searched_paths: vec![PathBuf::from("/path1"), PathBuf::from("/path2")],
    };

    let display = format!("{}", error);
    assert!(display.contains("Cannot find module 'test_module'"));
    assert!(display.contains("Searched paths:"));
    assert!(display.contains("/path1"));
    assert!(display.contains("/path2"));
}

#[test]
fn test_module_error_circular_dependency() {
    let cycle = vec![
        ModuleId::new(PathBuf::from("a.tl")),
        ModuleId::new(PathBuf::from("b.tl")),
        ModuleId::new(PathBuf::from("c.tl")),
    ];

    let error = ModuleError::CircularDependency {
        cycle: cycle.clone(),
    };

    let display = format!("{}", error);
    assert!(display.contains("Circular dependency detected:"));
    assert!(display.contains("a.tl"));
    assert!(display.contains("b.tl"));
    assert!(display.contains("c.tl"));
    assert!(display.contains("cycle"));
}

#[test]
fn test_module_error_invalid_path() {
    let error = ModuleError::InvalidPath {
        source: "../invalid/path".to_string(),
        reason: "contains parent directory reference".to_string(),
    };

    let display = format!("{}", error);
    assert!(display
        .contains("Invalid module path '../invalid/path': contains parent directory reference"));
}

#[test]
fn test_module_error_io_error() {
    let error = ModuleError::IoError {
        path: PathBuf::from("/test/file.tl"),
        message: "Permission denied".to_string(),
    };

    let display = format!("{}", error);
    assert!(display.contains("I/O error reading '/test/file.tl': Permission denied"));
}

#[test]
fn test_module_error_not_compiled() {
    let module_id = ModuleId::new(PathBuf::from("test.tl"));
    let error = ModuleError::NotCompiled { id: module_id };

    let display = format!("{}", error);
    assert!(display.contains("Module 'test.tl' has not been compiled yet"));
}

#[test]
fn test_module_error_export_not_found() {
    let module_id = ModuleId::new(PathBuf::from("test.tl"));
    let error = ModuleError::ExportNotFound {
        module_id,
        export_name: "unknown_function".to_string(),
    };

    let display = format!("{}", error);
    assert!(display.contains("Module 'test.tl' does not export 'unknown_function'"));
}

#[test]
fn test_module_error_is_error_trait() {
    use std::error::Error;

    let error = ModuleError::NotFound {
        source: "test".to_string(),
        searched_paths: vec![],
    };

    let source = error.source();
    assert!(source.is_none());
}

#[test]
fn test_module_kind_typed_extension() {
    assert_eq!(ModuleKind::from_extension("tl"), Some(ModuleKind::Typed));
}

#[test]
fn test_module_kind_from_extension() {
    assert_eq!(ModuleKind::from_extension("tl"), Some(ModuleKind::Typed));
    assert_eq!(
        ModuleKind::from_extension("lua"),
        Some(ModuleKind::PlainLua)
    );
    assert_eq!(
        ModuleKind::from_extension(".d.tl"),
        Some(ModuleKind::Declaration)
    );
    assert_eq!(ModuleKind::from_extension("d.tl"), None);
    assert_eq!(ModuleKind::from_extension("txt"), None);
    assert_eq!(ModuleKind::from_extension("rs"), None);
    assert_eq!(ModuleKind::from_extension(""), None);
}

#[test]
fn test_module_kind_declaration_extension() {
    assert_eq!(ModuleKind::from_extension("tl"), Some(ModuleKind::Typed));
    assert_eq!(
        ModuleKind::from_extension("lua"),
        Some(ModuleKind::PlainLua)
    );
    assert_eq!(ModuleKind::from_extension("d.ts"), None);
    assert_eq!(ModuleKind::from_extension("unknown"), None);
}

#[test]
fn test_module_kind_unknown_extension() {
    assert_eq!(ModuleKind::from_extension("txt"), None);
    assert_eq!(ModuleKind::from_extension("rs"), None);
    assert_eq!(ModuleKind::from_extension(""), None);
}

#[test]
fn test_module_kind_extension_method() {
    assert_eq!(ModuleKind::Typed.extension(), "tl");
    assert_eq!(ModuleKind::Declaration.extension(), "d.tl");
    assert_eq!(ModuleKind::PlainLua.extension(), "lua");
}

#[test]
fn test_module_kind_copy() {
    let kind = ModuleKind::Typed;
    let kind_copy = kind;
    assert_eq!(kind, kind_copy);
}

#[test]
fn test_module_error_clone() {
    let error1 = ModuleError::NotFound {
        source: "test".to_string(),
        searched_paths: vec![PathBuf::from("/test")],
    };

    let error2 = error1.clone();

    assert_eq!(format!("{}", error1), format!("{}", error2));
}

#[test]
fn test_module_id_clone() {
    let id1 = ModuleId::new(PathBuf::from("/test.tl"));
    let id2 = id1.clone();

    assert_eq!(id1.as_str(), id2.as_str());
}

#[test]
fn test_module_id_hash() {
    use std::collections::HashSet;

    let id1 = ModuleId::new(PathBuf::from("/test.tl"));
    let id2 = ModuleId::new(PathBuf::from("/test.tl"));
    let id3 = ModuleId::new(PathBuf::from("/other.tl"));

    let mut set = HashSet::new();
    set.insert(id1.clone());
    set.insert(id2.clone());
    set.insert(id3);

    assert_eq!(set.len(), 2);
    assert!(set.contains(&id1));
    assert!(set.contains(&id2));
}

#[test]
fn test_module_kind_partial_eq() {
    assert_eq!(ModuleKind::Typed, ModuleKind::Typed);
    assert_eq!(ModuleKind::Declaration, ModuleKind::Declaration);
    assert_eq!(ModuleKind::PlainLua, ModuleKind::PlainLua);
    assert_ne!(ModuleKind::Typed, ModuleKind::PlainLua);
}
