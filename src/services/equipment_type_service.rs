use crate::database::equipment_types::{EquipmentTypeQueries, EquipmentTypeRow};
use anyhow::{Context, Result, anyhow};
use sqlx::PgPool;
use time::OffsetDateTime;
use tracing::{debug, instrument};
use uuid::Uuid;

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

#[derive(Debug, Clone)]
pub struct EquipmentTypeService {
    db: PgPool,
}

impl EquipmentTypeService {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    #[instrument(skip(self))]
    pub async fn get_all(&self) -> Result<Vec<EquipmentType>> {
        debug!("Fetching all equipment types");
        let rows = EquipmentTypeQueries::get_all(&self.db)
            .await
            .context("Failed to fetch all equipment types")?;
        let types: Vec<EquipmentType> = rows.into_iter().map(EquipmentType::from).collect();
        debug!("Found {} equipment types", types.len());
        Ok(types)
    }

    #[instrument(skip(self), fields(type_id = %type_id))]
    pub async fn get_by_id(&self, type_id: Uuid) -> Result<EquipmentType> {
        debug!("Fetching equipment type by ID");
        let row = EquipmentTypeQueries::get_by_id(&self.db, type_id)
            .await
            .context("Failed to fetch equipment type by ID")?
            .ok_or_else(|| anyhow!("Equipment type with ID {} not found", type_id))?;

        debug!("Found equipment type: {}", row.type_name);
        Ok(EquipmentType::from(row))
    }

    #[instrument(skip(self), fields(type_name = %type_name))]
    pub async fn get_by_name(&self, type_name: &str) -> Result<Option<EquipmentType>> {
        debug!("Fetching equipment type by name");
        let row = EquipmentTypeQueries::get_by_name(&self.db, type_name)
            .await
            .context("Failed to fetch equipment type by name")?;

        match row {
            Some(r) => {
                debug!("Found equipment type: {}", r.type_name);
                Ok(Some(EquipmentType::from(r)))
            }
            None => {
                debug!("Equipment type not found");
                Ok(None)
            }
        }
    }

    #[instrument(skip(self), fields(type_name = %type_name))]
    pub async fn create(&self, type_name: &str) -> Result<EquipmentType> {
        debug!("Creating new equipment type");

        let row = EquipmentTypeQueries::create(&self.db, type_name)
            .await
            .map_err(|e| {
                // Check if it's a duplicate error and preserve the original message
                if e.to_string().contains("already exists") {
                    e // Return the original error to preserve "already exists" message
                } else {
                    // For other errors, add context
                    e.context(format!("Failed to create equipment type '{}'", type_name))
                }
            })?;

        debug!("Successfully created equipment type: {}", row.type_name);
        Ok(EquipmentType::from(row))
    }

    #[instrument(skip(self), fields(type_id = %type_id, type_name = %type_name))]
    pub async fn update(&self, type_id: Uuid, type_name: &str) -> Result<EquipmentType> {
        debug!("Updating equipment type");

        let row = EquipmentTypeQueries::update(&self.db, type_id, type_name)
            .await
            .with_context(|| format!("Failed to update equipment type {}", type_id))?
            .ok_or_else(|| anyhow!("Equipment type with ID {} not found", type_id))?;

        debug!("Successfully updated equipment type: {}", row.type_name);
        Ok(EquipmentType::from(row))
    }

    #[instrument(skip(self), fields(type_id = %type_id))]
    pub async fn delete(&self, type_id: Uuid) -> Result<()> {
        debug!("Deleting equipment type");

        // TODO: Add check to see if equipment type is in use
        // This would prevent deletion of types that are actively used
        // Example:
        // if self.is_in_use(type_id).await? {
        //     return Err(anyhow!("Equipment type is in use and cannot be deleted"));
        // }

        let deleted = EquipmentTypeQueries::delete(&self.db, type_id)
            .await
            .with_context(|| format!("Failed to delete equipment type {}", type_id))?;

        if !deleted {
            return Err(anyhow!("Equipment type with ID {} not found", type_id));
        }

        debug!("Successfully deleted equipment type");
        Ok(())
    }

    #[instrument(skip(self), fields(type_id = %type_id))]
    pub async fn exists(&self, type_id: Uuid) -> Result<bool> {
        let exists = EquipmentTypeQueries::exists(&self.db, type_id)
            .await
            .context("Failed to check if equipment type exists")?;
        debug!("Equipment type {} exists: {}", type_id, exists);
        Ok(exists)
    }

    #[instrument(skip(self), fields(type_name = %type_name))]
    pub async fn name_exists(&self, type_name: &str) -> Result<bool> {
        let exists = EquipmentTypeQueries::name_exists(&self.db, type_name)
            .await
            .context("Failed to check if equipment type name exists")?;
        debug!("Equipment type name '{}' exists: {}", type_name, exists);
        Ok(exists)
    }

