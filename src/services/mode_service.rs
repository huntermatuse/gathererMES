use anyhow::{Context, anyhow};
use sqlx::PgPool;
use time::OffsetDateTime;
use tracing::{debug, error, instrument};
use uuid::Uuid;

use crate::database::modes::{ModeRow, ModeRowQueries};

#[derive(Debug, Clone)]
pub struct Mode {
    pub mode_id: Uuid,
    pub mode_group_id: Uuid,
    pub mode_description: String,
    pub created_at: Option<OffsetDateTime>,
    pub updated_at: Option<OffsetDateTime>,
}

impl From<ModeRow> for Mode {
    fn from(row: ModeRow) -> Self {
        Self {
            mode_id: row.mode_id,
            mode_group_id: row.mode_group_id,
            mode_description: row.mode_description,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[derive(Clone)]
pub struct ModeService {
    db: PgPool,
}

impl ModeService {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    #[instrument(skip(self))]
    pub async fn get_all(&self) -> anyhow::Result<Vec<Mode>> {
        debug!("Fetching all modes");

        let rows = ModeRowQueries::get_all(&self.db)
            .await
            .context("Failed to fetch all modes")?;

        let modes: Vec<Mode> = rows.into_iter().map(Mode::from).collect();

        debug!("Retrieved {} modes", modes.len());
        Ok(modes)
    }

    #[instrument(skip(self), fields(offset = %offset, limit = %limit))]
    pub async fn get_paginated(&self, offset: i64, limit: i64) -> anyhow::Result<(Vec<Mode>, i64)> {
        debug!(
            "Fetching paginated modes: offset={}, limit={}",
            offset, limit
        );

        // Validate pagination parameters
        if offset < 0 {
            return Err(anyhow!("Offset cannot be negative"));
        }
        if limit <= 0 || limit > 1000 {
            return Err(anyhow!("Limit must be between 1 and 1000"));
        }

        // Get total count
        let total_count = sqlx::query_scalar!("SELECT COUNT(*) FROM core.mode")
            .fetch_one(&self.db)
            .await
            .context("Failed to get total mode count")?
            .unwrap_or(0);

        // Get paginated results
        let rows = sqlx::query_as!(
            ModeRow,
            r#"SELECT mode_id, mode_group_id, mode_description, created_at, updated_at
               FROM core.mode
               ORDER BY mode_description
               LIMIT $1 OFFSET $2"#,
            limit,
            offset
        )
        .fetch_all(&self.db)
        .await
        .context("Failed to fetch paginated modes")?;

        let modes: Vec<Mode> = rows.into_iter().map(Mode::from).collect();

        debug!("Retrieved {} modes (total: {})", modes.len(), total_count);
        Ok((modes, total_count))
    }

    #[instrument(skip(self), fields(id = %mode_id))]
    pub async fn get_by_id(&self, mode_id: Uuid) -> anyhow::Result<Mode> {
        debug!("Fetching mode by id: {}", mode_id);

        let row = ModeRowQueries::get_by_mode_id(&self.db, mode_id)
            .await
            .context("Failed to fetch mode by id")?
            .ok_or_else(|| anyhow!("Mode with id '{}' not found", mode_id))?;

        let mode = Mode::from(row);
        debug!("Retrieved mode: {}", mode.mode_description);
        Ok(mode)
    }

    #[instrument(skip(self), fields(description = %mode_description))]
    pub async fn get_by_description(&self, mode_description: &str) -> anyhow::Result<Option<Mode>> {
        debug!("Fetching mode by description: {}", mode_description);

        let row = ModeRowQueries::get_by_mode_description(&self.db, mode_description)
            .await
            .context("Failed to fetch mode by description")?;

        match row {
            Some(row) => {
                let mode = Mode::from(row);
                debug!("Found mode: {}", mode.mode_description);
                Ok(Some(mode))
            }
            None => {
                debug!("Mode with description '{}' not found", mode_description);
                Ok(None)
            }
        }
    }

    #[instrument(skip(self), fields(group_id = %mode_group_id))]
    pub async fn get_by_mode_group_id(&self, mode_group_id: Uuid) -> anyhow::Result<Vec<Mode>> {
        debug!("Fetching modes by mode_group_id: {}", mode_group_id);

        let rows = ModeRowQueries::get_modes_for_group(&self.db, mode_group_id)
            .await
            .context("Failed to fetch modes by mode_group_id")?;

        let modes: Vec<Mode> = rows.into_iter().map(Mode::from).collect();

        debug!(
            "Retrieved {} modes for group {}",
            modes.len(),
            mode_group_id
        );
        Ok(modes)
    }

    #[instrument(skip(self), fields(search_term = %search_term))]
    pub async fn search_by_description(&self, search_term: &str) -> anyhow::Result<Vec<Mode>> {
        debug!("Searching modes by description: {}", search_term);

        if search_term.trim().is_empty() {
            return Err(anyhow!("Search term cannot be empty"));
        }

        let rows = ModeRowQueries::search_by_description(&self.db, search_term)
            .await
            .context("Failed to search modes by description")?;

        let modes: Vec<Mode> = rows.into_iter().map(Mode::from).collect();

        debug!(
            "Found {} modes matching search term '{}'",
            modes.len(),
            search_term
        );
        Ok(modes)
    }

    #[instrument(skip(self), fields(group_id = %mode_group_id, description = %mode_description))]
    pub async fn create(
        &self,
        mode_group_id: Uuid,
        mode_description: &str,
    ) -> anyhow::Result<Mode> {
        debug!("Creating mode with description: {}", mode_description);

        let row = ModeRowQueries::create_mode(&self.db, mode_group_id, mode_description)
            .await
            .context("Failed to create mode")?;

        let mode = Mode::from(row);
        debug!("Successfully created mode: {}", mode.mode_description);
        Ok(mode)
    }

    #[instrument(skip(self))]
    pub async fn bulk_create(&self, mode_data: Vec<(Uuid, &str)>) -> anyhow::Result<Vec<Mode>> {
        debug!("Bulk creating {} modes", mode_data.len());

        if mode_data.is_empty() {
            return Err(anyhow!("No modes provided for bulk creation"));
        }

        if mode_data.len() > 100 {
            return Err(anyhow!("Cannot create more than 100 modes at once"));
        }

        let mut created_modes = Vec::new();

        // Use the pool directly for bulk operations since our queries expect &PgPool
        for (mode_group_id, mode_description) in mode_data {
            match ModeRowQueries::create_mode(&self.db, mode_group_id, mode_description).await {
                Ok(row) => {
                    created_modes.push(Mode::from(row));
                }
                Err(e) => {
                    error!("Failed to create mode '{}': {}", mode_description, e);
                    // Continue with other modes, don't fail the entire batch
                }
            }
        }

        debug!("Successfully bulk created {} modes", created_modes.len());
        Ok(created_modes)
    }

    #[instrument(skip(self), fields(id = %mode_id, new_description = %mode_description))]
    pub async fn update_description(
        &self,
        mode_id: Uuid,
        mode_description: &str,
    ) -> anyhow::Result<Mode> {
        debug!("Updating mode description for id: {}", mode_id);

        let row = ModeRowQueries::update_mode_description(&self.db, mode_id, mode_description)
            .await
            .context("Failed to update mode description")?
            .ok_or_else(|| anyhow!("Mode with id '{}' not found", mode_id))?;

        let mode = Mode::from(row);
        debug!(
            "Successfully updated mode description: {}",
            mode.mode_description
        );
        Ok(mode)
    }

    #[instrument(skip(self), fields(id = %mode_id, new_group_id = %mode_group_id))]
    pub async fn update_mode_group(
        &self,
        mode_id: Uuid,
        mode_group_id: Uuid,
    ) -> anyhow::Result<Mode> {
        debug!("Updating mode group for id: {}", mode_id);

        let row = ModeRowQueries::update_mode_group(&self.db, mode_id, mode_group_id)
            .await
            .context("Failed to update mode group")?
            .ok_or_else(|| anyhow!("Mode with id '{}' not found", mode_id))?;

        let mode = Mode::from(row);
        debug!(
            "Successfully updated mode group for mode: {}",
            mode.mode_description
        );
        Ok(mode)
    }

    #[instrument(skip(self), fields(id = %mode_id))]
    pub async fn delete(&self, mode_id: Uuid) -> anyhow::Result<()> {
        debug!("Deleting mode: {}", mode_id);

        let deleted = ModeRowQueries::delete_mode(&self.db, mode_id)
            .await
            .context("Failed to delete mode")?;

        if !deleted {
            return Err(anyhow!("Mode with id '{}' not found", mode_id));
        }

        debug!("Successfully deleted mode: {}", mode_id);
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn count(&self) -> anyhow::Result<i64> {
        debug!("Getting total mode count");

        let count = sqlx::query_scalar!("SELECT COUNT(*) FROM core.mode")
            .fetch_one(&self.db)
            .await
            .context("Failed to get mode count")?
            .unwrap_or(0);

        debug!("Total modes: {}", count);
        Ok(count)
    }

    #[instrument(skip(self), fields(id = %mode_id))]
    pub async fn exists(&self, mode_id: Uuid) -> anyhow::Result<bool> {
        debug!("Checking if mode exists: {}", mode_id);

        let exists = ModeRowQueries::exists(&self.db, mode_id)
            .await
            .context("Failed to check if mode exists")?;

        debug!("Mode {} exists: {}", mode_id, exists);
        Ok(exists)
    }

    #[instrument(skip(self), fields(group_id = %mode_group_id, description = %mode_description))]
    pub async fn description_exists_in_group(
        &self,
        mode_group_id: Uuid,
        mode_description: &str,
    ) -> anyhow::Result<bool> {
        debug!(
            "Checking if description exists in group: {} - {}",
            mode_group_id, mode_description
        );

        let exists =
            ModeRowQueries::description_exists_in_group(&self.db, mode_group_id, mode_description)
                .await
                .context("Failed to check if description exists in group")?;

        debug!(
            "Description '{}' exists in group {}: {}",
            mode_description, mode_group_id, exists
        );
        Ok(exists)
    }

    #[instrument(skip(self), fields(group_id = %mode_group_id))]
    pub async fn count_by_mode_group(&self, mode_group_id: Uuid) -> anyhow::Result<i64> {
        debug!("Getting mode count for group: {}", mode_group_id);

        let count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM core.mode WHERE mode_group_id = $1",
            mode_group_id
        )
        .fetch_one(&self.db)
        .await
        .context("Failed to get mode count by group")?
        .unwrap_or(0);

        debug!("Modes in group {}: {}", mode_group_id, count);
        Ok(count)
    }

    #[instrument(skip(self), fields(group_id = %mode_group_id))]
    pub async fn get_paginated_by_mode_group(
        &self,
        mode_group_id: Uuid,
        offset: i64,
        limit: i64,
    ) -> anyhow::Result<(Vec<Mode>, i64)> {
        debug!(
            "Fetching paginated modes for group: {}, offset={}, limit={}",
            mode_group_id, offset, limit
        );

        // Validate pagination parameters
        if offset < 0 {
            return Err(anyhow!("Offset cannot be negative"));
        }
        if limit <= 0 || limit > 1000 {
            return Err(anyhow!("Limit must be between 1 and 1000"));
        }

        // Check if mode group exists
        let group_exists = sqlx::query_scalar!(
            "SELECT EXISTS(SELECT 1 FROM core.mode_group WHERE mode_group_id = $1)",
            mode_group_id
        )
        .fetch_one(&self.db)
        .await
        .context("Failed to check if mode_group exists")?;

        if !group_exists.unwrap_or(false) {
            return Err(anyhow!("mode_group_id '{}' does not exist", mode_group_id));
        }

        // Get total count for this group
        let total_count = self.count_by_mode_group(mode_group_id).await?;

        // Get paginated results
        let rows = sqlx::query_as!(
            ModeRow,
            r#"SELECT mode_id, mode_group_id, mode_description, created_at, updated_at
               FROM core.mode
               WHERE mode_group_id = $1
               ORDER BY mode_description
               LIMIT $2 OFFSET $3"#,
            mode_group_id,
            limit,
            offset
        )
        .fetch_all(&self.db)
        .await
        .context("Failed to fetch paginated modes by group")?;

        let modes: Vec<Mode> = rows.into_iter().map(Mode::from).collect();

        debug!(
            "Retrieved {} modes for group {} (total: {})",
            modes.len(),
            mode_group_id,
            total_count
        );
        Ok((modes, total_count))
    }

    /// Validates that a mode group exists before performing operations
    #[instrument(skip(self), fields(group_id = %mode_group_id))]
    pub async fn validate_mode_group_exists(&self, mode_group_id: Uuid) -> anyhow::Result<()> {
        debug!("Validating mode group exists: {}", mode_group_id);

        let exists = sqlx::query_scalar!(
            "SELECT EXISTS(SELECT 1 FROM core.mode_group WHERE mode_group_id = $1)",
            mode_group_id
        )
        .fetch_one(&self.db)
        .await
        .context("Failed to check if mode_group exists")?;

        if !exists.unwrap_or(false) {
            return Err(anyhow!("mode_group_id '{}' does not exist", mode_group_id));
        }

        Ok(())
    }

    /// Get modes that match multiple criteria
    #[instrument(skip(self))]
    pub async fn search_with_filters(
        &self,
        mode_group_id: Option<Uuid>,
        description_search: Option<&str>,
        offset: i64,
        limit: i64,
    ) -> anyhow::Result<(Vec<Mode>, i64)> {
        debug!(
            "Searching modes with filters: group_id={:?}, search={:?}",
            mode_group_id, description_search
        );

        if offset < 0 {
            return Err(anyhow!("Offset cannot be negative"));
        }
        if limit <= 0 || limit > 1000 {
            return Err(anyhow!("Limit must be between 1 and 1000"));
        }

        // Build dynamic query with parameter index tracking
        let mut where_conditions = Vec::new();
        let mut param_index = 0;
        let mut bind_group_id = None;
        let mut bind_description = None;

        if let Some(group_id) = mode_group_id {
            param_index += 1;
            where_conditions.push(format!("mode_group_id = ${}", param_index));
            bind_group_id = Some(group_id);
        }

        if let Some(search) = description_search {
            if !search.trim().is_empty() {
                param_index += 1;
                where_conditions.push(format!(
                    "lower(mode_description::text COLLATE \"C\") LIKE ${}",
                    param_index
                ));
                bind_description = Some(format!("%{}%", search.trim().to_lowercase()));
            }
        }

        let where_clause = if where_conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", where_conditions.join(" AND "))
        };

        // --- Count query ---
        let count_query = format!("SELECT COUNT(*) FROM core.mode {}", &where_clause);
        let mut count_stmt = sqlx::query_scalar::<_, i64>(&count_query);

        if let Some(gid) = bind_group_id {
            count_stmt = count_stmt.bind(gid);
        }
        if let Some(desc) = &bind_description {
            count_stmt = count_stmt.bind(desc);
        }

        let total_count = count_stmt.fetch_one(&self.db).await?;

        // --- Data query ---
        param_index += 1;
        let limit_param = param_index;
        param_index += 1;
        let offset_param = param_index;

        let data_query = format!(
            "SELECT mode_id, mode_group_id, mode_description, created_at, updated_at \
         FROM core.mode {} ORDER BY mode_description LIMIT ${} OFFSET ${}",
            &where_clause, limit_param, offset_param
        );

        let mut data_stmt = sqlx::query_as::<_, ModeRow>(&data_query);

        if let Some(gid) = bind_group_id {
            data_stmt = data_stmt.bind(gid);
        }
        if let Some(desc) = &bind_description {
            data_stmt = data_stmt.bind(desc);
        }
        data_stmt = data_stmt.bind(limit).bind(offset);

        let rows = data_stmt.fetch_all(&self.db).await?;
        let modes: Vec<Mode> = rows.into_iter().map(Mode::from).collect();

        debug!(
            "Found {} modes with filters (total: {})",
            modes.len(),
            total_count
        );

        Ok((modes, total_count))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    // Helper to create a test mode group for testing
    async fn create_test_mode_group(pool: &PgPool) -> sqlx::Result<Uuid> {
        let unique_name = format!("Test Group {}", Uuid::new_v4());
        let result = sqlx::query_scalar!(
            r#"INSERT INTO core.mode_group (mode_group_name, mode_group_description)
               VALUES ($1, 'Test Group Description')
               RETURNING mode_group_id"#,
            unique_name
        )
        .fetch_one(pool)
        .await?;

        Ok(result)
    }

    #[sqlx::test]
    async fn test_create_mode_service(pool: PgPool) -> sqlx::Result<()> {
        let service = ModeService::new(pool);
        // Just verify the service can be created
        assert!(true);
        Ok(())
    }

    #[sqlx::test]
    async fn test_create_mode(pool: PgPool) -> sqlx::Result<()> {
        let service = ModeService::new(pool.clone());
        let mode_group_id = create_test_mode_group(&pool).await?;

        let mode = service
            .create(mode_group_id, "Test Mode Description")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        assert_eq!(mode.mode_group_id, mode_group_id);
        assert_eq!(mode.mode_description, "Test Mode Description");
        assert!(mode.mode_id != Uuid::nil());

        Ok(())
    }

    #[sqlx::test]
    async fn test_get_by_id(pool: PgPool) -> sqlx::Result<()> {
        let service = ModeService::new(pool.clone());
        let mode_group_id = create_test_mode_group(&pool).await?;

        let created = service
            .create(mode_group_id, "Test Mode")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        let retrieved = service
            .get_by_id(created.mode_id)
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        assert_eq!(retrieved.mode_id, created.mode_id);
        assert_eq!(retrieved.mode_description, "Test Mode");

        Ok(())
    }

    #[sqlx::test]
    async fn test_get_by_id_not_found(pool: PgPool) -> sqlx::Result<()> {
        let service = ModeService::new(pool);
        let random_id = Uuid::new_v4();

        let result = service.get_by_id(random_id).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));

        Ok(())
    }

    #[sqlx::test]
    async fn test_get_by_description(pool: PgPool) -> sqlx::Result<()> {
        let service = ModeService::new(pool.clone());
        let mode_group_id = create_test_mode_group(&pool).await?;
        let description = "Unique Test Description";

        let _created = service
            .create(mode_group_id, description)
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        let found = service
            .get_by_description(description)
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.mode_description, description);

        Ok(())
    }

