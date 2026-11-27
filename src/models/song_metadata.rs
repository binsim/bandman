use serde::{Deserialize, Serialize};
#[cfg(feature = "ssr")]
use sqlx::PgPool;
use uuid::Uuid;

pub const SONG_LABEL_COLORS: &[&str] = &[
    "#2563EB", "#15803D", "#C2410C", "#BE123C", "#7E22CE", "#475569",
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct SongCategory {
    pub id: Uuid,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct SongLabel {
    pub id: Uuid,
    pub name: String,
    pub color: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SongMetadataAssignment {
    pub category: Option<SongCategory>,
    pub labels: Vec<SongLabel>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SongMetadataSelection {
    pub category_id: Option<Uuid>,
    pub label_ids: Vec<Uuid>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SongMetadataOptions {
    pub categories: Vec<SongCategory>,
    pub labels: Vec<SongLabel>,
}

#[cfg(feature = "ssr")]
#[derive(Debug, thiserror::Error)]
pub enum SongMetadataError {
    #[error("Name must be 1-60 characters and contain no control characters")]
    InvalidCategoryName,
    #[error("Name must be 1-40 characters and contain no control characters")]
    InvalidLabelName,
    #[error("Select a supported label color")]
    InvalidColor,
    #[error("A category with this name already exists")]
    DuplicateCategory,
    #[error("A label with this name already exists")]
    DuplicateLabel,
    #[error("Category or label not found")]
    NotFound,
    #[error("Category or label assignment is invalid")]
    InvalidAssignment,
    #[error(transparent)]
    Database(#[from] sqlx::Error),
}

#[cfg(feature = "ssr")]
pub(crate) async fn validate_assignment(
    pool: &PgPool,
    category_id: Option<Uuid>,
    label_ids: &[Uuid],
) -> Result<Vec<Uuid>, SongMetadataError> {
    let mut unique_labels = Vec::new();
    for label_id in label_ids {
        if !unique_labels.contains(label_id) {
            unique_labels.push(*label_id);
        }
    }

    if let Some(category_id) = category_id {
        let category_exists: bool =
            sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM wishlist_categories WHERE id = $1)")
                .bind(category_id)
                .fetch_one(pool)
                .await?;
        if !category_exists {
            return Err(SongMetadataError::InvalidAssignment);
        }
    }

    let label_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM wishlist_labels WHERE id = ANY($1)")
            .bind(&unique_labels)
            .fetch_one(pool)
            .await?;
    if label_count != unique_labels.len() as i64 {
        return Err(SongMetadataError::InvalidAssignment);
    }

    Ok(unique_labels)
}

#[cfg(feature = "ssr")]
impl SongCategory {
    pub async fn list(pool: &PgPool) -> Result<Vec<Self>, SongMetadataError> {
        Ok(sqlx::query_as::<_, Self>(
            "SELECT id, name FROM wishlist_categories ORDER BY lower(name), name",
        )
        .fetch_all(pool)
        .await?)
    }

    pub async fn create(pool: &PgPool, name: &str) -> Result<Self, SongMetadataError> {
        let name = validate_name(name, 60, true)?;
        sqlx::query_as::<_, Self>(
            "INSERT INTO wishlist_categories (id, name) VALUES ($1, $2) RETURNING id, name",
        )
        .bind(Uuid::new_v4())
        .bind(name)
        .fetch_one(pool)
        .await
        .map_err(map_category_error)
    }

    pub async fn update(pool: &PgPool, id: Uuid, name: &str) -> Result<Self, SongMetadataError> {
        let name = validate_name(name, 60, true)?;
        sqlx::query_as::<_, Self>(
            "UPDATE wishlist_categories SET name = $1 WHERE id = $2 RETURNING id, name",
        )
        .bind(name)
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(map_category_error)?
        .ok_or(SongMetadataError::NotFound)
    }

    pub async fn delete(pool: &PgPool, id: Uuid) -> Result<(), SongMetadataError> {
        let deleted = sqlx::query("DELETE FROM wishlist_categories WHERE id = $1 RETURNING id")
            .bind(id)
            .fetch_optional(pool)
            .await?;
        if deleted.is_some() {
            Ok(())
        } else {
            Err(SongMetadataError::NotFound)
        }
    }
}

#[cfg(feature = "ssr")]
impl SongLabel {
    pub async fn list(pool: &PgPool) -> Result<Vec<Self>, SongMetadataError> {
        Ok(sqlx::query_as::<_, Self>(
            "SELECT id, name, color FROM wishlist_labels ORDER BY lower(name), name",
        )
        .fetch_all(pool)
        .await?)
    }

    pub async fn create(pool: &PgPool, name: &str, color: &str) -> Result<Self, SongMetadataError> {
        let name = validate_name(name, 40, false)?;
        let color = validate_color(color)?;
        sqlx::query_as::<_, Self>(
            "INSERT INTO wishlist_labels (id, name, color) VALUES ($1, $2, $3) RETURNING id, name, color",
        )
        .bind(Uuid::new_v4())
        .bind(name)
        .bind(color)
        .fetch_one(pool)
        .await
        .map_err(map_label_error)
    }

    pub async fn update(
        pool: &PgPool,
        id: Uuid,
        name: &str,
        color: &str,
    ) -> Result<Self, SongMetadataError> {
        let name = validate_name(name, 40, false)?;
        let color = validate_color(color)?;
        sqlx::query_as::<_, Self>(
            "UPDATE wishlist_labels SET name = $1, color = $2 WHERE id = $3 RETURNING id, name, color",
        )
        .bind(name)
        .bind(color)
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(map_label_error)?
        .ok_or(SongMetadataError::NotFound)
    }

    pub async fn delete(pool: &PgPool, id: Uuid) -> Result<(), SongMetadataError> {
        let deleted = sqlx::query("DELETE FROM wishlist_labels WHERE id = $1 RETURNING id")
            .bind(id)
            .fetch_optional(pool)
            .await?;
        if deleted.is_some() {
            Ok(())
        } else {
            Err(SongMetadataError::NotFound)
        }
    }
}

#[cfg(feature = "ssr")]
fn validate_name(
    name: &str,
    max_length: usize,
    is_category: bool,
) -> Result<String, SongMetadataError> {
    let name = name.trim();
    if name.is_empty() || name.chars().count() > max_length || name.chars().any(char::is_control) {
        return Err(if is_category {
            SongMetadataError::InvalidCategoryName
        } else {
            SongMetadataError::InvalidLabelName
        });
    }
    Ok(name.to_string())
}

#[cfg(feature = "ssr")]
fn validate_color(color: &str) -> Result<&'static str, SongMetadataError> {
    SONG_LABEL_COLORS
        .iter()
        .copied()
        .find(|candidate| *candidate == color)
        .ok_or(SongMetadataError::InvalidColor)
}

#[cfg(feature = "ssr")]
fn map_category_error(error: sqlx::Error) -> SongMetadataError {
    if matches!(&error, sqlx::Error::Database(database_error)
        if database_error.constraint() == Some("wishlist_categories_name_unique"))
    {
        SongMetadataError::DuplicateCategory
    } else {
        SongMetadataError::Database(error)
    }
}

#[cfg(feature = "ssr")]
fn map_label_error(error: sqlx::Error) -> SongMetadataError {
    if matches!(&error, sqlx::Error::Database(database_error)
        if database_error.constraint() == Some("wishlist_labels_name_unique"))
    {
        SongMetadataError::DuplicateLabel
    } else {
        SongMetadataError::Database(error)
    }
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::*;

    #[test]
    fn label_color_must_be_supported() {
        assert!(validate_color("#2563EB").is_ok());
        assert!(matches!(
            validate_color("red"),
            Err(SongMetadataError::InvalidColor)
        ));
    }

    #[test]
    fn names_are_trimmed_and_bounded() {
        assert_eq!(validate_name("  Party  ", 60, true).unwrap(), "Party");
        assert!(validate_name("  ", 60, true).is_err());
        assert!(validate_name("a".repeat(61).as_str(), 60, true).is_err());
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn categories_and_labels_are_managed(pool: PgPool) {
        let category = SongCategory::create(&pool, "Party").await.unwrap();
        let category = SongCategory::update(&pool, category.id, "Dancing")
            .await
            .unwrap();
        assert_eq!(category.name, "Dancing");

        let label = SongLabel::create(&pool, "2 guitars", "#2563EB")
            .await
            .unwrap();
        assert_eq!(SongLabel::list(&pool).await.unwrap(), [label.clone()]);
        SongLabel::delete(&pool, label.id).await.unwrap();
        SongCategory::delete(&pool, category.id).await.unwrap();
        assert!(SongLabel::list(&pool).await.unwrap().is_empty());
        assert!(SongCategory::list(&pool).await.unwrap().is_empty());
    }
}
