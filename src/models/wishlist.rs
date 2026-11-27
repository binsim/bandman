use super::song_metadata::{SongCategory, SongLabel, SongMetadataOptions};
#[cfg(feature = "ssr")]
use super::song_metadata::{
    SongMetadataAssignment, SongMetadataError, SongMetadataSelection, validate_assignment,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
#[cfg(feature = "ssr")]
use sqlx::{FromRow, PgPool, Row, postgres::PgRow};
use std::str::FromStr;
use uuid::Uuid;

pub const WISHLIST_MUSICAL_KEYS: &[&str] = &[
    "C major", "G major", "D major", "A major", "E major", "B major", "F# major", "C# major",
    "F major", "Bb major", "Eb major", "Ab major", "Db major", "Gb major", "Cb major", "A minor",
    "E minor", "B minor", "F# minor", "C# minor", "G# minor", "D# minor", "D minor", "G minor",
    "C minor", "F minor", "Bb minor", "Eb minor", "Ab minor", "A# minor",
];

#[derive(Debug, Clone, Copy)]
pub struct WishlistSongDetails<'a> {
    pub title: &'a str,
    pub artist: Option<&'a str>,
    pub link: Option<&'a str>,
    pub link_title: Option<&'a str>,
    pub tempo: Option<i32>,
    pub musical_key: Option<&'a str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WishlistCategory {
    Song,
    Medley,
    Try,
    Unplayable,
    OtherOrdering,
    NotFitMedley,
}

impl FromStr for WishlistCategory {
    type Err = std::io::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "song" => Ok(Self::Song),
            "medley" => Ok(Self::Medley),
            "try" => Ok(Self::Try),
            "unplayable" => Ok(Self::Unplayable),
            "other_ordering" => Ok(Self::OtherOrdering),
            "not_fit_medley" => Ok(Self::NotFitMedley),
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("unknown wishlist category: {value}"),
            )),
        }
    }
}

