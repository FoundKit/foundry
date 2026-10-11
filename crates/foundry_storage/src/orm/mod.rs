pub mod compiler;
pub mod dialect;
pub mod models;
pub mod query;
pub mod registry;

pub use compiler::{compile_schema, compile_schemas};
pub use dialect::Dialect;
pub use foundry_core::orm::{ColumnDef, ColumnType, DefaultValue, EntitySchema, IndexDef};
pub use models::core_platform_models;
pub use query::{OrmCondition, OrmDelete, OrmInsert, OrmSelect, OrmUpdate, OrmValue};
#[cfg(any(feature = "mysql", feature = "mariadb"))]
pub use query::{bind_mysql_query, bind_mysql_query_as};
#[cfg(feature = "postgres")]
pub use query::{bind_pg_query, bind_pg_query_as};
pub use registry::{
    clear_registered_orm_models, compile_all_ddl, get_all_orm_models, register_orm_model,
    register_orm_models,
};