    #[sqlx::test]
    async fn test_get_by_mode_group_id(pool: PgPool) -> sqlx::Result<()> {
        let service = ModeService::new(pool.clone());
        let mode_group_id = create_test_mode_group(&pool).await?;

        let mode1 = service
            .create(mode_group_id, "Mode 1")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
        let mode2 = service
            .create(mode_group_id, "Mode 2")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        let modes = service
            .get_by_mode_group_id(mode_group_id)
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        assert_eq!(modes.len(), 2);
        assert!(modes.iter().any(|m| m.mode_id == mode1.mode_id));
        assert!(modes.iter().any(|m| m.mode_id == mode2.mode_id));

        Ok(())
    }

    #[sqlx::test]
    async fn test_search_by_description(pool: PgPool) -> sqlx::Result<()> {
        let service = ModeService::new(pool.clone());
        let mode_group_id = create_test_mode_group(&pool).await?;

        let _pump = service
            .create(mode_group_id, "Water Pump Mode")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
        let _motor = service
            .create(mode_group_id, "Electric Motor Mode")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        let pump_results = service
            .search_by_description("pump")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        assert!(pump_results.len() >= 1);
        assert!(
            pump_results
                .iter()
                .any(|m| m.mode_description.contains("Pump"))
        );

        Ok(())
    }