impl std::fmt::Display for WishlistCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Song => "song",
            Self::Medley => "medley",
            Self::Try => "try",
            Self::Unplayable => "unplayable",
            Self::OtherOrdering => "other_ordering",
            Self::NotFitMedley => "not_fit_medley",
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WishlistItem {
    pub id: Uuid,
    pub proposed_by_id: Option<Uuid>,
    pub medley_id: Option<Uuid>,
    pub medley_position: Option<i32>,
    pub medley_size: Option<i64>,
    pub medley_created_by_id: Option<Uuid>,
    pub medley_name: Option<String>,
    pub title: String,
    pub artist: Option<String>,
    pub link: Option<String>,
    pub link_title: Option<String>,
    pub tempo: Option<i32>,
    pub musical_key: Option<String>,
    pub targets: Vec<String>,
    pub category: WishlistCategory,
    pub song_category: Option<SongCategory>,
    pub labels: Vec<SongLabel>,
    pub medley_category: Option<SongCategory>,
    pub medley_labels: Vec<SongLabel>,
    pub feedback: Vec<WishlistFeedback>,
    pub proposed_by: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WishlistFeedback {
    pub member_id: Uuid,
    pub member_name: String,
    pub category: WishlistCategory,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WishlistData {
    pub items: Vec<WishlistItem>,
    pub metadata: SongMetadataOptions,
}

#[cfg(feature = "ssr")]
#[derive(Debug, thiserror::Error)]
pub enum WishlistError {
    #[error("Song title must be 1-120 characters and contain no control characters")]
    InvalidTitle,
    #[error("Artist must be at most 120 characters and contain no control characters")]
    InvalidArtist,
    #[error("Link must be a valid HTTP or HTTPS URL")]
    InvalidLink,
    #[error("Tempo must be between 20 and 300 BPM")]
    InvalidTempo,
    #[error("Select a valid musical key")]
    InvalidMusicalKey,
    #[error("Provide no more than eight targets, each at most 40 characters")]
    InvalidTargets,
    #[error("This song and artist are already on the wishlist")]
    DuplicateSong,
    #[error("You do not have permission to delete this song")]
    DeleteForbidden,
    #[error("You do not have permission to edit this song")]
    EditForbidden,
    #[error("Select at least two unique songs for this medley")]
    InvalidMedleySelection,
    #[error("One or more selected songs are already in a medley")]
    SongAlreadyInMedley,
    #[error("You do not have permission to change this medley")]
    MedleyForbidden,
    #[error("Medley-specific feedback can only be given for songs in a medley")]
    MedleyFeedbackRequiresMedley,
    #[error("Medley name must be 1-120 characters and contain no control characters")]
    InvalidMedleyName,
    #[error("Wishlist song not found")]
    NotFound,
    #[error(transparent)]
    Database(#[from] sqlx::Error),
    #[error(transparent)]
    Metadata(#[from] SongMetadataError),
}

#[cfg(feature = "ssr")]
impl<'r> FromRow<'r, PgRow> for WishlistItem {
    fn from_row(row: &'r PgRow) -> Result<Self, sqlx::Error> {
        let category: String = row.try_get("category")?;
        let category =
            category
                .parse::<WishlistCategory>()
                .map_err(|error| sqlx::Error::ColumnDecode {
                    index: "category".to_string(),
                    source: Box::new(error),
                })?;
        let song_category_id: Option<Uuid> = row.try_get("song_category_id")?;
        let song_category_name: Option<String> = row.try_get("song_category_name")?;
        let medley_category_id: Option<Uuid> = row.try_get("medley_category_id")?;
        let medley_category_name: Option<String> = row.try_get("medley_category_name")?;

        Ok(Self {
            id: row.try_get("id")?,
            proposed_by_id: row.try_get("proposed_by")?,
            medley_id: row.try_get("medley_id")?,
            medley_position: row.try_get("medley_position")?,
            medley_size: row.try_get("medley_size")?,
            medley_created_by_id: row.try_get("medley_created_by")?,
            medley_name: row.try_get("medley_name")?,
            title: row.try_get("title")?,
            artist: row.try_get("artist")?,
            link: row.try_get("link")?,
            link_title: row.try_get("link_title")?,
            tempo: row.try_get("tempo")?,
            musical_key: row.try_get("musical_key")?,
            targets: row.try_get("targets")?,
            category,
            song_category: song_category_id
                .zip(song_category_name)
                .map(|(id, name)| SongCategory { id, name }),
            labels: Vec::new(),
            medley_category: medley_category_id
                .zip(medley_category_name)
                .map(|(id, name)| SongCategory { id, name }),
            medley_labels: Vec::new(),
            feedback: Vec::new(),
            proposed_by: row.try_get("proposed_by_name")?,
            created_at: row.try_get("created_at")?,
        })
    }
}

#[cfg(feature = "ssr")]
impl WishlistItem {
    pub async fn list(pool: &PgPool) -> Result<Vec<Self>, WishlistError> {
        let mut items = sqlx::query_as::<_, Self>(
            r#"
            SELECT item.id, item.proposed_by, membership.medley_id, membership.position AS medley_position,
                NULLIF(count(membership.wishlist_item_id) OVER (PARTITION BY membership.medley_id), 0) AS medley_size,
                medley.created_by AS medley_created_by, medley.name AS medley_name,
                item_category.id AS song_category_id, item_category.name AS song_category_name,
                medley_category.id AS medley_category_id, medley_category.name AS medley_category_name,
                item.title, item.artist, item.link, item.tempo, item.musical_key,
                item.link_title, item.targets, item.category, item.proposed_by_name, item.created_at
            FROM wishlist_items AS item
            LEFT JOIN wishlist_medley_items AS membership ON membership.wishlist_item_id = item.id
            LEFT JOIN wishlist_medleys AS medley ON medley.id = membership.medley_id
            LEFT JOIN wishlist_categories AS item_category ON item_category.id = item.song_category_id
            LEFT JOIN wishlist_categories AS medley_category ON medley_category.id = medley.song_category_id
            ORDER BY COALESCE(lower(medley.name), lower(item.title)),
                membership.position NULLS FIRST, lower(item.title), item.title
            "#,
        )
        .fetch_all(pool)
        .await?;

        let labels = sqlx::query_as::<_, (Uuid, Uuid, String, String)>(
            r#"
            SELECT assignment.wishlist_item_id, label.id, label.name, label.color
            FROM wishlist_item_labels AS assignment
            JOIN wishlist_labels AS label ON label.id = assignment.label_id
            ORDER BY lower(label.name), label.name
            "#,
        )
        .fetch_all(pool)
        .await?;
        for (item_id, id, name, color) in labels {
            if let Some(item) = items.iter_mut().find(|item| item.id == item_id) {
                item.labels.push(SongLabel { id, name, color });
            }
        }

        let medley_labels = sqlx::query_as::<_, (Uuid, Uuid, String, String)>(
            r#"
            SELECT assignment.medley_id, label.id, label.name, label.color
            FROM wishlist_medley_labels AS assignment
            JOIN wishlist_labels AS label ON label.id = assignment.label_id
            ORDER BY lower(label.name), label.name
            "#,
        )
        .fetch_all(pool)
        .await?;
        for (medley_id, id, name, color) in medley_labels {
            for item in items
                .iter_mut()
                .filter(|item| item.medley_id == Some(medley_id))
            {
                item.medley_labels.push(SongLabel {
                    id,
                    name: name.clone(),
                    color: color.clone(),
                });
            }
        }

        let feedback = sqlx::query_as::<_, (Uuid, Uuid, String, String)>(
            r#"
            SELECT feedback.wishlist_item_id, feedback.member_id, members.name, feedback.category
            FROM wishlist_feedback AS feedback
            JOIN members ON members.id = feedback.member_id
            JOIN wishlist_items ON wishlist_items.id = feedback.wishlist_item_id
            ORDER BY lower(members.name), members.name
            "#,
        )
        .fetch_all(pool)
        .await?;

        for (item_id, member_id, member_name, category) in feedback {
            let category = category.parse::<WishlistCategory>().map_err(|error| {
                sqlx::Error::ColumnDecode {
                    index: "category".to_string(),
                    source: Box::new(error),
                }
            })?;
            if let Some(item) = items.iter_mut().find(|item| item.id == item_id) {
                item.feedback.push(WishlistFeedback {
                    member_id,
                    member_name,
                    category,
                });
            }
        }
        Ok(items)
    }

    pub async fn set_song_metadata(
        pool: &PgPool,
        item_id: Uuid,
        category_id: Option<Uuid>,
        label_ids: &[Uuid],
    ) -> Result<SongMetadataAssignment, SongMetadataError> {
        let label_ids = validate_assignment(pool, category_id, label_ids).await?;
        let mut transaction = pool.begin().await?;
        let updated = sqlx::query_scalar::<_, Uuid>(
            "UPDATE wishlist_items SET song_category_id = $1 WHERE id = $2 RETURNING id",
        )
        .bind(category_id)
        .bind(item_id)
        .fetch_optional(&mut *transaction)
        .await?;
        if updated.is_none() {
            return Err(SongMetadataError::NotFound);
        }

        sqlx::query("DELETE FROM wishlist_item_labels WHERE wishlist_item_id = $1")
            .bind(item_id)
            .execute(&mut *transaction)
            .await?;
        for label_id in label_ids {
            sqlx::query(
                "INSERT INTO wishlist_item_labels (wishlist_item_id, label_id) VALUES ($1, $2)",
            )
            .bind(item_id)
            .bind(label_id)
            .execute(&mut *transaction)
            .await?;
        }
        transaction.commit().await?;

        Self::song_metadata_assignment(pool, item_id).await
    }

    pub async fn set_medley_metadata(
        pool: &PgPool,
        medley_id: Uuid,
        category_id: Option<Uuid>,
        label_ids: &[Uuid],
    ) -> Result<SongMetadataAssignment, SongMetadataError> {
        let label_ids = validate_assignment(pool, category_id, label_ids).await?;
        let mut transaction = pool.begin().await?;
        let updated = sqlx::query_scalar::<_, Uuid>(
            "UPDATE wishlist_medleys SET song_category_id = $1 WHERE id = $2 RETURNING id",
        )
        .bind(category_id)
        .bind(medley_id)
        .fetch_optional(&mut *transaction)
        .await?;
        if updated.is_none() {
            return Err(SongMetadataError::NotFound);
        }

        sqlx::query("DELETE FROM wishlist_medley_labels WHERE medley_id = $1")
            .bind(medley_id)
            .execute(&mut *transaction)
            .await?;
        for label_id in label_ids {
            sqlx::query("INSERT INTO wishlist_medley_labels (medley_id, label_id) VALUES ($1, $2)")
                .bind(medley_id)
                .bind(label_id)
                .execute(&mut *transaction)
                .await?;
        }
        transaction.commit().await?;

        Self::medley_metadata_assignment(pool, medley_id).await
    }

    async fn song_metadata_assignment(
        pool: &PgPool,
        item_id: Uuid,
    ) -> Result<SongMetadataAssignment, SongMetadataError> {
        let category = sqlx::query_as::<_, SongCategory>(
            "SELECT id, name FROM wishlist_categories WHERE id = (SELECT song_category_id FROM wishlist_items WHERE id = $1)",
        )
        .bind(item_id)
        .fetch_optional(pool)
        .await?;
        let labels = sqlx::query_as::<_, SongLabel>(
            r#"
            SELECT label.id, label.name, label.color
            FROM wishlist_item_labels AS assignment
            JOIN wishlist_labels AS label ON label.id = assignment.label_id
            WHERE assignment.wishlist_item_id = $1
            ORDER BY lower(label.name), label.name
            "#,
        )
        .bind(item_id)
        .fetch_all(pool)
        .await?;
        Ok(SongMetadataAssignment { category, labels })
    }

    async fn medley_metadata_assignment(
        pool: &PgPool,
        medley_id: Uuid,
    ) -> Result<SongMetadataAssignment, SongMetadataError> {
        let category = sqlx::query_as::<_, SongCategory>(
            "SELECT id, name FROM wishlist_categories WHERE id = (SELECT song_category_id FROM wishlist_medleys WHERE id = $1)",
        )
        .bind(medley_id)
        .fetch_optional(pool)
        .await?;
        let labels = sqlx::query_as::<_, SongLabel>(
            r#"
            SELECT label.id, label.name, label.color
            FROM wishlist_medley_labels AS assignment
            JOIN wishlist_labels AS label ON label.id = assignment.label_id
            WHERE assignment.medley_id = $1
            ORDER BY lower(label.name), label.name
            "#,
        )
        .bind(medley_id)
        .fetch_all(pool)
        .await?;
        Ok(SongMetadataAssignment { category, labels })
    }

    pub async fn create_medley(
        pool: &PgPool,
        item_ids: &[Uuid],
        name: &str,
        member_id: Uuid,
    ) -> Result<Uuid, WishlistError> {
        let name = validate_medley_name(name)?;
        if item_ids.len() < 2 {
            return Err(WishlistError::InvalidMedleySelection);
        }
        let unique_ids: std::collections::HashSet<_> = item_ids.iter().copied().collect();
        if unique_ids.len() != item_ids.len() {
            return Err(WishlistError::InvalidMedleySelection);
        }

        let mut transaction = pool.begin().await?;
        let existing_ids: Vec<Uuid> =
            sqlx::query_scalar("SELECT id FROM wishlist_items WHERE id = ANY($1) FOR UPDATE")
                .bind(item_ids)
                .fetch_all(&mut *transaction)
                .await?;
        if existing_ids.len() != item_ids.len() {
            return Err(WishlistError::InvalidMedleySelection);
        }
        let already_grouped: bool = sqlx::query_scalar(
            "SELECT EXISTS (SELECT 1 FROM wishlist_medley_items WHERE wishlist_item_id = ANY($1))",
        )
        .bind(item_ids)
        .fetch_one(&mut *transaction)
        .await?;
        if already_grouped {
            return Err(WishlistError::SongAlreadyInMedley);
        }

        let medley_id = Uuid::new_v4();
        sqlx::query("INSERT INTO wishlist_medleys (id, created_by, name) VALUES ($1, $2, $3)")
            .bind(medley_id)
            .bind(member_id)
            .bind(name)
            .execute(&mut *transaction)
            .await?;
        for (position, item_id) in item_ids.iter().enumerate() {
            sqlx::query(
                "INSERT INTO wishlist_medley_items (medley_id, wishlist_item_id, position) VALUES ($1, $2, $3)",
            )
            .bind(medley_id)
            .bind(item_id)
            .bind(position as i32)
            .execute(&mut *transaction)
            .await
            .map_err(map_medley_unique_violation)?;
        }
        transaction.commit().await?;
        Ok(medley_id)
    }

    pub async fn update_medley_name(
        pool: &PgPool,
        medley_id: Uuid,
        name: &str,
        member_id: Uuid,
        is_admin: bool,
    ) -> Result<(), WishlistError> {
        let name = validate_medley_name(name)?;
        let updated = sqlx::query_scalar::<_, Uuid>(
            "UPDATE wishlist_medleys SET name = $1 WHERE id = $2 AND (created_by = $3 OR $4) RETURNING id",
        )
        .bind(name)
        .bind(medley_id)
        .bind(member_id)
        .bind(is_admin)
        .fetch_optional(pool)
        .await?;
        if updated.is_some() {
            return Ok(());
        }
        let exists: bool =
            sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM wishlist_medleys WHERE id = $1)")
                .bind(medley_id)
                .fetch_one(pool)
                .await?;
        Err(if exists {
            WishlistError::MedleyForbidden
        } else {
            WishlistError::NotFound
        })
    }

    pub async fn update_medley(
        pool: &PgPool,
        medley_id: Uuid,
        item_ids: &[Uuid],
        name: &str,
        member_id: Uuid,
        is_admin: bool,
    ) -> Result<(), WishlistError> {
        Self::update_medley_inner(pool, medley_id, item_ids, name, member_id, is_admin, None).await
    }

    pub async fn update_medley_with_metadata(
        pool: &PgPool,
        medley_id: Uuid,
        item_ids: &[Uuid],
        name: &str,
        member_id: Uuid,
        is_admin: bool,
        metadata: SongMetadataSelection,
    ) -> Result<SongMetadataAssignment, WishlistError> {
        let label_ids =
            validate_assignment(pool, metadata.category_id, &metadata.label_ids).await?;
        Self::update_medley_inner(
            pool,
            medley_id,
            item_ids,
            name,
            member_id,
            is_admin,
            Some((metadata.category_id, label_ids)),
        )
        .await?;
        Self::medley_metadata_assignment(pool, medley_id)
            .await
            .map_err(Into::into)
    }

    async fn update_medley_inner(
        pool: &PgPool,
        medley_id: Uuid,
        item_ids: &[Uuid],
        name: &str,
        member_id: Uuid,
        is_admin: bool,
        metadata: Option<(Option<Uuid>, Vec<Uuid>)>,
    ) -> Result<(), WishlistError> {
        let name = validate_medley_name(name)?;
        if item_ids.len() < 2
            || item_ids
                .iter()
                .copied()
                .collect::<std::collections::HashSet<_>>()
                .len()
                != item_ids.len()
        {
            return Err(WishlistError::InvalidMedleySelection);
        }

        let mut transaction = pool.begin().await?;
        let medley_exists = sqlx::query_scalar::<_, Uuid>(
            "SELECT id FROM wishlist_medleys WHERE id = $1 AND (created_by = $2 OR $3) FOR UPDATE",
        )
        .bind(medley_id)
        .bind(member_id)
        .bind(is_admin)
        .fetch_optional(&mut *transaction)
        .await?;
        if medley_exists.is_none() {
            let exists: bool =
                sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM wishlist_medleys WHERE id = $1)")
                    .bind(medley_id)
                    .fetch_one(&mut *transaction)
                    .await?;
            return Err(if exists {
                WishlistError::MedleyForbidden
            } else {
                WishlistError::NotFound
            });
        }

        let existing_songs: Vec<Uuid> = sqlx::query_scalar(
            "SELECT id FROM wishlist_items WHERE id = ANY($1) ORDER BY id FOR UPDATE",
        )
        .bind(item_ids)
        .fetch_all(&mut *transaction)
        .await?;
        if existing_songs.len() != item_ids.len() {
            return Err(WishlistError::InvalidMedleySelection);
        }
        let belongs_to_another_medley: bool = sqlx::query_scalar(
            r#"
            SELECT EXISTS (
                SELECT 1 FROM wishlist_medley_items
                WHERE wishlist_item_id = ANY($1) AND medley_id <> $2
            )
            "#,
        )
        .bind(item_ids)
        .bind(medley_id)
        .fetch_one(&mut *transaction)
        .await?;
        if belongs_to_another_medley {
            return Err(WishlistError::SongAlreadyInMedley);
        }

        sqlx::query("UPDATE wishlist_medleys SET name = $1 WHERE id = $2")
            .bind(name)
            .bind(medley_id)
            .execute(&mut *transaction)
            .await?;
        if let Some((category_id, label_ids)) = metadata {
            sqlx::query("UPDATE wishlist_medleys SET song_category_id = $1 WHERE id = $2")
                .bind(category_id)
                .bind(medley_id)
                .execute(&mut *transaction)
                .await?;
            sqlx::query("DELETE FROM wishlist_medley_labels WHERE medley_id = $1")
                .bind(medley_id)
                .execute(&mut *transaction)
                .await?;
            for label_id in label_ids {
                sqlx::query(
                    "INSERT INTO wishlist_medley_labels (medley_id, label_id) VALUES ($1, $2)",
                )
                .bind(medley_id)
                .bind(label_id)
                .execute(&mut *transaction)
                .await?;
            }
        }
        sqlx::query("DELETE FROM wishlist_medley_items WHERE medley_id = $1")
            .bind(medley_id)
            .execute(&mut *transaction)
            .await?;
        for (position, item_id) in item_ids.iter().enumerate() {
            sqlx::query(
                "INSERT INTO wishlist_medley_items (medley_id, wishlist_item_id, position) VALUES ($1, $2, $3)",
            )
            .bind(medley_id)
            .bind(item_id)
            .bind(position as i32)
            .execute(&mut *transaction)
            .await
            .map_err(map_medley_unique_violation)?;
        }
        transaction.commit().await?;
        Ok(())
    }

    pub async fn add_medley_items(
        pool: &PgPool,
        medley_id: Uuid,
        item_ids: &[Uuid],
        member_id: Uuid,
        is_admin: bool,
    ) -> Result<(), WishlistError> {
        if item_ids.is_empty()
            || item_ids
                .iter()
                .copied()
                .collect::<std::collections::HashSet<_>>()
                .len()
                != item_ids.len()
        {
            return Err(WishlistError::InvalidMedleySelection);
        }
        let mut transaction = pool.begin().await?;
        let medley_exists = sqlx::query_scalar::<_, Uuid>(
            "SELECT id FROM wishlist_medleys WHERE id = $1 AND (created_by = $2 OR $3) FOR UPDATE",
        )
        .bind(medley_id)
        .bind(member_id)
        .bind(is_admin)
        .fetch_optional(&mut *transaction)
        .await?;
        if medley_exists.is_none() {
            let exists: bool =
                sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM wishlist_medleys WHERE id = $1)")
                    .bind(medley_id)
                    .fetch_one(&mut *transaction)
                    .await?;
            return Err(if exists {
                WishlistError::MedleyForbidden
            } else {
                WishlistError::NotFound
            });
        }
        let existing_ids: Vec<Uuid> =
            sqlx::query_scalar("SELECT id FROM wishlist_items WHERE id = ANY($1) FOR UPDATE")
                .bind(item_ids)
                .fetch_all(&mut *transaction)
                .await?;
        if existing_ids.len() != item_ids.len() {
            return Err(WishlistError::InvalidMedleySelection);
        }
        let already_grouped: bool = sqlx::query_scalar(
            "SELECT EXISTS (SELECT 1 FROM wishlist_medley_items WHERE wishlist_item_id = ANY($1))",
        )
        .bind(item_ids)
        .fetch_one(&mut *transaction)
        .await?;
        if already_grouped {
            return Err(WishlistError::SongAlreadyInMedley);
        }
        let mut position: i32 = sqlx::query_scalar(
            "SELECT COALESCE(MAX(position), -1) + 1 FROM wishlist_medley_items WHERE medley_id = $1",
        )
        .bind(medley_id)
        .fetch_one(&mut *transaction)
        .await?;
        for item_id in item_ids {
            sqlx::query(
                "INSERT INTO wishlist_medley_items (medley_id, wishlist_item_id, position) VALUES ($1, $2, $3)",
            )
            .bind(medley_id)
            .bind(item_id)
            .bind(position)
            .execute(&mut *transaction)
            .await
            .map_err(map_medley_unique_violation)?;
            position += 1;
        }
        transaction.commit().await?;
        Ok(())
    }

    pub async fn remove_medley_item(
        pool: &PgPool,
        medley_id: Uuid,
        item_id: Uuid,
        member_id: Uuid,
        is_admin: bool,
    ) -> Result<(), WishlistError> {
        let mut transaction = pool.begin().await?;
        let medley_exists = sqlx::query_scalar::<_, Uuid>(
            "SELECT id FROM wishlist_medleys WHERE id = $1 AND (created_by = $2 OR $3) FOR UPDATE",
        )
        .bind(medley_id)
        .bind(member_id)
        .bind(is_admin)
        .fetch_optional(&mut *transaction)
        .await?;
        if medley_exists.is_none() {
            let exists: bool =
                sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM wishlist_medleys WHERE id = $1)")
                    .bind(medley_id)
                    .fetch_one(&mut *transaction)
                    .await?;
            return Err(if exists {
                WishlistError::MedleyForbidden
            } else {
                WishlistError::NotFound
            });
        }
        let removed = sqlx::query(
            "DELETE FROM wishlist_medley_items WHERE medley_id = $1 AND wishlist_item_id = $2",
        )
        .bind(medley_id)
        .bind(item_id)
        .execute(&mut *transaction)
        .await?
        .rows_affected();
        if removed == 0 {
            return Err(WishlistError::NotFound);
        }
        let remaining: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM wishlist_medley_items WHERE medley_id = $1")
                .bind(medley_id)
                .fetch_one(&mut *transaction)
                .await?;
        if remaining < 2 {
            sqlx::query("DELETE FROM wishlist_medleys WHERE id = $1")
                .bind(medley_id)
                .execute(&mut *transaction)
                .await?;
        } else {
            let offset: i32 = sqlx::query_scalar(
                "SELECT COALESCE(MAX(position), 0) + 1 FROM wishlist_medley_items WHERE medley_id = $1",
            )
            .bind(medley_id)
            .fetch_one(&mut *transaction)
            .await?;
            sqlx::query(
                "UPDATE wishlist_medley_items SET position = position + $1 WHERE medley_id = $2",
            )
            .bind(offset)
            .bind(medley_id)
            .execute(&mut *transaction)
            .await?;
            sqlx::query(
                r#"
                WITH ordered AS (
                    SELECT wishlist_item_id,
                        row_number() OVER (ORDER BY position) - 1 AS new_position
                    FROM wishlist_medley_items
                    WHERE medley_id = $1
                )
                UPDATE wishlist_medley_items AS membership
                SET position = ordered.new_position::INTEGER
                FROM ordered
                WHERE membership.wishlist_item_id = ordered.wishlist_item_id
                "#,
            )
            .bind(medley_id)
            .execute(&mut *transaction)
            .await?;
        }
        transaction.commit().await?;
        Ok(())
    }

    pub async fn move_medley_item(
        pool: &PgPool,
        medley_id: Uuid,
        item_id: Uuid,
        position_delta: i32,
        member_id: Uuid,
        is_admin: bool,
    ) -> Result<Vec<Uuid>, WishlistError> {
        if !matches!(position_delta, -1 | 1) {
            return Err(WishlistError::InvalidMedleySelection);
        }

        let mut transaction = pool.begin().await?;
        let medley_exists = sqlx::query_scalar::<_, Uuid>(
            "SELECT id FROM wishlist_medleys WHERE id = $1 AND (created_by = $2 OR $3) FOR UPDATE",
        )
        .bind(medley_id)
        .bind(member_id)
        .bind(is_admin)
        .fetch_optional(&mut *transaction)
        .await?;
        if medley_exists.is_none() {
            let exists: bool =
                sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM wishlist_medleys WHERE id = $1)")
                    .bind(medley_id)
                    .fetch_one(&mut *transaction)
                    .await?;
            return Err(if exists {
                WishlistError::MedleyForbidden
            } else {
                WishlistError::NotFound
            });
        }

        let mut item_ids: Vec<Uuid> = sqlx::query_scalar(
            "SELECT wishlist_item_id FROM wishlist_medley_items WHERE medley_id = $1 ORDER BY position FOR UPDATE",
        )
        .bind(medley_id)
        .fetch_all(&mut *transaction)
        .await?;
        let Some(current_position) = item_ids.iter().position(|id| *id == item_id) else {
            return Err(WishlistError::NotFound);
        };
        let next_position = current_position as i32 + position_delta;
        if next_position >= 0 && (next_position as usize) < item_ids.len() {
            item_ids.swap(current_position, next_position as usize);
            let offset = item_ids.len() as i32;
            sqlx::query(
                "UPDATE wishlist_medley_items SET position = position + $1 WHERE medley_id = $2",
            )
            .bind(offset)
            .bind(medley_id)
            .execute(&mut *transaction)
            .await?;
            for (position, item_id) in item_ids.iter().enumerate() {
                sqlx::query(
                    "UPDATE wishlist_medley_items SET position = $1 WHERE medley_id = $2 AND wishlist_item_id = $3",
                )
                .bind(position as i32)
                .bind(medley_id)
                .bind(item_id)
                .execute(&mut *transaction)
                .await?;
            }
        }
        transaction.commit().await?;
        Ok(item_ids)
    }

    pub async fn disband_medley(
        pool: &PgPool,
        medley_id: Uuid,
        member_id: Uuid,
        is_admin: bool,
    ) -> Result<(), WishlistError> {
        let deleted = sqlx::query_scalar::<_, Uuid>(
            "DELETE FROM wishlist_medleys WHERE id = $1 AND (created_by = $2 OR $3) RETURNING id",
        )
        .bind(medley_id)
        .bind(member_id)
        .bind(is_admin)
        .fetch_optional(pool)
        .await?;
        if deleted.is_some() {
            return Ok(());
        }

        let exists: bool =
            sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM wishlist_medleys WHERE id = $1)")
                .bind(medley_id)
                .fetch_one(pool)
                .await?;
        Err(if exists {
            WishlistError::MedleyForbidden
        } else {
            WishlistError::NotFound
        })
    }

    pub async fn set_feedback(
        pool: &PgPool,
        item_id: Uuid,
        member_id: Uuid,
        category: Option<WishlistCategory>,
    ) -> Result<Option<WishlistFeedback>, WishlistError> {
        let Some(category) = category else {
            sqlx::query(
                "DELETE FROM wishlist_feedback WHERE wishlist_item_id = $1 AND member_id = $2",
            )
            .bind(item_id)
            .bind(member_id)
            .execute(pool)
            .await?;
            return Ok(None);
        };

        if matches!(
            category,
            WishlistCategory::OtherOrdering | WishlistCategory::NotFitMedley
        ) {
            let is_medley_member: bool = sqlx::query_scalar(
                "SELECT EXISTS (SELECT 1 FROM wishlist_medley_items WHERE wishlist_item_id = $1)",
            )
            .bind(item_id)
            .fetch_one(pool)
            .await?;
            if !is_medley_member {
                return Err(WishlistError::MedleyFeedbackRequiresMedley);
            }
        }

        let row: Option<(String,)> = sqlx::query_as(
            r#"
            INSERT INTO wishlist_feedback (wishlist_item_id, member_id, category)
            VALUES ($1, $2, $3)
            ON CONFLICT (wishlist_item_id, member_id)
            DO UPDATE SET category = EXCLUDED.category, updated_at = NOW()
            RETURNING (SELECT name FROM members WHERE id = $2)
            "#,
        )
        .bind(item_id)
        .bind(member_id)
        .bind(category.to_string())
        .fetch_optional(pool)
        .await?;
        let member_name = row
            .map(|(name,)| name)
            .ok_or_else(|| sqlx::Error::RowNotFound)?;
        Ok(Some(WishlistFeedback {
            member_id,
            member_name,
            category,
        }))
    }

    pub async fn delete(
        pool: &PgPool,
        item_id: Uuid,
        member_id: Uuid,
        is_admin: bool,
    ) -> Result<(), WishlistError> {
        let mut transaction = pool.begin().await?;
        let medley_id = sqlx::query_as::<_, (Option<Uuid>,)>(
            r#"
            SELECT membership.medley_id
            FROM wishlist_items AS item
            LEFT JOIN wishlist_medley_items AS membership
                ON membership.wishlist_item_id = item.id
            WHERE item.id = $1 AND (item.proposed_by = $2 OR $3)
            FOR UPDATE OF item
            "#,
        )
        .bind(item_id)
        .bind(member_id)
        .bind(is_admin)
        .fetch_optional(&mut *transaction)
        .await?;
        let Some((medley_id,)) = medley_id else {
            let exists: bool =
                sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM wishlist_items WHERE id = $1)")
                    .bind(item_id)
                    .fetch_one(&mut *transaction)
                    .await?;
            return Err(if exists {
                WishlistError::DeleteForbidden
            } else {
                WishlistError::NotFound
            });
        };

        let deleted = sqlx::query_scalar::<_, Uuid>(
            r#"
            DELETE FROM wishlist_items
            WHERE id = $1 AND (proposed_by = $2 OR $3)
            RETURNING id
            "#,
        )
        .bind(item_id)
        .bind(member_id)
        .bind(is_admin)
        .fetch_optional(&mut *transaction)
        .await?;
        if deleted.is_none() {
            return Err(WishlistError::NotFound);
        }

        if let Some(medley_id) = medley_id {
            let remaining: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM wishlist_medley_items WHERE medley_id = $1",
            )
            .bind(medley_id)
            .fetch_one(&mut *transaction)
            .await?;
            if remaining < 2 {
                sqlx::query("DELETE FROM wishlist_medleys WHERE id = $1")
                    .bind(medley_id)
                    .execute(&mut *transaction)
                    .await?;
            } else {
                let offset: i32 = sqlx::query_scalar(
                    "SELECT COALESCE(MAX(position), 0) + 1 FROM wishlist_medley_items WHERE medley_id = $1",
                )
                .bind(medley_id)
                .fetch_one(&mut *transaction)
                .await?;
                sqlx::query(
                    "UPDATE wishlist_medley_items SET position = position + $1 WHERE medley_id = $2",
                )
                .bind(offset)
                .bind(medley_id)
                .execute(&mut *transaction)
                .await?;
                sqlx::query(
                    r#"
                    WITH ordered AS (
                        SELECT wishlist_item_id,
                            row_number() OVER (ORDER BY position) - 1 AS new_position
                        FROM wishlist_medley_items
                        WHERE medley_id = $1
                    )
                    UPDATE wishlist_medley_items AS membership
                    SET position = ordered.new_position::INTEGER
                    FROM ordered
                    WHERE membership.wishlist_item_id = ordered.wishlist_item_id
                    "#,
                )
                .bind(medley_id)
                .execute(&mut *transaction)
                .await?;
            }
        }
        transaction.commit().await?;
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn update_details(
        pool: &PgPool,
        item_id: Uuid,
        member_id: Uuid,
        is_admin: bool,
        title: &str,
        artist: Option<&str>,
        link: Option<&str>,
        link_title: Option<&str>,
    ) -> Result<Self, WishlistError> {
        Self::update_details_with_music(
            pool,
            item_id,
            member_id,
            is_admin,
            WishlistSongDetails {
                title,
                artist,
                link,
                link_title,
                tempo: None,
                musical_key: None,
            },
        )
        .await
    }

    pub async fn update_details_with_music(
        pool: &PgPool,
        item_id: Uuid,
        member_id: Uuid,
        is_admin: bool,
        details: WishlistSongDetails<'_>,
    ) -> Result<Self, WishlistError> {
        Self::update_details_with_optional_metadata(
            pool, item_id, member_id, is_admin, details, None,
        )
        .await
    }

    pub async fn update_details_with_song_metadata(
        pool: &PgPool,
        item_id: Uuid,
        member_id: Uuid,
        is_admin: bool,
        details: WishlistSongDetails<'_>,
        metadata: SongMetadataSelection,
    ) -> Result<Self, WishlistError> {
        let label_ids =
            validate_assignment(pool, metadata.category_id, &metadata.label_ids).await?;
        Self::update_details_with_optional_metadata(
            pool,
            item_id,
            member_id,
            is_admin,
            details,
            Some((metadata.category_id, label_ids)),
        )
        .await
    }

    async fn update_details_with_optional_metadata(
        pool: &PgPool,
        item_id: Uuid,
        member_id: Uuid,
        is_admin: bool,
        details: WishlistSongDetails<'_>,
        metadata: Option<(Option<Uuid>, Vec<Uuid>)>,
    ) -> Result<Self, WishlistError> {
        let title = validate_title(details.title)?;
        let artist = validate_artist(details.artist)?;
        let link = validate_link(details.link)?;
        let tempo = validate_tempo(details.tempo)?;
        let musical_key = validate_musical_key(details.musical_key)?;

        let (update_metadata, category_id, label_ids) = match metadata {
            Some((category_id, label_ids)) => (true, category_id, Some(label_ids)),
            None => (false, None, None),
        };
        let mut transaction = pool.begin().await?;
        let updated = sqlx::query_as::<_, Self>(
            r#"
            UPDATE wishlist_items
            SET title = $1, artist = $2, link = $3, link_title = $4, tempo = $5, musical_key = $6,
                song_category_id = CASE WHEN $7 THEN $8 ELSE song_category_id END
            WHERE id = $9 AND (proposed_by = $10 OR $11)
            RETURNING id, proposed_by, title, artist, link, link_title, tempo, musical_key, targets,
                category, proposed_by_name, created_at, song_category_id, NULL::TEXT AS song_category_name,
                NULL::UUID AS medley_category_id, NULL::TEXT AS medley_category_name, NULL::UUID AS medley_id,
                NULL::INTEGER AS medley_position, NULL::BIGINT AS medley_size,
                NULL::UUID AS medley_created_by, NULL::TEXT AS medley_name
            "#,
        )
        .bind(&title)
        .bind(&artist)
        .bind(&link)
        .bind(details.link_title)
        .bind(tempo)
        .bind(&musical_key)
        .bind(update_metadata)
        .bind(category_id)
        .bind(item_id)
        .bind(member_id)
        .bind(is_admin)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(map_wishlist_unique_violation)?;
        if let Some(updated) = updated {
            if let Some(label_ids) = label_ids {
                sqlx::query("DELETE FROM wishlist_item_labels WHERE wishlist_item_id = $1")
                    .bind(item_id)
                    .execute(&mut *transaction)
                    .await?;
                for label_id in label_ids {
                    sqlx::query(
                        "INSERT INTO wishlist_item_labels (wishlist_item_id, label_id) VALUES ($1, $2)",
                    )
                    .bind(item_id)
                    .bind(label_id)
                    .execute(&mut *transaction)
                    .await?;
                }
            }
            transaction.commit().await?;
            let metadata = Self::song_metadata_assignment(pool, item_id).await?;
            return Ok(Self {
                song_category: metadata.category,
                labels: metadata.labels,
                ..updated
            });
        }

        transaction.rollback().await?;
        let exists: bool =
            sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM wishlist_items WHERE id = $1)")
                .bind(item_id)
                .fetch_one(pool)
                .await?;
        Err(if exists {
            WishlistError::EditForbidden
        } else {
            WishlistError::NotFound
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn create(
        pool: &PgPool,
        title: &str,
        artist: Option<&str>,
        link: Option<&str>,
        link_title: Option<&str>,
        targets: &[String],
        category: WishlistCategory,
        member_id: Uuid,
        member_name: &str,
    ) -> Result<Self, WishlistError> {
        Self::create_with_music_details(
            pool,
            WishlistSongDetails {
                title,
                artist,
                link,
                link_title,
                tempo: None,
                musical_key: None,
            },
            targets,
            category,
            member_id,
            member_name,
        )
        .await
    }

    pub async fn create_with_music_details(
        pool: &PgPool,
        details: WishlistSongDetails<'_>,
        targets: &[String],
        category: WishlistCategory,
        member_id: Uuid,
        member_name: &str,
    ) -> Result<Self, WishlistError> {
        Self::create_with_song_metadata(
            pool,
            details,
            targets,
            category,
            member_id,
            member_name,
            None,
            &[],
        )
        .await
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn create_with_song_metadata(
        pool: &PgPool,
        details: WishlistSongDetails<'_>,
        targets: &[String],
        category: WishlistCategory,
        member_id: Uuid,
        member_name: &str,
        song_category_id: Option<Uuid>,
        label_ids: &[Uuid],
    ) -> Result<Self, WishlistError> {
        let title = validate_title(details.title)?;
        let artist = validate_artist(details.artist)?;
        let link = validate_link(details.link)?;
        let targets = validate_targets(targets)?;
        let tempo = validate_tempo(details.tempo)?;
        let musical_key = validate_musical_key(details.musical_key)?;
        let label_ids = validate_assignment(pool, song_category_id, label_ids).await?;

        let existing: bool = sqlx::query_scalar(
            r#"
            SELECT EXISTS (
                SELECT 1
                FROM wishlist_items
                WHERE lower(btrim(title)) = lower($1)
                  AND lower(btrim(coalesce(artist, ''))) = lower($2)
            )
            "#,
        )
        .bind(&title)
        .bind(artist.as_deref().unwrap_or_default())
        .fetch_one(pool)
        .await?;
        if existing {
            return Err(WishlistError::DuplicateSong);
        }

        let mut transaction = pool.begin().await?;
        let mut item = sqlx::query_as::<_, Self>(
            r#"
            INSERT INTO wishlist_items
                (id, title, artist, link, link_title, tempo, musical_key, targets, category, proposed_by, proposed_by_name, song_category_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING id, proposed_by, title, artist, link, link_title, tempo, musical_key, targets, category,
                proposed_by_name, created_at, song_category_id,
                (SELECT name FROM wishlist_categories WHERE id = wishlist_items.song_category_id) AS song_category_name,
                NULL::UUID AS medley_category_id, NULL::TEXT AS medley_category_name, NULL::UUID AS medley_id,
                NULL::INTEGER AS medley_position,
                NULL::BIGINT AS medley_size, NULL::UUID AS medley_created_by, NULL::TEXT AS medley_name
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(title)
        .bind(artist)
        .bind(link)
        .bind(details.link_title)
        .bind(tempo)
        .bind(&musical_key)
        .bind(targets)
        .bind(category.to_string())
        .bind(member_id)
        .bind(member_name)
        .bind(song_category_id)
        .fetch_one(&mut *transaction)
        .await
        .map_err(map_wishlist_unique_violation)?;

        for label_id in label_ids {
            sqlx::query(
                "INSERT INTO wishlist_item_labels (wishlist_item_id, label_id) VALUES ($1, $2)",
            )
            .bind(item.id)
            .bind(label_id)
            .execute(&mut *transaction)
            .await?;
        }
        item.labels = sqlx::query_as::<_, SongLabel>(
            r#"
            SELECT label.id, label.name, label.color
            FROM wishlist_item_labels AS assignment
            JOIN wishlist_labels AS label ON label.id = assignment.label_id
            WHERE assignment.wishlist_item_id = $1
            ORDER BY lower(label.name), label.name
            "#,
        )
        .bind(item.id)
        .fetch_all(&mut *transaction)
        .await?;
        transaction.commit().await?;

        Ok(item)
    }
}

#[cfg(feature = "ssr")]
fn map_wishlist_unique_violation(error: sqlx::Error) -> WishlistError {
    match &error {
        sqlx::Error::Database(database_error)
            if database_error.constraint() == Some("idx_wishlist_items_title_artist_unique") =>
        {
            WishlistError::DuplicateSong
        }
        _ => WishlistError::Database(error),
    }
}

#[cfg(feature = "ssr")]
fn map_medley_unique_violation(error: sqlx::Error) -> WishlistError {
    match &error {
        sqlx::Error::Database(database_error)
            if database_error.constraint()
                == Some("wishlist_medley_items_wishlist_item_id_key") =>
        {
            WishlistError::SongAlreadyInMedley
        }
        _ => WishlistError::Database(error),
    }
}

#[cfg(feature = "ssr")]
fn validate_title(title: &str) -> Result<String, WishlistError> {
    let title = title.trim();
    if title.is_empty() || title.chars().count() > 120 || title.chars().any(char::is_control) {
        return Err(WishlistError::InvalidTitle);
    }
    Ok(title.to_string())
}

#[cfg(feature = "ssr")]
fn validate_medley_name(name: &str) -> Result<String, WishlistError> {
    let name = name.trim();
    if name.is_empty() || name.chars().count() > 120 || name.chars().any(char::is_control) {
        return Err(WishlistError::InvalidMedleyName);
    }
    Ok(name.to_string())
}

#[cfg(feature = "ssr")]
fn validate_artist(artist: Option<&str>) -> Result<Option<String>, WishlistError> {
    let Some(artist) = artist.map(str::trim).filter(|artist| !artist.is_empty()) else {
        return Ok(None);
    };
    if artist.chars().count() > 120 || artist.chars().any(char::is_control) {
        return Err(WishlistError::InvalidArtist);
    }
    Ok(Some(artist.to_string()))
}

#[cfg(feature = "ssr")]
fn validate_link(link: Option<&str>) -> Result<Option<String>, WishlistError> {
    let Some(link) = link.map(str::trim).filter(|link| !link.is_empty()) else {
        return Ok(None);
    };
    let lower = link.to_ascii_lowercase();
    if link.len() > 2048
        || link.chars().any(char::is_whitespace)
        || link.chars().any(char::is_control)
        || !(lower.starts_with("https://") || lower.starts_with("http://"))
    {
        return Err(WishlistError::InvalidLink);
    }
    Ok(Some(link.to_string()))
}

#[cfg(feature = "ssr")]
fn validate_tempo(tempo: Option<i32>) -> Result<Option<i32>, WishlistError> {
    match tempo {
        Some(tempo @ 20..=300) => Ok(Some(tempo)),
        Some(_) => Err(WishlistError::InvalidTempo),
        None => Ok(None),
    }
}

#[cfg(feature = "ssr")]
fn validate_musical_key(key: Option<&str>) -> Result<Option<String>, WishlistError> {
    let Some(key) = key.map(str::trim).filter(|key| !key.is_empty()) else {
        return Ok(None);
    };
    if !WISHLIST_MUSICAL_KEYS.contains(&key) {
        return Err(WishlistError::InvalidMusicalKey);
    }
    Ok(Some(key.to_string()))
}

#[cfg(feature = "ssr")]
fn validate_targets(targets: &[String]) -> Result<Vec<String>, WishlistError> {
    let mut normalized = Vec::new();
    for target in targets {
        let target = target.trim();
        if target.is_empty() {
            continue;
        }
        if target.chars().count() > 40 || target.chars().any(char::is_control) {
            return Err(WishlistError::InvalidTargets);
        }
        if !normalized
            .iter()
            .any(|existing: &String| existing.eq_ignore_ascii_case(target))
        {
            normalized.push(target.to_string());
        }
    }
    if normalized.len() > 8 {
        return Err(WishlistError::InvalidTargets);
    }
    Ok(normalized)
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::*;
    use crate::models::Member;

    #[test]
    fn wishlist_category_round_trips() {
        for category in [
            WishlistCategory::Song,
            WishlistCategory::Medley,
            WishlistCategory::Try,
            WishlistCategory::Unplayable,
            WishlistCategory::OtherOrdering,
            WishlistCategory::NotFitMedley,
        ] {
            assert_eq!(
                category.to_string().parse::<WishlistCategory>().unwrap(),
                category
            );
        }
        assert!("unknown".parse::<WishlistCategory>().is_err());
    }

    #[test]
    fn validates_and_normalizes_wishlist_fields() {
        assert_eq!(validate_title("  Song name ").unwrap(), "Song name");
        assert_eq!(
            validate_artist(Some("  Artist ")).unwrap(),
            Some("Artist".into())
        );
        assert_eq!(
            validate_link(Some(" https://example.com/song ")).unwrap(),
            Some("https://example.com/song".into())
        );
        assert_eq!(validate_tempo(Some(120)).unwrap(), Some(120));
        assert_eq!(
            validate_musical_key(Some(" G major ")).unwrap(),
            Some("G major".into())
        );
        assert_eq!(
            validate_targets(&["Party".into(), "party".into(), "Slow dance".into()]).unwrap(),
            vec!["Party", "Slow dance"]
        );
    }

    #[test]
    fn rejects_invalid_wishlist_fields() {
        assert!(matches!(
            validate_title("  "),
            Err(WishlistError::InvalidTitle)
        ));
        assert!(matches!(
            validate_title(&"a".repeat(121)),
            Err(WishlistError::InvalidTitle)
        ));
        assert!(matches!(
            validate_artist(Some("a\nb")),
            Err(WishlistError::InvalidArtist)
        ));
        assert!(matches!(
            validate_link(Some("javascript:alert(1)")),
            Err(WishlistError::InvalidLink)
        ));
        assert!(matches!(
            validate_link(Some("https://example.com/a b")),
            Err(WishlistError::InvalidLink)
        ));
        assert!(matches!(
            validate_tempo(Some(301)),
            Err(WishlistError::InvalidTempo)
        ));
        assert!(matches!(
            validate_musical_key(Some("H major")),
            Err(WishlistError::InvalidMusicalKey)
        ));
        assert!(matches!(
            validate_targets(&["x".repeat(41)]),
            Err(WishlistError::InvalidTargets)
        ));
        assert!(matches!(
            validate_targets(&(0..9).map(|n| format!("target-{n}")).collect::<Vec<_>>()),
            Err(WishlistError::InvalidTargets)
        ));
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn wishlist_item_persists_targets_and_optional_details(pool: PgPool) {
        let member = Member::find_active_by_name(&pool, "Admin")
            .await
            .expect("seeded member lookup should succeed")
            .expect("migration should seed an admin");
        let item = WishlistItem::create_with_music_details(
            &pool,
            WishlistSongDetails {
                title: "  Song  ",
                artist: Some("  Artist "),
                link: Some("https://example.com/song"),
                link_title: Some("Example song"),
                tempo: Some(124),
                musical_key: Some("G major"),
            },
            &["party".into(), "slow-dance".into()],
            WishlistCategory::Medley,
            member.id,
            &member.name,
        )
        .await
        .expect("wishlist item should be saved");

        assert_eq!(item.title, "Song");
        assert_eq!(item.artist.as_deref(), Some("Artist"));
        assert_eq!(item.link_title.as_deref(), Some("Example song"));
        assert_eq!(item.tempo, Some(124));
        assert_eq!(item.musical_key.as_deref(), Some("G major"));
        assert_eq!(item.targets, ["party", "slow-dance"]);
        assert_eq!(item.category, WishlistCategory::Medley);
        assert!(
            WishlistItem::list(&pool)
                .await
                .expect("wishlist should be listed")
                .iter()
                .any(|listed| listed.id == item.id)
        );
        let listed = WishlistItem::list(&pool)
            .await
            .expect("wishlist should be listed")
            .into_iter()
            .find(|listed| listed.id == item.id)
            .expect("created song should be listed");
        assert_eq!(listed.tempo, Some(124));
        assert_eq!(listed.musical_key.as_deref(), Some("G major"));
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn song_and_medley_metadata_is_persisted(pool: PgPool) {
        let member = Member::find_active_by_name(&pool, "Admin")
            .await
            .expect("seeded member lookup should succeed")
            .expect("migration should seed an admin");
        let song = WishlistItem::create(
            &pool,
            "Metadata song",
            None,
            None,
            None,
            &[],
            WishlistCategory::Song,
            member.id,
            &member.name,
        )
        .await
        .expect("song should be saved");
        let second_song = WishlistItem::create(
            &pool,
            "Metadata song two",
            None,
            None,
            None,
            &[],
            WishlistCategory::Song,
            member.id,
            &member.name,
        )
        .await
        .expect("second song should be saved");
        let category = SongCategory::create(&pool, "Party")
            .await
            .expect("category should be created");
        let song_label = SongLabel::create(&pool, "2 guitars", "#2563EB")
            .await
            .expect("label should be created");
        let medley_label = SongLabel::create(&pool, "Short", "#15803D")
            .await
            .expect("label should be created");

        let updated_song = WishlistItem::update_details_with_song_metadata(
            &pool,
            song.id,
            member.id,
            true,
            WishlistSongDetails {
                title: "Updated metadata song",
                artist: None,
                link: None,
                link_title: None,
                tempo: None,
                musical_key: None,
            },
            SongMetadataSelection {
                category_id: Some(category.id),
                label_ids: vec![song_label.id, song_label.id],
            },
        )
        .await
        .expect("song details and metadata should be updated together");
        assert_eq!(updated_song.title, "Updated metadata song");
        assert_eq!(updated_song.song_category, Some(category.clone()));
        assert_eq!(updated_song.labels, [song_label.clone()]);

        let medley_id = WishlistItem::create_medley(
            &pool,
            &[song.id, second_song.id],
            "Metadata medley",
            member.id,
        )
        .await
        .expect("medley should be created");
        let medley_assignment = WishlistItem::update_medley_with_metadata(
            &pool,
            medley_id,
            &[song.id, second_song.id],
            "Updated metadata medley",
            member.id,
            true,
            SongMetadataSelection {
                category_id: Some(category.clone().id),
                label_ids: vec![medley_label.id],
            },
        )
        .await
        .expect("medley details and metadata should be updated together");
        assert_eq!(medley_assignment.category, Some(category.clone()));
        assert_eq!(medley_assignment.labels, [medley_label.clone()]);

        let listed = WishlistItem::list(&pool)
            .await
            .expect("wishlist should load");
        let listed_song = listed.iter().find(|item| item.id == song.id).unwrap();
        assert_eq!(listed_song.song_category.as_ref().unwrap().name, "Party");
        assert_eq!(listed_song.labels, [song_label.clone()]);
        assert_eq!(listed_song.medley_category.as_ref().unwrap().name, "Party");
        assert_eq!(listed_song.medley_labels, [medley_label.clone()]);

        SongLabel::delete(&pool, song_label.id)
            .await
            .expect("song label should be removable");
        SongLabel::delete(&pool, medley_label.id)
            .await
            .expect("medley label should be removable");
        SongCategory::delete(&pool, category.id)
            .await
            .expect("category should be removable");
        let listed = WishlistItem::list(&pool)
            .await
            .expect("wishlist should load after metadata removal");
        let listed_song = listed.iter().find(|item| item.id == song.id).unwrap();
        assert!(listed_song.song_category.is_none());
        assert!(listed_song.labels.is_empty());
        assert!(listed_song.medley_category.is_none());
        assert!(listed_song.medley_labels.is_empty());
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn suggested_song_is_created_with_category_and_labels(pool: PgPool) {
        let member = Member::find_active_by_name(&pool, "Admin")
            .await
            .unwrap()
            .unwrap();
        let category = SongCategory::create(&pool, "Dancing").await.unwrap();
        let label = SongLabel::create(&pool, "Two guitars", "#2563EB")
            .await
            .unwrap();

        let item = WishlistItem::create_with_song_metadata(
            &pool,
            WishlistSongDetails {
                title: "Song with metadata",
                artist: None,
                link: None,
                link_title: None,
                tempo: None,
                musical_key: None,
            },
            &[],
            WishlistCategory::Song,
            member.id,
            &member.name,
            Some(category.id),
            &[label.id],
        )
        .await
        .unwrap();

        assert_eq!(item.song_category, Some(category));
        assert_eq!(item.labels, [label]);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn invalid_metadata_does_not_create_suggested_song(pool: PgPool) {
        let member = Member::find_active_by_name(&pool, "Admin")
            .await
            .unwrap()
            .unwrap();
        let result = WishlistItem::create_with_song_metadata(
            &pool,
            WishlistSongDetails {
                title: "Invalid metadata song",
                artist: None,
                link: None,
                link_title: None,
                tempo: None,
                musical_key: None,
            },
            &[],
            WishlistCategory::Song,
            member.id,
            &member.name,
            Some(Uuid::new_v4()),
            &[],
        )
        .await;

        assert!(matches!(
            result,
            Err(WishlistError::Metadata(
                SongMetadataError::InvalidAssignment
            ))
        ));
        assert!(
            !WishlistItem::list(&pool)
                .await
                .unwrap()
                .iter()
                .any(|item| item.title == "Invalid metadata song")
        );
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn wishlist_items_are_listed_alphabetically(pool: PgPool) {
        let member = Member::find_active_by_name(&pool, "Admin")
            .await
            .expect("seeded member lookup should succeed")
            .expect("migration should seed an admin");
        let mut created_ids = Vec::new();
        for title in ["Zulu", "alpha", "Bravo"] {
            let item = WishlistItem::create(
                &pool,
                title,
                None,
                None,
                None,
                &[],
                WishlistCategory::Song,
                member.id,
                &member.name,
            )
            .await
            .expect("wishlist item should be saved");
            created_ids.push(item.id);
        }

        let listed: Vec<_> = WishlistItem::list(&pool)
            .await
            .expect("wishlist should be listed")
            .into_iter()
            .filter(|item| created_ids.contains(&item.id))
            .map(|item| item.title)
            .collect();
        assert_eq!(listed, ["alpha", "Bravo", "Zulu"]);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn wishlist_rejects_duplicate_title_and_artist_pairs(pool: PgPool) {
        let member = Member::find_active_by_name(&pool, "Admin")
            .await
            .expect("seeded member lookup should succeed")
            .expect("migration should seed an admin");
        WishlistItem::create(
            &pool,
            "Song title",
            Some("Artist"),
            None,
            None,
            &[],
            WishlistCategory::Song,
            member.id,
            &member.name,
        )
        .await
        .expect("first wishlist item should be saved");

        let database_error = sqlx::query(
            r#"
            INSERT INTO wishlist_items
                (id, title, artist, category, proposed_by, proposed_by_name)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(" song title ")
        .bind("artist")
        .bind(WishlistCategory::Song.to_string())
        .bind(member.id)
        .bind(&member.name)
        .execute(&pool)
        .await
        .expect_err("database index should prevent duplicate song and artist");
        assert!(matches!(
            database_error,
            sqlx::Error::Database(ref error)
                if error.constraint() == Some("idx_wishlist_items_title_artist_unique")
        ));

        assert!(matches!(
            WishlistItem::create(
                &pool,
                "  SONG TITLE ",
                Some(" artist "),
                None,
                None,
                &[],
                WishlistCategory::Try,
                member.id,
                &member.name,
            )
            .await,
            Err(WishlistError::DuplicateSong)
        ));

        WishlistItem::create(
            &pool,
            "Song title",
            Some("Different Artist"),
            None,
            None,
            &[],
            WishlistCategory::Song,
            member.id,
            &member.name,
        )
        .await
        .expect("same title by a different artist should be allowed");

        WishlistItem::create(
            &pool,
            "Instrumental",
            None,
            None,
            None,
            &[],
            WishlistCategory::Song,
            member.id,
            &member.name,
        )
        .await
        .expect("song without artist should be allowed");
        assert!(matches!(
            WishlistItem::create(
                &pool,
                "Instrumental",
                Some(""),
                None,
                None,
                &[],
                WishlistCategory::Song,
                member.id,
                &member.name,
            )
            .await,
            Err(WishlistError::DuplicateSong)
        ));
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn member_feedback_is_independent_and_can_be_changed_or_removed(pool: PgPool) {
        let admin = Member::find_active_by_name(&pool, "Admin")
            .await
            .expect("seeded member lookup should succeed")
            .expect("migration should seed an admin");
        let second_member = Member::create(
            &pool,
            &format!("Feedback member {}", Uuid::new_v4()),
            crate::models::MemberRole::Member,
        )
        .await
        .expect("second member should be created");
        let item = WishlistItem::create(
            &pool,
            "Feedback song",
            None,
            None,
            None,
            &[],
            WishlistCategory::Song,
            admin.id,
            &admin.name,
        )
        .await
        .expect("wishlist item should be created");

        WishlistItem::set_feedback(&pool, item.id, admin.id, Some(WishlistCategory::Try))
            .await
            .expect("admin feedback should be saved");
        WishlistItem::set_feedback(
            &pool,
            item.id,
            second_member.id,
            Some(WishlistCategory::Medley),
        )
        .await
        .expect("second member feedback should be saved");
        WishlistItem::set_feedback(&pool, item.id, admin.id, Some(WishlistCategory::Unplayable))
            .await
            .expect("admin feedback should be updated");

        let listed = WishlistItem::list(&pool)
            .await
            .expect("wishlist should be listed")
            .into_iter()
            .find(|listed| listed.id == item.id)
            .expect("wishlist item should be present");
        assert_eq!(listed.feedback.len(), 2);
        assert_eq!(
            listed
                .feedback
                .iter()
                .find(|feedback| feedback.member_id == admin.id)
                .map(|feedback| feedback.category),
            Some(WishlistCategory::Unplayable)
        );
        assert_eq!(
            listed
                .feedback
                .iter()
                .find(|feedback| feedback.member_id == second_member.id)
                .map(|feedback| feedback.category),
            Some(WishlistCategory::Medley)
        );

        WishlistItem::set_feedback(&pool, item.id, admin.id, None)
            .await
            .expect("admin feedback should be removed");
        let remaining = WishlistItem::list(&pool)
            .await
            .expect("wishlist should be listed")
            .into_iter()
            .find(|listed| listed.id == item.id)
            .expect("wishlist item should remain");
        assert_eq!(remaining.feedback.len(), 1);
        assert_eq!(remaining.feedback[0].member_id, second_member.id);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn proposer_can_delete_wishlist_item_and_others_cannot(pool: PgPool) {
        let admin = Member::find_active_by_name(&pool, "Admin")
            .await
            .expect("seeded member lookup should succeed")
            .expect("migration should seed an admin");
        let proposer = Member::create(
            &pool,
            &format!("Wishlist proposer {}", Uuid::new_v4()),
            crate::models::MemberRole::Member,
        )
        .await
        .expect("proposer should be created");
        let item = WishlistItem::create(
            &pool,
            "Delete me",
            Some("Artist"),
            None,
            None,
            &[],
            WishlistCategory::Song,
            proposer.id,
            &proposer.name,
        )
        .await
        .expect("wishlist item should be created");
        WishlistItem::set_feedback(&pool, item.id, admin.id, Some(WishlistCategory::Try))
            .await
            .expect("feedback should be saved");
        assert!(matches!(
            WishlistItem::delete(&pool, item.id, admin.id, false).await,
            Err(WishlistError::DeleteForbidden)
        ));
        assert!(
            WishlistItem::delete(&pool, item.id, proposer.id, false)
                .await
                .is_ok()
        );
        assert!(
            WishlistItem::list(&pool)
                .await
                .expect("wishlist should be listed")
                .iter()
                .all(|listed| listed.id != item.id)
        );
        let feedback_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM wishlist_feedback WHERE wishlist_item_id = $1",
        )
        .bind(item.id)
        .fetch_one(&pool)
        .await
        .expect("feedback count query should succeed");
        assert_eq!(feedback_count, 0);

        let admin_item = WishlistItem::create(
            &pool,
            "Admin delete",
            None,
            None,
            None,
            &[],
            WishlistCategory::Song,
            proposer.id,
            &proposer.name,
        )
        .await
        .expect("another wishlist item should be created");
        WishlistItem::delete(&pool, admin_item.id, admin.id, true)
            .await
            .expect("admins should be able to delete any wishlist item");
        assert!(matches!(
            WishlistItem::delete(&pool, admin_item.id, admin.id, true).await,
            Err(WishlistError::NotFound)
        ));
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn proposer_and_admin_can_edit_wishlist_items_with_duplicate_protection(pool: PgPool) {
        let admin = Member::find_active_by_name(&pool, "Admin")
            .await
            .expect("seeded member lookup should succeed")
            .expect("migration should seed an admin");
        let proposer = Member::create(
            &pool,
            &format!("Edit proposer {}", Uuid::new_v4()),
            crate::models::MemberRole::Member,
        )
        .await
        .expect("proposer should be created");
        let other_member = Member::create(
            &pool,
            &format!("Other member {}", Uuid::new_v4()),
            crate::models::MemberRole::Member,
        )
        .await
        .expect("other member should be created");
        let item = WishlistItem::create(
            &pool,
            "Original title",
            Some("Original artist"),
            None,
            None,
            &[],
            WishlistCategory::Song,
            proposer.id,
            &proposer.name,
        )
        .await
        .expect("wishlist item should be created");
        let duplicate = WishlistItem::create(
            &pool,
            "Existing title",
            Some("Existing artist"),
            None,
            None,
            &[],
            WishlistCategory::Song,
            proposer.id,
            &proposer.name,
        )
        .await
        .expect("second wishlist item should be created");

        assert!(matches!(
            WishlistItem::update_details(
                &pool,
                item.id,
                other_member.id,
                false,
                "Unauthorized",
                None,
                None,
                None,
            )
            .await,
            Err(WishlistError::EditForbidden)
        ));
        assert!(matches!(
            WishlistItem::update_details(
                &pool,
                item.id,
                proposer.id,
                false,
                "Existing title",
                Some("Existing artist"),
                None,
                None,
            )
            .await,
            Err(WishlistError::DuplicateSong)
        ));
        let updated = WishlistItem::update_details_with_music(
            &pool,
            item.id,
            proposer.id,
            false,
            WishlistSongDetails {
                title: "Renamed title",
                artist: Some("Updated artist"),
                link: Some("https://example.com/updated"),
                link_title: Some("Updated link title"),
                tempo: Some(98),
                musical_key: Some("A minor"),
            },
        )
        .await
        .expect("proposer should be able to edit their wishlist item");
        assert_eq!(updated.title, "Renamed title");
        assert_eq!(updated.artist.as_deref(), Some("Updated artist"));
        assert_eq!(updated.link.as_deref(), Some("https://example.com/updated"));
        assert_eq!(updated.link_title.as_deref(), Some("Updated link title"));
        assert_eq!(updated.tempo, Some(98));
        assert_eq!(updated.musical_key.as_deref(), Some("A minor"));

        let admin_updated = WishlistItem::update_details(
            &pool,
            duplicate.id,
            admin.id,
            true,
            "Admin edited title",
            Some("Admin edited artist"),
            None,
            None,
        )
        .await
        .expect("admin should be able to edit any wishlist item");
        assert_eq!(admin_updated.title, "Admin edited title");
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn members_can_create_ordered_medleys_and_only_creator_or_admin_can_disband(
        pool: PgPool,
    ) {
        let admin = Member::find_active_by_name(&pool, "Admin")
            .await
            .expect("seeded member lookup should succeed")
            .expect("migration should seed an admin");
        let member = Member::create(
            &pool,
            &format!("Medley member {}", Uuid::new_v4()),
            crate::models::MemberRole::Member,
        )
        .await
        .expect("member should be created");
        let mut song_ids = Vec::new();
        for title in ["First song", "Second song", "Third song"] {
            song_ids.push(
                WishlistItem::create(
                    &pool,
                    title,
                    None,
                    None,
                    None,
                    &[],
                    WishlistCategory::Song,
                    member.id,
                    &member.name,
                )
                .await
                .expect("song should be created")
                .id,
            );
        }

        assert!(matches!(
            WishlistItem::create_medley(&pool, &[song_ids[0]], "Test medley", member.id).await,
            Err(WishlistError::InvalidMedleySelection)
        ));
        let selected_ids = [song_ids[1], song_ids[0], song_ids[2]];
        let medley_id = WishlistItem::create_medley(&pool, &selected_ids, "Test medley", member.id)
            .await
            .expect("selected songs should be grouped in order");
        let grouped: Vec<_> = WishlistItem::list(&pool)
            .await
            .expect("wishlist should be listed")
            .into_iter()
            .filter(|item| item.medley_id == Some(medley_id))
            .collect();
        assert_eq!(grouped.len(), 3);
        assert_eq!(grouped[0].id, song_ids[1]);
        assert_eq!(grouped[0].medley_position, Some(0));
        assert_eq!(grouped[0].medley_size, Some(3));
        assert_eq!(grouped[1].id, song_ids[0]);
        assert_eq!(grouped[1].medley_position, Some(1));
        assert_eq!(grouped[2].id, song_ids[2]);
        assert_eq!(grouped[2].medley_position, Some(2));
        assert!(matches!(
            WishlistItem::create_medley(
                &pool,
                &[song_ids[0], song_ids[2]],
                "Test medley",
                member.id
            )
            .await,
            Err(WishlistError::SongAlreadyInMedley)
        ));
        assert!(matches!(
            WishlistItem::disband_medley(&pool, medley_id, admin.id, false).await,
            Err(WishlistError::MedleyForbidden)
        ));
        WishlistItem::disband_medley(&pool, medley_id, member.id, false)
            .await
            .expect("creator should be able to disband their medley");
        assert!(
            WishlistItem::list(&pool)
                .await
                .expect("wishlist should be listed")
                .iter()
                .filter(|item| song_ids.contains(&item.id))
                .all(|item| item.medley_id.is_none())
        );

        let medley_id = WishlistItem::create_medley(&pool, &selected_ids, "Test medley", member.id)
            .await
            .expect("songs should be groupable again");
        WishlistItem::delete(&pool, song_ids[1], member.id, false)
            .await
            .expect("song proposer should be able to delete a medley part");
        let remaining: Vec<_> = WishlistItem::list(&pool)
            .await
            .expect("wishlist should be listed")
            .into_iter()
            .filter(|item| item.medley_id == Some(medley_id))
            .collect();
        assert_eq!(remaining.len(), 2);
        assert_eq!(remaining[0].medley_position, Some(0));
        assert_eq!(remaining[1].medley_position, Some(1));
        WishlistItem::delete(&pool, remaining[0].id, member.id, false)
            .await
            .expect("song proposer should be able to delete the next medley part");
        let last_item = WishlistItem::list(&pool)
            .await
            .expect("wishlist should be listed")
            .into_iter()
            .find(|item| item.id == remaining[1].id)
            .expect("last song should remain");
        assert!(last_item.medley_id.is_none());
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn medleys_can_be_renamed_extended_and_have_songs_removed(pool: PgPool) {
        let admin = Member::find_active_by_name(&pool, "Admin")
            .await
            .expect("seeded member lookup should succeed")
            .expect("migration should seed an admin");
        let member = Member::create(
            &pool,
            &format!("Medley owner {}", Uuid::new_v4()),
            crate::models::MemberRole::Member,
        )
        .await
        .expect("member should be created");
        let other_member = Member::create(
            &pool,
            &format!("Other member {}", Uuid::new_v4()),
            crate::models::MemberRole::Member,
        )
        .await
        .expect("member should be created");
        let mut song_ids = Vec::new();
        for title in ["Medley first", "Medley second", "Medley third"] {
            song_ids.push(
                WishlistItem::create(
                    &pool,
                    &format!("{title} {}", Uuid::new_v4()),
                    None,
                    None,
                    None,
                    &[],
                    WishlistCategory::Song,
                    member.id,
                    &member.name,
                )
                .await
                .expect("song should be created")
                .id,
            );
        }

        let medley_id =
            WishlistItem::create_medley(&pool, &song_ids[..2], "Opening set", member.id)
                .await
                .expect("songs should be grouped");
        assert!(matches!(
            WishlistItem::set_feedback(
                &pool,
                song_ids[2],
                member.id,
                Some(WishlistCategory::NotFitMedley)
            )
            .await,
            Err(WishlistError::MedleyFeedbackRequiresMedley)
        ));
        WishlistItem::set_feedback(
            &pool,
            song_ids[0],
            member.id,
            Some(WishlistCategory::OtherOrdering),
        )
        .await
        .expect("medley-specific feedback should be accepted for a medley song");
        assert!(matches!(
            WishlistItem::update_medley_name(
                &pool,
                medley_id,
                "Other name",
                other_member.id,
                false
            )
            .await,
            Err(WishlistError::MedleyForbidden)
        ));
        assert!(matches!(
            WishlistItem::add_medley_items(
                &pool,
                medley_id,
                &[song_ids[2]],
                other_member.id,
                false
            )
            .await,
            Err(WishlistError::MedleyForbidden)
        ));
        assert!(matches!(
            WishlistItem::remove_medley_item(&pool, medley_id, song_ids[0], other_member.id, false)
                .await,
            Err(WishlistError::MedleyForbidden)
        ));
        assert!(matches!(
            WishlistItem::move_medley_item(
                &pool,
                medley_id,
                song_ids[0],
                1,
                other_member.id,
                false
            )
            .await,
            Err(WishlistError::MedleyForbidden)
        ));

        WishlistItem::update_medley_name(&pool, medley_id, "Opening set", member.id, false)
            .await
            .expect("creator should be able to rename the medley");
        assert!(matches!(
            WishlistItem::update_medley_name(&pool, medley_id, "  ", member.id, false).await,
            Err(WishlistError::InvalidMedleyName)
        ));
        WishlistItem::add_medley_items(&pool, medley_id, &[song_ids[2]], member.id, false)
            .await
            .expect("creator should be able to add songs");
        let reordered =
            WishlistItem::move_medley_item(&pool, medley_id, song_ids[2], -1, member.id, false)
                .await
                .expect("creator should be able to reorder songs");
        assert_eq!(reordered, vec![song_ids[0], song_ids[2], song_ids[1]]);
        let unchanged =
            WishlistItem::move_medley_item(&pool, medley_id, song_ids[0], -1, admin.id, true)
                .await
                .expect("admin can request a move at the start of the list");
        assert_eq!(unchanged, reordered);
        let grouped: Vec<_> = WishlistItem::list(&pool)
            .await
            .expect("wishlist should be listed")
            .into_iter()
            .filter(|item| item.medley_id == Some(medley_id))
            .collect();
        assert_eq!(grouped.len(), 3);
        assert!(
            grouped
                .iter()
                .all(|item| item.medley_name.as_deref() == Some("Opening set"))
        );
        assert_eq!(
            grouped.iter().map(|item| item.id).collect::<Vec<_>>(),
            vec![song_ids[0], song_ids[2], song_ids[1]]
        );

        WishlistItem::remove_medley_item(&pool, medley_id, song_ids[1], member.id, false)
            .await
            .expect("creator should be able to remove a song");
        let grouped: Vec<_> = WishlistItem::list(&pool)
            .await
            .expect("wishlist should be listed")
            .into_iter()
            .filter(|item| item.medley_id == Some(medley_id))
            .collect();
        assert_eq!(grouped.len(), 2);
        assert_eq!(grouped[0].medley_position, Some(0));
        assert_eq!(grouped[1].medley_position, Some(1));

        WishlistItem::remove_medley_item(&pool, medley_id, song_ids[0], admin.id, true)
            .await
            .expect("admin should be able to remove a song");
        let remaining: Vec<_> = WishlistItem::list(&pool)
            .await
            .expect("wishlist should be listed")
            .into_iter()
            .filter(|item| song_ids.contains(&item.id))
            .collect();
        assert!(remaining.iter().all(|item| item.medley_id.is_none()));
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn medley_edit_atomically_updates_name_membership_and_order(pool: PgPool) {
        let member = Member::create(
            &pool,
            &format!("Medley editor {}", Uuid::new_v4()),
            crate::models::MemberRole::Member,
        )
        .await
        .expect("member should be created");
        let mut song_ids = Vec::new();
        for title in [
            "Edit medley one",
            "Edit medley two",
            "Edit medley three",
            "Edit medley four",
        ] {
            song_ids.push(
                WishlistItem::create(
                    &pool,
                    &format!("{title} {}", Uuid::new_v4()),
                    None,
                    None,
                    None,
                    &[],
                    WishlistCategory::Song,
                    member.id,
                    &member.name,
                )
                .await
                .expect("song should be created")
                .id,
            );
        }
        let medley_id = WishlistItem::create_medley(
            &pool,
            &[song_ids[0], song_ids[1]],
            "Before edit",
            member.id,
        )
        .await
        .expect("medley should be created");

        assert!(matches!(
            WishlistItem::update_medley(
                &pool,
                medley_id,
                &[song_ids[2]],
                "Invalid edit",
                member.id,
                false,
            )
            .await,
            Err(WishlistError::InvalidMedleySelection)
        ));
        WishlistItem::update_medley(
            &pool,
            medley_id,
            &[song_ids[2], song_ids[0], song_ids[3]],
            "After edit",
            member.id,
            false,
        )
        .await
        .expect("medley edit should apply");

        let listed = WishlistItem::list(&pool)
            .await
            .expect("wishlist should be listed");
        let grouped: Vec<_> = listed
            .iter()
            .filter(|item| item.medley_id == Some(medley_id))
            .collect();
        assert_eq!(
            grouped.iter().map(|item| item.id).collect::<Vec<_>>(),
            vec![song_ids[2], song_ids[0], song_ids[3]]
        );
        assert!(
            grouped
                .iter()
                .all(|item| item.medley_name.as_deref() == Some("After edit"))
        );
        let removed_song = listed
            .iter()
            .find(|item| item.id == song_ids[1])
            .expect("removed song should remain in the wishlist");
        assert_eq!(removed_song.medley_id, None);

        assert!(matches!(
            WishlistItem::update_medley(
                &pool,
                medley_id,
                &[song_ids[0], song_ids[1]],
                "Unauthorized",
                Uuid::new_v4(),
                false,
            )
            .await,
            Err(WishlistError::MedleyForbidden)
        ));
    }
}
