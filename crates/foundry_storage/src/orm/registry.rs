use crate::orm::compiler::compile_schemas;
use crate::orm::dialect::Dialect;
use crate::orm::models::core_platform_models;
use foundry_core::orm::EntitySchema;
use std::sync::RwLock;

static CUSTOM_ORM_MODELS: RwLock<Vec<EntitySchema>> = RwLock::new(Vec::new());

/// Registers a custom code-defined ORM model schema into the global runtime registry.
pub fn register_orm_model(schema: EntitySchema) {
    let mut models = CUSTOM_ORM_MODELS
        .write()
        .expect("CUSTOM_ORM_MODELS lock poisoned");
    // Prevent duplicate registrations by table_name
    if !models.iter().any(|m| m.table_name == schema.table_name) {
        models.push(schema);
    }
}

/// Registers multiple code-defined ORM model schemas into the global runtime registry.
pub fn register_orm_models(schemas: &[EntitySchema]) {
    for schema in schemas {
        register_orm_model(schema.clone());
    }
}

/// Retrieves all registered ORM model schemas (7 platform core schemas + custom subsystem schemas).
pub fn get_all_orm_models() -> Vec<EntitySchema> {
    let mut all = core_platform_models();
    if let Ok(custom) = CUSTOM_ORM_MODELS.read() {
        for m in custom.iter() {
            if !all.iter().any(|c| c.table_name == m.table_name) {
                all.push(m.clone());
            }
        }
    }
    all
}

/// Compiles all registered ORM schemas into a complete DDL migration for the specified dialect.
pub fn compile_all_ddl(dialect: Dialect) -> String {
    let models = get_all_orm_models();
    compile_schemas(&models, dialect)
}

/// Clears custom registered ORM models (useful for testing).
pub fn clear_registered_orm_models() {
    let mut models = CUSTOM_ORM_MODELS
        .write()
        .expect("CUSTOM_ORM_MODELS lock poisoned");
    models.clear();
}
