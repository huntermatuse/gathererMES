use crate::database::equipment_types::{EquipmentTypeQueries, EquipmentTypeRow};
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct EquipmentType {
    pub type_id: Uuid,
    pub type_name: String,
    pub created_at: Option<OffsetDateTime>,
    pub updated_at: Option<OffsetDateTime>,
}


impl From<EquipmentTypeRow> for EquipmentType {
    fn from(row: EquipmentTypeRow) -> Self {
        Self {
            type_id: row.type_id,
            type_name: row.type_name,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[derive(Debug)]
pub enum EquipmentTypeServiceError {
    NotFound,
    NameAlreadyExists,
    InvalidName(String),
    Database(sqlx::Error),
}

impl std::fmt::Display for EquipmentTypeServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EquipmentTypeServiceError::NotFound => write!(f, "Equipment type not found"),
            EquipmentTypeServiceError::NameAlreadyExists => write!(f, "Equipment type name already exists"),
            EquipmentTypeServiceError::InvalidName(msg) => write!(f, "Invalid equipment type name: {}", msg),
            EquipmentTypeServiceError::Database(err) => write!(f, "Database error: {}", err),
        }
    }
}

impl std::error::Error for EquipmentTypeServiceError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            EquipmentTypeServiceError::Database(err) => Some(err),
            _ => None,
        }
    }
}

impl From<sqlx::Error> for EquipmentTypeServiceError {
    fn from(err: sqlx::Error) -> Self {
        EquipmentTypeServiceError::Database(err)
    }
}

#[derive(Debug, Clone)]
pub struct EquipmentTypeService {
    db: PgPool,
}

impl EquipmentTypeService {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    pub async fn get_all(&self) -> Result<Vec<EquipmentType>, EquipmentTypeServiceError> {
        let rows = EquipmentTypeQueries::get_all(&self.db).await?;
        Ok(rows.into_iter().map(EquipmentType::from).collect())
    }

    pub async fn get_by_id(&self, type_id: Uuid) -> Result<EquipmentType, EquipmentTypeServiceError> {
        let row = EquipmentTypeQueries::get_by_id(&self.db, type_id)
            .await?
            .ok_or(EquipmentTypeServiceError::NotFound)?;
        
        Ok(EquipmentType::from(row))
    }

    pub async fn create(&self, type_name: String) -> Result<EquipmentType, EquipmentTypeServiceError> {
        // validate input
        self.validate_type_name(&type_name)?;

        let row = EquipmentTypeQueries::create(&self.db, &type_name)
            .await
            .map_err(|e| match e {
                sqlx::Error::Database(db_err) if db_err.constraint().is_some() => {
                    EquipmentTypeServiceError::NameAlreadyExists
                }
                _ => EquipmentTypeServiceError::Database(e),
            })?;

        Ok(EquipmentType::from(row))
    }

    pub async fn update(
        &self,
        type_id: Uuid,
        type_name: String,
    ) -> Result<EquipmentType, EquipmentTypeServiceError> {
        self.validate_type_name(&type_name)?;

        let row = EquipmentTypeQueries::update(&self.db, type_id, &type_name)
            .await
            .map_err(|e| match e {
                sqlx::Error::Database(db_err) if db_err.constraint().is_some() => {
                    EquipmentTypeServiceError::NameAlreadyExists
                }
                _ => EquipmentTypeServiceError::Database(e),
            })?
            .ok_or(EquipmentTypeServiceError::NotFound)?;

        Ok(EquipmentType::from(row))
    }

    pub async fn delete(&self, type_id: Uuid) -> Result<(), EquipmentTypeServiceError> {
        let deleted = EquipmentTypeQueries::delete(&self.db, type_id).await?;
        
        if !deleted {
            return Err(EquipmentTypeServiceError::NotFound);
        }

        Ok(())
    }

    // private validation methods
    fn validate_type_name(&self, name: &str) -> Result<(), EquipmentTypeServiceError> {
        if name.trim().is_empty() {
            return Err(EquipmentTypeServiceError::InvalidName(
                "Equipment type name cannot be empty".to_string(),
            ));
        }

        if name.len() > 100 {
            return Err(EquipmentTypeServiceError::InvalidName(
                "Equipment type name cannot exceed 100 characters".to_string(),
            ));
        }

        Ok(())
    }
}