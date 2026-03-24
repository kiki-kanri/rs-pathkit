//! SeaORM integration for [`Path`].
//!
//! `Path` is stored as `String` in the database.
//! The following traits are implemented to allow `Path` as a model field:
//!
//! - [`Into<Value>`][1] — enables `ActiveValue::Set(Path(...))` and query parameters
//! - [`ValueType`][2] — describes the column type to the schema machinery
//! - [`Nullable`][3] — enables `Option<Path>` in models
//! - [`TryGetable`][4] — enables reading `Path` from query results
//!
//! [1]: Value
//! [2]: ValueType
//! [3]: Nullable
//! [4]: sea_orm::TryGetable
//!
//! # Example
//!
//! ```rust,ignore
//! use sea_orm::entity::prelude::*;
//! use pathkit::Path;
//!
//! #[derive(Clone, Debug, DeriveEntityModel)]
//! #[sea_orm(table_name = "files")]
//! struct Model {
//!     #[sea_orm(primary_key)]
//!     id: i32,
//!     path: Path,
//! }
//! ```

use sea_orm::{
    prelude::*,
    sea_query::{
        ArrayType,
        Nullable,
        ValueType,
        ValueTypeErr,
    },
    ColIdx,
    QueryResult,
    TryGetError,
    TryGetable,
};

use crate::Path;

// ---------------------------------------------------------------------------
// Into<Value> — allows ActiveValue::Set(Path(...)) and query parameters
// ---------------------------------------------------------------------------

impl From<Path> for Value {
    fn from(val: Path) -> Self {
        Value::String(Some(Box::new(val.to_string_lossy().into_owned())))
    }
}

// ---------------------------------------------------------------------------
// Nullable — enables Option<Path> in models
// ---------------------------------------------------------------------------

impl Nullable for Path {
    fn null() -> Value {
        Value::String(None)
    }
}

// ---------------------------------------------------------------------------
// TryGetable — enables reading Path from query results
// ---------------------------------------------------------------------------

impl TryGetable for Path {
    fn try_get_by<I: ColIdx>(result: &QueryResult, idx: I) -> Result<Self, TryGetError> {
        let string = <String as TryGetable>::try_get_by(result, idx)?;
        Ok(Self::new(string))
    }
}

// ---------------------------------------------------------------------------
// ValueType — describes the column type to the schema machinery
// ---------------------------------------------------------------------------

impl ValueType for Path {
    fn try_from(v: Value) -> Result<Self, ValueTypeErr> {
        match v {
            Value::String(Some(s)) => Ok(Self::new(s.as_str())),
            Value::String(None) => Err(ValueTypeErr),
            _ => Err(ValueTypeErr),
        }
    }

    fn type_name() -> String {
        stringify!(Path).to_owned()
    }

    fn array_type() -> ArrayType {
        ArrayType::String
    }

    fn column_type() -> ColumnType {
        ColumnType::String(StringLen::None)
    }
}

// ---------------------------------------------------------------------------
// tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use tempfile::{
        tempdir,
        NamedTempFile,
    };

    use super::*;

    // -------------------------------------------------------------------
    // Compile-check models using DeriveEntityModel with Path fields
    // -------------------------------------------------------------------

    mod file_model {
        use sea_orm::entity::*;

        use super::*;

        #[derive(Clone, Debug, DeriveEntityModel)]
        #[sea_orm(table_name = "files")]
        pub struct Model {
            #[sea_orm(primary_key)]
            id: i32,
            path: Path,
            display_name: Option<Path>,
        }

        #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
        pub enum Relation {}

        impl ActiveModelBehavior for ActiveModel {}
    }

    // -----------------------------------------------------------------------
    // Entity with multiple Path fields
    // -----------------------------------------------------------------------

    mod asset_model {
        use sea_orm::entity::*;

        use super::*;

        #[derive(Clone, Debug, DeriveEntityModel)]
        #[sea_orm(table_name = "assets")]
        pub struct Model {
            #[sea_orm(primary_key)]
            id: i32,
            source_path: Path,
            dest_path: Path,
            maybe_path: Option<Path>,
        }

        #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
        pub enum Relation {}

        impl ActiveModelBehavior for ActiveModel {}
    }

    // -----------------------------------------------------------------------
    // Trait tests using tempfile
    // -----------------------------------------------------------------------

    #[test]
    fn test_into_value_roundtrip_tempdir() {
        let temp_dir = tempdir().unwrap();
        let path = Path::new(temp_dir.path());

        // Path -> Value -> Path roundtrip
        let value: Value = path.clone().into();
        let recovered = <Path as ValueType>::try_from(value).unwrap();
        assert_eq!(recovered.to_str(), path.to_str());
    }

    #[test]
    fn test_path_column_type_is_string() {
        assert_eq!(<Path as ValueType>::column_type(), ColumnType::String(StringLen::None));
        assert_eq!(<Path as ValueType>::array_type(), ArrayType::String);
        assert_eq!(<Path as ValueType>::type_name(), "Path");
    }

    #[test]
    fn test_nullable_null_is_string_none() {
        let null: Value = <Path as Nullable>::null();
        assert!(matches!(null, Value::String(None)));
    }

    #[test]
    fn test_value_type_err_on_int() {
        let value = Value::Int(Some(42));
        let result: Result<Path, _> = ValueType::try_from(value);
        assert!(result.is_err());
    }

    #[test]
    fn test_value_type_err_on_null() {
        let value = Value::String(None);
        let result: Result<Path, _> = ValueType::try_from(value);
        assert!(result.is_err());
    }

    #[test]
    fn test_path_with_unicode() {
        let temp_dir = tempdir().unwrap();
        let path = Path::new(temp_dir.path().join("文件/日本語.txt"));

        let value: Value = path.clone().into();
        let recovered = <Path as ValueType>::try_from(value).unwrap();
        assert_eq!(recovered.to_str(), path.to_str());
    }

    #[test]
    fn test_into_value_named_temp_file() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = Path::new(temp_file.path());

        let value: Value = path.clone().into();
        assert!(
            matches!(value, Value::String(Some(_))),
            "expected Value::String(Some(...)), got {value:?}"
        );
    }

    // -------------------------------------------------------------------
    // Compile-check models using DeriveEntityModel with Path fields
    // -------------------------------------------------------------------

    #[test]
    fn test_file_model_path_column_is_string() {
        use file_model::Entity as FileEntity;

        // Column enum variant exists and is accessible — compile-check only
        let _col = <FileEntity as EntityTrait>::Column::Path;
    }

    #[test]
    fn test_file_model_option_path_column_is_string() {
        use file_model::Entity as FileEntity;

        let _col = <FileEntity as EntityTrait>::Column::DisplayName;
    }

    #[test]
    fn test_asset_model_multiple_paths() {
        use asset_model::Entity as AssetEntity;

        // All Path columns compile correctly
        let _ = <AssetEntity as EntityTrait>::Column::SourcePath;
        let _ = <AssetEntity as EntityTrait>::Column::DestPath;
        let _ = <AssetEntity as EntityTrait>::Column::MaybePath;
    }
}
