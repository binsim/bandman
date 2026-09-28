use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
#[cfg(feature = "ssr")]
use sqlx::{postgres::PgRow, FromRow, PgPool, Row};
use uuid::Uuid;

/// Band member role used for authorization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemberRole {
    Admin,
    Member,
    Participant,
}

impl MemberRole {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Admin => "admin",
            Self::Member => "member",
            Self::Participant => "participant",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "admin" => Some(Self::Admin),
            "member" => Some(Self::Member),
            "participant" => Some(Self::Participant),
            _ => None,
        }
    }

    pub fn is_admin(self) -> bool {
        matches!(self, Self::Admin)
    }

    pub fn is_participant(self) -> bool {
        matches!(self, Self::Participant)
    }
}

impl TryFrom<String> for MemberRole {
    type Error = std::io::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::parse(&value).ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("unknown member role: {value}"),
            )
        })
    }
}

impl std::fmt::Display for MemberRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A band member who can log in by name.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Member {
    pub id: Uuid,
    pub name: String,
    pub role: MemberRole,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
}

#[cfg(feature = "ssr")]
#[derive(Debug, thiserror::Error)]
pub enum MemberError {
    #[error("Name must be 1-80 characters and contain no control characters")]
    InvalidName,
    #[error("A member with this name already exists")]
    DuplicateName,
    #[error("Member not found")]
    NotFound,
    #[error("At least one active admin must remain")]
    LastActiveAdmin,
    #[error(transparent)]
    Database(#[from] sqlx::Error),
}

#[cfg(feature = "ssr")]
impl<'r> FromRow<'r, PgRow> for Member {
    fn from_row(row: &'r PgRow) -> Result<Self, sqlx::Error> {
        let role: String = row.try_get("role")?;
        let role = MemberRole::try_from(role).map_err(|error| sqlx::Error::ColumnDecode {
            index: "role".to_string(),
            source: Box::new(error),
        })?;

        Ok(Self {
            id: row.try_get("id")?,
            name: row.try_get("name")?,
            role,
            active: row.try_get("active")?,
            created_at: row.try_get("created_at")?,
            last_login: row.try_get("last_login")?,
        })
    }
}

#[cfg(feature = "ssr")]
impl Member {
    const ADMIN_MUTATION_LOCK: i64 = 6_204_119_021;

    pub async fn list_active(pool: &PgPool) -> Result<Vec<Self>, MemberError> {
        Ok(sqlx::query_as::<_, Member>(
            r#"
            SELECT id, name, role, active, created_at, last_login
            FROM members
            WHERE active = TRUE
            ORDER BY name ASC
            "#,
        )
        .fetch_all(pool)
        .await?)
    }

    pub async fn list_all(pool: &PgPool) -> Result<Vec<Self>, MemberError> {
        Ok(sqlx::query_as::<_, Member>(
            r#"
            SELECT id, name, role, active, created_at, last_login
            FROM members
            ORDER BY lower(name) ASC, name ASC
            "#,
        )
        .fetch_all(pool)
        .await?)
    }