    #[sqlx::test]
    async fn test_update_description(pool: PgPool) -> sqlx::Result<()> {
        let service = ModeService::new(pool.clone());
        let mode_group_id = create_test_mode_group(&pool).await?;

        let created = service
            .create(mode_group_id, "Original Description")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        let updated = service
            .update_description(created.mode_id, "Updated Description")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        assert_eq!(updated.mode_id, created.mode_id);
        assert_eq!(updated.mode_description, "Updated Description");

        Ok(())
    }

    #[sqlx::test]
    async fn test_update_mode_group(pool: PgPool) -> sqlx::Result<()> {
        let service = ModeService::new(pool.clone());
        let group1_id = create_test_mode_group(&pool).await?;
        let group2_id = create_test_mode_group(&pool).await?;

        let created = service
            .create(group1_id, "Test Mode")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        let updated = service
            .update_mode_group(created.mode_id, group2_id)
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        assert_eq!(updated.mode_id, created.mode_id);
        assert_eq!(updated.mode_group_id, group2_id);

        Ok(())
    }

    #[sqlx::test]
    async fn test_delete_mode(pool: PgPool) -> sqlx::Result<()> {
        let service = ModeService::new(pool.clone());
        let mode_group_id = create_test_mode_group(&pool).await?;

        let created = service
            .create(mode_group_id, "To Delete")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        service
            .delete(created.mode_id)
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        // Verify it's gone
        let result = service.get_by_id(created.mode_id).await;
        assert!(result.is_err());

        Ok(())
    }