    #[instrument(skip(self), fields(search_term = %search_term))]
    pub async fn search_by_name(&self, search_term: &str) -> Result<Vec<EquipmentType>> {
        debug!("Searching equipment types by name");
        let rows = EquipmentTypeQueries::search_by_name(&self.db, search_term)
            .await
            .context("Failed to search equipment types by name")?;
        let types: Vec<EquipmentType> = rows.into_iter().map(EquipmentType::from).collect();
        debug!("Found {} equipment types matching search", types.len());
        Ok(types)
    }

    // TODO: Implement this method to check if an equipment type is in use
    // #[instrument(skip(self), fields(type_id = %type_id))]
    // pub async fn is_in_use(&self, type_id: Uuid) -> Result<bool> {
    //     // This would check if the equipment type is referenced in other tables
    //     // Example: equipment, maintenance_records, etc.
    //     // let count = sqlx::query_scalar!(
    //     //     "SELECT COUNT(*) FROM equipment WHERE type_id = $1",
    //     //     type_id
    //     // )
    //     // .fetch_one(&self.db)
    //     // .await
    //     // .context("Failed to check equipment type usage")?;
    //     //
    //     // Ok(count.unwrap_or(0) > 0)
    //     Ok(false)
    // }

    /// Bulk create equipment types (useful for initial setup or imports)
    #[instrument(skip(self, type_names))]
    pub async fn bulk_create(&self, type_names: Vec<&str>) -> Result<Vec<EquipmentType>> {
        debug!("Bulk creating {} equipment types", type_names.len());
        let mut created_types = Vec::new();
        let mut errors = Vec::new();

        for type_name in type_names {
            match self.create(type_name).await {
                Ok(equipment_type) => {
                    created_types.push(equipment_type);
                }
                Err(e) if e.to_string().contains("already exists") => {
                    // Skip duplicates, optionally log warning
                    debug!("Skipping duplicate equipment type: {}", type_name);
                }
                Err(e) => {
                    errors.push(format!("Failed to create '{}': {}", type_name, e));
                }
            }
        }

        if !errors.is_empty() {
            return Err(anyhow!(
                "Bulk create failed with errors: {}",
                errors.join(", ")
            ));
        }

        debug!(
            "Successfully created {} equipment types",
            created_types.len()
        );
        Ok(created_types)
    }

    /// Get equipment types with pagination
    #[instrument(skip(self))]
    pub async fn get_paginated(
        &self,
        offset: i64,
        limit: i64,
    ) -> Result<(Vec<EquipmentType>, i64)> {
        debug!(
            "Fetching equipment types with pagination: offset={}, limit={}",
            offset, limit
        );

        if offset < 0 {
            return Err(anyhow!("Offset cannot be negative"));
        }

        if limit <= 0 || limit > 1000 {
            return Err(anyhow!("Limit must be between 1 and 1000"));
        }

        // Get total count
        let total_count = sqlx::query_scalar!("SELECT COUNT(*) FROM core.equipment_type")
            .fetch_one(&self.db)
            .await
            .context("Failed to get total count of equipment types")?
            .unwrap_or(0);

        // Get paginated results
        let rows = sqlx::query_as!(
            EquipmentTypeRow,
            r#"SELECT type_id, type_name, created_at, updated_at
               FROM core.equipment_type
               ORDER BY type_name
               LIMIT $1 OFFSET $2"#,
            limit,
            offset
        )
        .fetch_all(&self.db)
        .await
        .context("Failed to fetch paginated equipment types")?;

        let types: Vec<EquipmentType> = rows.into_iter().map(EquipmentType::from).collect();
        debug!(
            "Found {} equipment types (total: {})",
            types.len(),
            total_count
        );

        Ok((types, total_count))
    }

    /// Get equipment types created within a date range
    #[instrument(skip(self))]
    pub async fn get_by_date_range(
        &self,
        start_date: OffsetDateTime,
        end_date: OffsetDateTime,
    ) -> Result<Vec<EquipmentType>> {
        debug!("Fetching equipment types by date range");

        if start_date > end_date {
            return Err(anyhow!("Start date cannot be after end date"));
        }

        let rows = sqlx::query_as!(
            EquipmentTypeRow,
            r#"SELECT type_id, type_name, created_at, updated_at
               FROM core.equipment_type
               WHERE created_at >= $1 AND created_at <= $2
               ORDER BY created_at DESC"#,
            start_date,
            end_date
        )
        .fetch_all(&self.db)
        .await
        .context("Failed to fetch equipment types by date range")?;

        let types: Vec<EquipmentType> = rows.into_iter().map(EquipmentType::from).collect();
        debug!("Found {} equipment types in date range", types.len());
        Ok(types)
    }