    pub async fn find_active_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Self>, MemberError> {
        Ok(sqlx::query_as::<_, Member>(
            r#"
            SELECT id, name, role, active, created_at, last_login
            FROM members
            WHERE id = $1 AND active = TRUE
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?)
    }

    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Self>, MemberError> {
        Ok(sqlx::query_as::<_, Member>(
            r#"
            SELECT id, name, role, active, created_at, last_login
            FROM members
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?)
    }

    pub async fn find_active_by_name(
        pool: &PgPool,
        name: &str,
    ) -> Result<Option<Self>, MemberError> {
        sqlx::query_as::<_, Member>(
            r#"
            SELECT id, name, role, active, created_at, last_login
            FROM members
            WHERE lower(name) = lower($1) AND active = TRUE
            "#,
        )
        .bind(name)
        .fetch_optional(pool)
        .await
        .map_err(MemberError::from)
    }

    pub async fn create(pool: &PgPool, name: &str, role: MemberRole) -> Result<Self, MemberError> {
        let name = validate_member_name(name)?;
        let mut transaction = Self::begin_member_mutation(pool).await?;

        let member = sqlx::query_as::<_, Member>(
            r#"
            INSERT INTO members (id, name, role)
            SELECT $1, $2, $3
            WHERE NOT EXISTS (
                SELECT 1 FROM members WHERE lower(name) = lower($2)
            )
            RETURNING id, name, role, active, created_at, last_login
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(name)
        .bind(role.as_str())
        .fetch_optional(&mut *transaction)
        .await?;
        let Some(member) = member else {
            return Err(MemberError::DuplicateName);
        };

        transaction.commit().await?;
        Ok(member)
    }

    pub async fn update_name(&mut self, pool: &PgPool, name: &str) -> Result<(), MemberError> {
        let name = validate_member_name(name)?;
        let mut transaction = Self::begin_member_mutation(pool).await?;

        let member = sqlx::query_as::<_, Member>(
            r#"
            UPDATE members
            SET name = $1
            WHERE id = $2
                AND NOT EXISTS (
                    SELECT 1
                    FROM members
                    WHERE lower(name) = lower($1) AND id <> $2
                )
            RETURNING id, name, role, active, created_at, last_login
            "#,
        )
        .bind(name)
        .bind(self.id)
        .fetch_optional(&mut *transaction)
        .await?;
        let Some(member) = member else {
            return Err(if Self::member_exists(&mut transaction, self.id).await? {
                MemberError::DuplicateName
            } else {
                MemberError::NotFound
            });
        };

        transaction.commit().await?;
        *self = member;
        Ok(())
    }

    pub async fn update_role(
        &mut self,
        pool: &PgPool,
        role: MemberRole,
    ) -> Result<(), MemberError> {
        let mut transaction = Self::begin_member_mutation(pool).await?;

        let member = sqlx::query_as::<_, Member>(
            r#"
            UPDATE members
            SET role = $1
            WHERE id = $2
                AND (
                    role <> 'admin'
                    OR NOT active
                    OR $1 = 'admin'
                    OR (SELECT COUNT(*) FROM members WHERE active AND role = 'admin') > 1
                )
            RETURNING id, name, role, active, created_at, last_login
            "#,
        )
        .bind(role.as_str())
        .bind(self.id)
        .fetch_optional(&mut *transaction)
        .await?;
        let Some(member) = member else {
            return Err(if Self::member_exists(&mut transaction, self.id).await? {
                MemberError::LastActiveAdmin
            } else {
                MemberError::NotFound
            });
        };

        transaction.commit().await?;
        *self = member;
        Ok(())
    }

    pub async fn set_active(&mut self, pool: &PgPool, active: bool) -> Result<(), MemberError> {
        let mut transaction = Self::begin_member_mutation(pool).await?;

        let member = sqlx::query_as::<_, Member>(
            r#"
            UPDATE members
            SET active = $1
            WHERE id = $2
                AND (
                    role <> 'admin'
                    OR NOT active
                    OR $1
                    OR (SELECT COUNT(*) FROM members WHERE active AND role = 'admin') > 1
                )
            RETURNING id, name, role, active, created_at, last_login
            "#,
        )
        .bind(active)
        .bind(self.id)
        .fetch_optional(&mut *transaction)
        .await?;
        let Some(member) = member else {
            return Err(if Self::member_exists(&mut transaction, self.id).await? {
                MemberError::LastActiveAdmin
            } else {
                MemberError::NotFound
            });
        };

        transaction.commit().await?;
        *self = member;
        Ok(())
    }

    pub async fn delete(pool: &PgPool, member_id: Uuid) -> Result<(), MemberError> {
        let mut transaction = Self::begin_member_mutation(pool).await?;

        let deleted = sqlx::query(
            r#"
            DELETE FROM members
            WHERE id = $1
                AND (
                    role <> 'admin'
                    OR NOT active
                    OR (SELECT COUNT(*) FROM members WHERE active AND role = 'admin') > 1
                )
            RETURNING id
            "#,
        )
        .bind(member_id)
        .fetch_optional(&mut *transaction)
        .await?;
        if deleted.is_none() {
            return Err(if Self::member_exists(&mut transaction, member_id).await? {
                MemberError::LastActiveAdmin
            } else {
                MemberError::NotFound
            });
        }

        transaction.commit().await?;
        Ok(())
    }

    pub async fn update_last_login(&self, pool: &PgPool) -> Result<Self, MemberError> {
        Ok(sqlx::query_as::<_, Member>(
            r#"
            UPDATE members
            SET last_login = NOW()
            WHERE id = $1
            RETURNING id, name, role, active, created_at, last_login
            "#,
        )
        .bind(self.id)
        .fetch_one(pool)
        .await?)
    }

    async fn begin_member_mutation(
        pool: &PgPool,
    ) -> Result<sqlx::Transaction<'static, sqlx::Postgres>, MemberError> {
        let mut transaction = pool.begin().await?;
        sqlx::query("SELECT pg_advisory_xact_lock($1)")
            .bind(Self::ADMIN_MUTATION_LOCK)
            .execute(&mut *transaction)
            .await?;
        Ok(transaction)
    }

    async fn member_exists(
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        member_id: Uuid,
    ) -> Result<bool, MemberError> {
        Ok(
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM members WHERE id = $1)")
                .bind(member_id)
                .fetch_one(&mut **transaction)
                .await?,
        )
    }
}

#[cfg(feature = "ssr")]
fn validate_member_name(name: &str) -> Result<String, MemberError> {
    let name = name.trim();
    if name.is_empty() || name.chars().count() > 80 || name.chars().any(char::is_control) {
        return Err(MemberError::InvalidName);
    }
    Ok(name.to_string())
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn role_roundtrips() {
        assert_eq!(MemberRole::parse("admin"), Some(MemberRole::Admin));
        assert_eq!(MemberRole::parse("member"), Some(MemberRole::Member));
        assert_eq!(
            MemberRole::parse("participant"),
            Some(MemberRole::Participant)
        );
        assert_eq!(MemberRole::parse("nope"), None);
        assert!(MemberRole::Admin.is_admin());
        assert!(!MemberRole::Admin.is_participant());
        assert!(!MemberRole::Member.is_admin());
        assert!(!MemberRole::Member.is_participant());
        assert!(!MemberRole::Participant.is_admin());
        assert!(MemberRole::Participant.is_participant());
        assert_eq!(MemberRole::Admin.as_str(), "admin");
        assert_eq!(MemberRole::Member.as_str(), "member");
        assert_eq!(MemberRole::Participant.as_str(), "participant");
    }

    #[test]
    fn role_converts_from_database_value() {
        assert!(matches!(
            MemberRole::try_from("participant".to_string()),
            Ok(MemberRole::Participant)
        ));
        assert!(MemberRole::try_from("unknown".to_string()).is_err());
    }

    #[test]
    fn member_name_is_trimmed() {
        assert_eq!(
            validate_member_name("  Band Member  ").unwrap(),
            "Band Member"
        );
    }

    #[test]
    fn member_name_rejects_empty_control_and_overlong_values() {
        assert!(validate_member_name("  ").is_err());
        assert!(validate_member_name("Band\nMember").is_err());
        assert!(validate_member_name("a".repeat(81).as_str()).is_err());
    }

    #[test]
    fn member_name_accepts_up_to_eighty_characters() {
        let expected = "é".repeat(80);
        assert_eq!(validate_member_name(expected.as_str()).unwrap(), expected);
    }
}