    #[sqlx::test]
    async fn test_count(pool: PgPool) -> sqlx::Result<()> {
        let service = ModeService::new(pool.clone());
        let mode_group_id = create_test_mode_group(&pool).await?;

        let initial_count = service
            .count()
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        let _created = service
            .create(mode_group_id, "Count Test")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        let new_count = service
            .count()
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        assert_eq!(new_count, initial_count + 1);

        Ok(())
    }

    #[sqlx::test]
    async fn test_exists(pool: PgPool) -> sqlx::Result<()> {
        let service = ModeService::new(pool.clone());
        let mode_group_id = create_test_mode_group(&pool).await?;

        let created = service
            .create(mode_group_id, "Exists Test")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        let exists = service
            .exists(created.mode_id)
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
        assert!(exists);

        let random_id = Uuid::new_v4();
        let not_exists = service
            .exists(random_id)
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
        assert!(!not_exists);

        Ok(())
    }

    #[sqlx::test]
    async fn test_paginated_results(pool: PgPool) -> sqlx::Result<()> {
        let service = ModeService::new(pool.clone());
        let mode_group_id = create_test_mode_group(&pool).await?;

        // Create test data
        for i in 1..=5 {
            let _ = service
                .create(mode_group_id, &format!("Pagination Mode {}", i))
                .await
                .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
        }

        let (modes, total_count) = service
            .get_paginated(0, 3)
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        assert!(modes.len() <= 3);
        assert!(total_count >= 5);

        Ok(())
    }

