use crate::database::mode_groups::{ModeGroupQueries, ModeGroupRow};
use anyhow::{Context, Result, anyhow};
use sqlx::PgPool;
use time::OffsetDateTime;
use tracing::{debug, instrument};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ModeGroup {
    pub mode_group_id: Uuid,
    pub mode_group_name: String,
    pub mode_group_description: String,
    pub created_at: Option<OffsetDateTime>,
    pub updated_at: Option<OffsetDateTime>,
}

impl From<ModeGroupRow> for ModeGroup {
    fn from(row: ModeGroupRow) -> Self {
        Self {
            mode_group_id: row.mode_group_id,
            mode_group_name: row.mode_group_name,
            mode_group_description: row.mode_group_description,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ModeGroupService {
    db: PgPool,
}

impl ModeGroupService {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    #[instrument(skip(self))]
    pub async fn get_all(&self) -> Result<Vec<ModeGroup>> {
        debug!("Fetching all mode groups");
        let rows = ModeGroupQueries::get_all(&self.db)
            .await
            .context("Failed to fetch all mode groups")?;
        let groups: Vec<ModeGroup> = rows.into_iter().map(ModeGroup::from).collect();
        debug!("Found {} mode groups", groups.len());
        Ok(groups)
    }

    #[instrument(skip(self), fields(mode_group_id = %mode_group_id))]
    pub async fn get_by_id(&self, mode_group_id: Uuid) -> Result<ModeGroup> {
        debug!("Fetching mode group by ID");
        let row = ModeGroupQueries::get_by_mode_group_id(&self.db, mode_group_id)
            .await
            .context("Failed to fetch mode group by ID")?
            .ok_or_else(|| anyhow!("Mode group with ID {} not found", mode_group_id))?;

        debug!("Found mode group: {}", row.mode_group_name);
        Ok(ModeGroup::from(row))
    }

    #[instrument(skip(self), fields(mode_group_name = %mode_group_name))]
    pub async fn get_by_name(&self, mode_group_name: &str) -> Result<Option<ModeGroup>> {
        debug!("Fetching mode group by name");
        let row = ModeGroupQueries::get_by_mode_group_name(&self.db, mode_group_name)
            .await
            .context("Failed to fetch mode group by name")?;

        match row {
            Some(r) => {
                debug!("Found mode group: {}", r.mode_group_name);
                Ok(Some(ModeGroup::from(r)))
            }
            None => {
                debug!("Mode group not found");
                Ok(None)
            }
        }
    }

    #[instrument(skip(self), fields(mode_group_description = %mode_group_description))]
    pub async fn get_by_description(
        &self,
        mode_group_description: &str,
    ) -> Result<Option<ModeGroup>> {
        debug!("Fetching mode group by description");
        let row = ModeGroupQueries::get_by_mode_group_description(&self.db, mode_group_description)
            .await
            .context("Failed to fetch mode group by description")?;

        match row {
            Some(r) => {
                debug!("Found mode group: {}", r.mode_group_name);
                Ok(Some(ModeGroup::from(r)))
            }
            None => {
                debug!("Mode group not found");
                Ok(None)
            }
        }
    }

    #[instrument(skip(self), fields(mode_group_name = %mode_group_name))]
    pub async fn create(
        &self,
        mode_group_name: &str,
        mode_group_description: &str,
    ) -> Result<ModeGroup> {
        debug!("Creating new mode group");

        let row =
            ModeGroupQueries::create_mode_group(&self.db, mode_group_name, mode_group_description)
                .await
                .map_err(|e| {
                    // Check if it's a duplicate error and preserve the original message
                    if e.to_string().contains("already exists") {
                        e // Return the original error to preserve "already exists" message
                    } else {
                        // For other errors, add context
                        e.context(format!("Failed to create mode group '{}'", mode_group_name))
                    }
                })?;

        debug!("Successfully created mode group: {}", row.mode_group_name);
        Ok(ModeGroup::from(row))
    }

    #[instrument(skip(self), fields(mode_group_id = %mode_group_id, mode_group_name = %mode_group_name))]
    pub async fn update_name(
        &self,
        mode_group_id: Uuid,
        mode_group_name: &str,
    ) -> Result<ModeGroup> {
        debug!("Updating mode group name");

        let row =
            ModeGroupQueries::update_mode_group_name(&self.db, mode_group_id, mode_group_name)
                .await
                .with_context(|| format!("Failed to update mode group name for {}", mode_group_id))?
                .ok_or_else(|| anyhow!("Mode group with ID {} not found", mode_group_id))?;

        debug!(
            "Successfully updated mode group name: {}",
            row.mode_group_name
        );
        Ok(ModeGroup::from(row))
    }

    #[instrument(skip(self), fields(mode_group_id = %mode_group_id))]
    pub async fn update_description(
        &self,
        mode_group_id: Uuid,
        mode_group_description: &str,
    ) -> Result<ModeGroup> {
        debug!("Updating mode group description");

        let row = ModeGroupQueries::update_mode_group_description(
            &self.db,
            mode_group_id,
            mode_group_description,
        )
        .await
        .with_context(|| {
            format!(
                "Failed to update mode group description for {}",
                mode_group_id
            )
        })?
        .ok_or_else(|| anyhow!("Mode group with ID {} not found", mode_group_id))?;

        debug!("Successfully updated mode group description");
        Ok(ModeGroup::from(row))
    }

    #[instrument(skip(self), fields(mode_group_id = %mode_group_id))]
    pub async fn delete(&self, mode_group_id: Uuid) -> Result<()> {
        debug!("Deleting mode group");

        // TODO: Add check to see if mode group is in use
        // This would prevent deletion of groups that are actively used
        // Example:
        // if self.is_in_use(mode_group_id).await? {
        //     return Err(anyhow!("Mode group is in use and cannot be deleted"));
        // }

        let deleted = ModeGroupQueries::delete_mode_group(&self.db, mode_group_id)
            .await
            .with_context(|| format!("Failed to delete mode group {}", mode_group_id))?;

        if !deleted {
            return Err(anyhow!("Mode group with ID {} not found", mode_group_id));
        }

        debug!("Successfully deleted mode group");
        Ok(())
    }

    #[instrument(skip(self), fields(mode_group_id = %mode_group_id))]
    pub async fn exists(&self, mode_group_id: Uuid) -> Result<bool> {
        let exists = ModeGroupQueries::exists(&self.db, mode_group_id)
            .await
            .context("Failed to check if mode group exists")?;
        debug!("Mode group {} exists: {}", mode_group_id, exists);
        Ok(exists)
    }

    /// Get mode groups with pagination
    #[instrument(skip(self))]
    pub async fn get_paginated(&self, offset: i64, limit: i64) -> Result<(Vec<ModeGroup>, i64)> {
        debug!(
            "Fetching mode groups with pagination: offset={}, limit={}",
            offset, limit
        );

        if offset < 0 {
            return Err(anyhow!("Offset cannot be negative"));
        }

        if limit <= 0 || limit > 1000 {
            return Err(anyhow!("Limit must be between 1 and 1000"));
        }

        // Get total count
        let total_count = sqlx::query_scalar!("SELECT COUNT(*) FROM core.mode_group")
            .fetch_one(&self.db)
            .await
            .context("Failed to get total count of mode groups")?
            .unwrap_or(0);

        // Get paginated results
        let rows = sqlx::query_as!(
            ModeGroupRow,
            r#"SELECT mode_group_id, mode_group_name, mode_group_description, created_at, updated_at
               FROM core.mode_group
               ORDER BY mode_group_name
               LIMIT $1 OFFSET $2"#,
            limit,
            offset
        )
        .fetch_all(&self.db)
        .await
        .context("Failed to fetch paginated mode groups")?;

        let groups: Vec<ModeGroup> = rows.into_iter().map(ModeGroup::from).collect();
        debug!(
            "Found {} mode groups (total: {})",
            groups.len(),
            total_count
        );

        Ok((groups, total_count))
    }

    /// Get mode groups created within a date range
    #[instrument(skip(self))]
    pub async fn get_by_date_range(
        &self,
        start_date: OffsetDateTime,
        end_date: OffsetDateTime,
    ) -> Result<Vec<ModeGroup>> {
        debug!("Fetching mode groups by date range");

        if start_date > end_date {
            return Err(anyhow!("Start date cannot be after end date"));
        }

        let rows = sqlx::query_as!(
            ModeGroupRow,
            r#"SELECT mode_group_id, mode_group_name, mode_group_description, created_at, updated_at
               FROM core.mode_group
               WHERE created_at >= $1 AND created_at <= $2
               ORDER BY created_at DESC"#,
            start_date,
            end_date
        )
        .fetch_all(&self.db)
        .await
        .context("Failed to fetch mode groups by date range")?;

        let groups: Vec<ModeGroup> = rows.into_iter().map(ModeGroup::from).collect();
        debug!("Found {} mode groups in date range", groups.len());
        Ok(groups)
    }

    /// Get count of all mode groups
    #[instrument(skip(self))]
    pub async fn count(&self) -> Result<i64> {
        let count = sqlx::query_scalar!("SELECT COUNT(*) FROM core.mode_group")
            .fetch_one(&self.db)
            .await
            .context("Failed to count mode groups")?
            .unwrap_or(0);

        debug!("Total mode groups count: {}", count);
        Ok(count)
    }

    /// Bulk create mode groups (useful for initial setup or imports)
    #[instrument(skip(self, mode_groups))]
    pub async fn bulk_create(&self, mode_groups: Vec<(&str, &str)>) -> Result<Vec<ModeGroup>> {
        debug!("Bulk creating {} mode groups", mode_groups.len());
        let mut created_groups = Vec::new();
        let mut errors = Vec::new();

        for (name, description) in mode_groups {
            match self.create(name, description).await {
                Ok(mode_group) => {
                    created_groups.push(mode_group);
                }
                Err(e) if e.to_string().contains("already exists") => {
                    // Skip duplicates, optionally log warning
                    debug!("Skipping duplicate mode group: {}", name);
                }
                Err(e) => {
                    errors.push(format!("Failed to create '{}': {}", name, e));
                }
            }
        }

        if !errors.is_empty() {
            return Err(anyhow!(
                "Bulk create failed with errors: {}",
                errors.join(", ")
            ));
        }

        debug!("Successfully created {} mode groups", created_groups.len());
        Ok(created_groups)
    }

    // TODO: Implement this method to check if a mode group is in use
    // #[instrument(skip(self), fields(mode_group_id = %mode_group_id))]
    // pub async fn is_in_use(&self, mode_group_id: Uuid) -> Result<bool> {
    //     // This would check if the mode group is referenced in other tables
    //     // Example: modes, equipment_modes, etc.
    //     // let count = sqlx::query_scalar!(
    //     //     "SELECT COUNT(*) FROM modes WHERE mode_group_id = $1",
    //     //     mode_group_id
    //     // )
    //     // .fetch_one(&self.db)
    //     // .await
    //     // .context("Failed to check mode group usage")?;
    //     //
    //     // Ok(count.unwrap_or(0) > 0)
    //     Ok(false)
    // }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;

    #[sqlx::test]
    async fn test_service_create_and_get(pool: PgPool) -> sqlx::Result<()> {
        let service = ModeGroupService::new(pool);

        // Create
        let created = service
            .create("Test Service Group", "Test Service Description")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        assert_eq!(created.mode_group_name, "Test Service Group");
        assert_eq!(created.mode_group_description, "Test Service Description");
        assert!(created.mode_group_id != Uuid::nil());

        // Get by ID
        let found = service
            .get_by_id(created.mode_group_id)
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        assert_eq!(found.mode_group_id, created.mode_group_id);
        assert_eq!(found.mode_group_name, "Test Service Group");
        assert_eq!(found.mode_group_description, "Test Service Description");

        Ok(())
    }

    #[sqlx::test]
    async fn test_service_validation(pool: PgPool) -> sqlx::Result<()> {
        let service = ModeGroupService::new(pool);

        // Test empty name
        let result = service.create("", "Valid description").await;
        assert!(result.is_err());

        // Test empty description
        let result = service.create("Valid name", "").await;
        assert!(result.is_err());

        // Test whitespace inputs
        let result = service.create("   ", "Valid description").await;
        assert!(result.is_err());

        let result = service.create("Valid name", "   ").await;
        assert!(result.is_err());

        Ok(())
    }

    #[sqlx::test]
    async fn test_service_duplicate_handling(pool: PgPool) -> sqlx::Result<()> {
        let service = ModeGroupService::new(pool);

        // Create first
        let _first = service
            .create("Duplicate Test", "Description 1")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        // Try to create duplicate
        let result = service.create("Duplicate Test", "Description 2").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already exists"));

        Ok(())
    }

    #[sqlx::test]
    async fn test_service_updates(pool: PgPool) -> sqlx::Result<()> {
        let service = ModeGroupService::new(pool);

        // Create
        let created = service
            .create("Original Name", "Original Description")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        // Update name
        let updated_name = service
            .update_name(created.mode_group_id, "Updated Name")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        assert_eq!(updated_name.mode_group_name, "Updated Name");
        assert_eq!(updated_name.mode_group_description, "Original Description");

        // Update description
        let updated_desc = service
            .update_description(created.mode_group_id, "Updated Description")
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        assert_eq!(updated_desc.mode_group_name, "Updated Name");
        assert_eq!(updated_desc.mode_group_description, "Updated Description");

        Ok(())
    }

    #[sqlx::test]
    async fn test_service_pagination(pool: PgPool) -> sqlx::Result<()> {
        let service = ModeGroupService::new(pool);

        // Create test data
        for i in 1..=5 {
            let _ = service
                .create(
                    &format!("Pagination Group {}", i),
                    &format!("Description {}", i),
                )
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

        Ok(())
    }

    #[sqlx::test]
    async fn test_service_bulk_create(pool: PgPool) -> sqlx::Result<()> {
        let service = ModeGroupService::new(pool);

        let groups = vec![
            ("Bulk Group 1", "Bulk Description 1"),
            ("Bulk Group 2", "Bulk Description 2"),
            ("Bulk Group 3", "Bulk Description 3"),
        ];

        let created = service
            .bulk_create(groups)
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

        assert_eq!(created.len(), 3);
        assert!(created.iter().any(|g| g.mode_group_name == "Bulk Group 1"));
        assert!(created.iter().any(|g| g.mode_group_name == "Bulk Group 2"));
        assert!(created.iter().any(|g| g.mode_group_name == "Bulk Group 3"));

        Ok(())
    }
}