    /// Get count of all equipment types
    #[instrument(skip(self))]
    pub async fn count(&self) -> Result<i64> {
        let count = sqlx::query_scalar!("SELECT COUNT(*) FROM core.equipment_type")
            .fetch_one(&self.db)
            .await
            .context("Failed to count equipment types")?
            .unwrap_or(0);

        debug!("Total equipment types count: {}", count);
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;

    #[sqlx::test]
    async fn test_service_create_and_get(pool: PgPool) -> sqlx::Result<()> {
        let service = EquipmentTypeService::new(pool);

        // Create
        let created = service
            .create("Test Service Type")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        assert_eq!(created.type_name, "Test Service Type");
        assert!(created.type_id != Uuid::nil());

        // Get by ID
        let found = service
            .get_by_id(created.type_id)
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        assert_eq!(found.type_id, created.type_id);
        assert_eq!(found.type_name, "Test Service Type");

        Ok(())
    }

    #[sqlx::test]
    async fn test_service_validation(pool: PgPool) -> sqlx::Result<()> {
        let service = EquipmentTypeService::new(pool);

        // Test empty name
        let result = service.create("").await;
        assert!(result.is_err());

        // Test whitespace name
        let result = service.create("   ").await;
        assert!(result.is_err());

        Ok(())
    }

    #[sqlx::test]
    async fn test_service_duplicate_handling_debug(pool: PgPool) -> sqlx::Result<()> {
        let service = EquipmentTypeService::new(pool);

        // Create first
        let first = service
            .create("Duplicate Test")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        println!("Created first equipment type: {:?}", first);

        // Check if it exists
        let exists = service
            .name_exists("Duplicate Test")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        println!("Name exists check: {}", exists);

        // Try to create duplicate
        let result = service.create("Duplicate Test").await;

        match &result {
            Ok(equipment_type) => {
                println!("ERROR: Duplicate creation succeeded when it should have failed!");
                println!("Created: {:?}", equipment_type);
            }
            Err(e) => {
                println!("Duplicate creation failed as expected with error: {}", e);
                println!(
                    "Error contains 'already exists': {}",
                    e.to_string().contains("already exists")
                );
            }
        }

        assert!(result.is_err(), "Expected duplicate creation to fail");

        let error = result.unwrap_err();
        let error_string = error.to_string();

        if !error_string.contains("already exists") {
            println!("ERROR: Error message doesn't contain 'already exists'");
            println!("Actual error message: '{}'", error_string);
            return Err(sqlx::Error::Protocol(format!(
                "Expected error to contain 'already exists', but got: '{}'",
                error_string
            )));
        }

        Ok(())
    }

    #[sqlx::test]
    async fn test_service_search(pool: PgPool) -> sqlx::Result<()> {
        let service = EquipmentTypeService::new(pool);

        // Create test data
        let _pump = service
            .create("Water Pump")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
        let _motor = service
            .create("Electric Motor")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        // Search
        let results = service
            .search_by_name("pump")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        assert!(results.len() >= 1);
        assert!(results.iter().any(|t| t.type_name.contains("Pump")));

        Ok(())
    }

    #[sqlx::test]
    async fn test_service_bulk_create(pool: PgPool) -> sqlx::Result<()> {
        let service = EquipmentTypeService::new(pool);

        let type_names = vec!["Bulk Type 1", "Bulk Type 2", "Bulk Type 3"];
        let created = service
            .bulk_create(type_names)
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        assert_eq!(created.len(), 3);
        assert!(created.iter().any(|t| t.type_name == "Bulk Type 1"));
        assert!(created.iter().any(|t| t.type_name == "Bulk Type 2"));
        assert!(created.iter().any(|t| t.type_name == "Bulk Type 3"));

        Ok(())
    }

    #[sqlx::test]
    async fn test_service_pagination(pool: PgPool) -> sqlx::Result<()> {
        let service = EquipmentTypeService::new(pool);

        // Create test data
        for i in 1..=5 {
            let _ = service
                .create(&format!("Pagination Type {}", i))
                .await
                .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
        }

        // Test pagination
        let (page1, total) = service
            .get_paginated(0, 3)
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        assert!(page1.len() <= 3);
        assert!(total >= 5);

        let (page2, _) = service
            .get_paginated(3, 3)
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        assert!(page2.len() >= 0);

        Ok(())
    }

    #[sqlx::test]
    async fn test_service_pagination_validation(pool: PgPool) -> sqlx::Result<()> {
        let service = EquipmentTypeService::new(pool);

        // Test negative offset
        let result = service.get_paginated(-1, 10).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("negative"));

        // Test invalid limit
        let result = service.get_paginated(0, 0).await;
        assert!(result.is_err());

        let result = service.get_paginated(0, 2000).await;
        assert!(result.is_err());

        Ok(())
    }

    #[sqlx::test]
    async fn test_service_count(pool: PgPool) -> sqlx::Result<()> {
        let service = EquipmentTypeService::new(pool);

        let initial_count = service
            .count()
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        // Create some types
        let _ = service
            .create("Count Test 1")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
        let _ = service
            .create("Count Test 2")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        let new_count = service
            .count()
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        assert_eq!(new_count, initial_count + 2);

        Ok(())
    }
}