    #[sqlx::test]
    async fn test_bulk_create(pool: PgPool) -> sqlx::Result<()> {
        let service = ModeService::new(pool.clone());
        let mode_group_id = create_test_mode_group(&pool).await?;

        let mode_data = vec![
            (mode_group_id, "Bulk Mode 1"),
            (mode_group_id, "Bulk Mode 2"),
            (mode_group_id, "Bulk Mode 3"),
        ];

        let created_modes = service
            .bulk_create(mode_data)
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        assert_eq!(created_modes.len(), 3);
        assert!(
            created_modes
                .iter()
                .any(|m| m.mode_description == "Bulk Mode 1")
        );
        assert!(
            created_modes
                .iter()
                .any(|m| m.mode_description == "Bulk Mode 2")
        );
        assert!(
            created_modes
                .iter()
                .any(|m| m.mode_description == "Bulk Mode 3")
        );

        Ok(())
    }

    #[sqlx::test]
    async fn test_description_exists_in_group(pool: PgPool) -> sqlx::Result<()> {
        let service = ModeService::new(pool.clone());
        let mode_group_id = create_test_mode_group(&pool).await?;
        let description = "Exists Test Description";

        let _created = service
            .create(mode_group_id, description)
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        let exists = service
            .description_exists_in_group(mode_group_id, description)
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
        assert!(exists);

        let not_exists = service
            .description_exists_in_group(mode_group_id, "Does Not Exist")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
        assert!(!not_exists);

        Ok(())
    }

    #[sqlx::test]
    async fn test_count_by_mode_group(pool: PgPool) -> sqlx::Result<()> {
        let service = ModeService::new(pool.clone());
        let mode_group_id = create_test_mode_group(&pool).await?;

        let initial_count = service
            .count_by_mode_group(mode_group_id)
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        let _created = service
            .create(mode_group_id, "Count Test")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        let new_count = service
            .count_by_mode_group(mode_group_id)
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        assert_eq!(new_count, initial_count + 1);

        Ok(())
    }

    #[sqlx::test]
    async fn test_get_paginated_by_mode_group(pool: PgPool) -> sqlx::Result<()> {
        let service = ModeService::new(pool.clone());
        let mode_group_id = create_test_mode_group(&pool).await?;

        // Create test data
        for i in 1..=5 {
            let _ = service
                .create(mode_group_id, &format!("Group Mode {}", i))
                .await
                .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
        }

        let (modes, total_count) = service
            .get_paginated_by_mode_group(mode_group_id, 0, 3)
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        assert!(modes.len() <= 3);
        assert!(total_count >= 5);
        assert!(modes.iter().all(|m| m.mode_group_id == mode_group_id));

        Ok(())
    }

    #[sqlx::test]
    async fn test_search_with_filters(pool: PgPool) -> sqlx::Result<()> {
        let service = ModeService::new(pool.clone());
        let mode_group_id = create_test_mode_group(&pool).await?;

        let _pump = service
            .create(mode_group_id, "Water Pump Mode")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
        let _motor = service
            .create(mode_group_id, "Electric Motor Mode")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        // Search with group filter only
        let (group_modes, _) = service
            .search_with_filters(Some(mode_group_id), None, 0, 10)
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        assert!(group_modes.len() >= 2);
        assert!(group_modes.iter().all(|m| m.mode_group_id == mode_group_id));

        // Search with description filter
        let (pump_modes, _) = service
            .search_with_filters(Some(mode_group_id), Some("pump"), 0, 10)
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        assert!(pump_modes.len() >= 1);
        assert!(
            pump_modes
                .iter()
                .any(|m| m.mode_description.contains("Pump"))
        );
        Ok(())
    }

    #[sqlx::test]
    async fn test_validate_mode_group_exists(pool: PgPool) -> sqlx::Result<()> {
        let service = ModeService::new(pool.clone());
        let mode_group_id = create_test_mode_group(&pool).await?;

        // Valid group should pass
        let result = service.validate_mode_group_exists(mode_group_id).await;
        assert!(result.is_ok());

        // Invalid group should fail
        let invalid_id = Uuid::new_v4();
        let result = service.validate_mode_group_exists(invalid_id).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("does not exist"));

        Ok(())
    }

    #[sqlx::test]
    async fn test_get_all_modes(pool: PgPool) -> sqlx::Result<()> {
        let service = ModeService::new(pool.clone());
        let mode_group_id = create_test_mode_group(&pool).await?;

        let mode1 = service
            .create(mode_group_id, "All Mode 1")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
        let mode2 = service
            .create(mode_group_id, "All Mode 2")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        let all_modes = service
            .get_all()
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        assert!(all_modes.len() >= 2);
        assert!(all_modes.iter().any(|m| m.mode_id == mode1.mode_id));
        assert!(all_modes.iter().any(|m| m.mode_id == mode2.mode_id));

        Ok(())
    }

    #[sqlx::test]
    async fn test_error_handling(pool: PgPool) -> sqlx::Result<()> {
        let service = ModeService::new(pool);

        // Test invalid pagination parameters
        let result = service.get_paginated(-1, 10).await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("cannot be negative")
        );

        let result = service.get_paginated(0, 0).await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("must be between 1 and 1000")
        );

        let result = service.get_paginated(0, 2000).await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("must be between 1 and 1000")
        );

        // Test empty search term
        let result = service.search_by_description("").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot be empty"));

        let result = service.search_by_description("   ").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot be empty"));

        // Test empty bulk create
        let result = service.bulk_create(vec![]).await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("No modes provided")
        );

        // Test too many bulk create
        let too_many: Vec<(Uuid, &str)> = (0..101).map(|i| (Uuid::new_v4(), "test")).collect();
        let result = service.bulk_create(too_many).await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Cannot create more than 100")
        );

        Ok(())
    }
}
