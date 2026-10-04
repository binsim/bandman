use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
#[cfg(feature = "ssr")]
use sqlx::{postgres::PgRow, FromRow, PgPool, Row};
use std::str::FromStr;
use uuid::Uuid;

/// Band member role used for authorization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemberRole {
    Admin,
    Member,
    Participant,
}

impl FromStr for MemberRole {
    type Err = std::io::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "admin" => Ok(Self::Admin),
            "member" => Ok(Self::Member),
            "participant" => Ok(Self::Participant),
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("unknown member role: {value}"),
            )),
        }
    }
}

impl std::fmt::Display for MemberRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Admin => "admin",
            Self::Member => "member",
            Self::Participant => "participant",
        })
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
        let role = role
            .parse::<MemberRole>()
            .map_err(|error| sqlx::Error::ColumnDecode {
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

    pub async fn wishlist_show_needs_feedback(
        pool: &PgPool,
        member_id: Uuid,
    ) -> Result<bool, MemberError> {
        Ok(sqlx::query_scalar(
            "SELECT wishlist_show_needs_feedback FROM members WHERE id = $1 AND active = TRUE",
        )
        .bind(member_id)
        .fetch_optional(pool)
        .await?
        .ok_or(MemberError::NotFound)?)
    }

    pub async fn set_wishlist_show_needs_feedback(
        pool: &PgPool,
        member_id: Uuid,
        show_needs_feedback: bool,
    ) -> Result<(), MemberError> {
        let updated = sqlx::query(
            "UPDATE members SET wishlist_show_needs_feedback = $1 WHERE id = $2 AND active = TRUE",
        )
        .bind(show_needs_feedback)
        .bind(member_id)
        .execute(pool)
        .await?
        .rows_affected();
        if updated == 1 {
            Ok(())
        } else {
            Err(MemberError::NotFound)
        }
    }

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
        .bind(&name)
        .bind(role.to_string())
        .fetch_optional(&mut *transaction)
        .await?;
        let Some(member) = member else {
            return Err(MemberError::DuplicateName);
        };

        transaction.commit().await?;
        Ok(member)
    }

    pub async fn update_details(
        &mut self,
        pool: &PgPool,
        name: &str,
        role: MemberRole,
        active: bool,
    ) -> Result<(), MemberError> {
        let name = validate_member_name(name)?;
        let mut transaction = Self::begin_member_mutation(pool).await?;

        let member = sqlx::query_as::<_, Member>(
            r#"
            UPDATE members
            SET name = $1, role = $2, active = $3
            WHERE id = $4
                AND NOT EXISTS (
                    SELECT 1
                    FROM members
                    WHERE lower(name) = lower($1) AND id <> $4
                )
                AND (
                    role <> 'admin'
                    OR NOT active
                    OR ($2 = 'admin' AND $3)
                    OR (SELECT COUNT(*) FROM members WHERE active AND role = 'admin') > 1
                )
            RETURNING id, name, role, active, created_at, last_login
            "#,
        )
        .bind(&name)
        .bind(role.to_string())
        .bind(active)
        .bind(self.id)
        .fetch_optional(&mut *transaction)
        .await?;
        let Some(member) = member else {
            if !Self::member_exists(&mut transaction, self.id).await? {
                return Err(MemberError::NotFound);
            }

            let duplicate_name: bool = sqlx::query_scalar(
                "SELECT EXISTS(SELECT 1 FROM members WHERE lower(name) = lower($1) AND id <> $2)",
            )
            .bind(name)
            .bind(self.id)
            .fetch_one(&mut *transaction)
            .await?;
            return Err(if duplicate_name {
                MemberError::DuplicateName
            } else {
                MemberError::LastActiveAdmin
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

#[cfg(all(test, feature = "ssr"))]
mod tests {

    use super::*;

    #[sqlx::test(migrations = "./migrations")]
    async fn database_persists_and_retrieves_all_member_roles(pool: PgPool) {
        async fn assert_role_persists(pool: &PgPool, role: MemberRole) {
            let name = format!("Role {role} {}", Uuid::new_v4());
            let created = Member::create(pool, &name, role)
                .await
                .expect("member should be created with the role");
            let retrieved = Member::find_by_id(pool, created.id)
                .await
                .expect("member query should succeed")
                .expect("created member should exist");

            assert_eq!(created.role, role);
            assert_eq!(retrieved.role, role);
        }

        for role in [
            MemberRole::Admin,
            MemberRole::Member,
            MemberRole::Participant,
        ] {
            match role {
                MemberRole::Admin => assert_role_persists(&pool, role).await,
                MemberRole::Member => assert_role_persists(&pool, role).await,
                MemberRole::Participant => assert_role_persists(&pool, role).await,
            }
        }
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn wishlist_filter_preference_is_saved_per_member(pool: PgPool) {
        let first = Member::create(
            &pool,
            &format!("Filter preference {}", Uuid::new_v4()),
            MemberRole::Member,
        )
        .await
        .expect("member should be created");
        let second = Member::create(
            &pool,
            &format!("Filter preference {}", Uuid::new_v4()),
            MemberRole::Member,
        )
        .await
        .expect("member should be created");

        assert!(Member::wishlist_show_needs_feedback(&pool, first.id)
            .await
            .expect("default preference should load"));
        assert!(Member::wishlist_show_needs_feedback(&pool, second.id)
            .await
            .expect("second preference should load"));

        Member::set_wishlist_show_needs_feedback(&pool, first.id, false)
            .await
            .expect("preference should save");
        assert!(!Member::wishlist_show_needs_feedback(&pool, first.id)
            .await
            .expect("updated preference should load"));
        assert!(Member::wishlist_show_needs_feedback(&pool, second.id)
            .await
            .expect("other member preference should be unchanged"));
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

    #[sqlx::test(migrations = "./migrations")]
    async fn database_crud_persists_member_changes(pool: PgPool) {
        let original_name = format!("Member test {}", Uuid::new_v4());
        let mut member = Member::create(&pool, &original_name, MemberRole::Member)
            .await
            .expect("member should be created");

        assert_eq!(member.name, original_name);
        assert!(matches!(
            Member::create(&pool, &original_name.to_uppercase(), MemberRole::Member).await,
            Err(MemberError::DuplicateName)
        ));

        let renamed = format!("{original_name} renamed");
        member
            .update_details(&pool, &renamed, MemberRole::Participant, true)
            .await
            .expect("member details should update");
        member
            .update_details(&pool, &renamed, MemberRole::Participant, false)
            .await
            .expect("member should deactivate");

        let persisted = Member::find_by_id(&pool, member.id)
            .await
            .expect("member query should succeed")
            .expect("updated member should exist");
        assert_eq!(persisted.name, renamed);
        assert_eq!(persisted.role, MemberRole::Participant);
        assert!(!persisted.active);

        member
            .update_details(&pool, &renamed, MemberRole::Participant, true)
            .await
            .expect("member should reactivate");
        Member::delete(&pool, member.id)
            .await
            .expect("member should be deleted");
        assert!(Member::find_by_id(&pool, member.id)
            .await
            .expect("member query should succeed")
            .is_none());

        let mut seeded_admin = Member::find_active_by_name(&pool, "Admin")
            .await
            .expect("admin query should succeed")
            .expect("migration should seed an admin");
        assert!(matches!(
            seeded_admin
                .update_details(&pool, "Renamed admin", MemberRole::Member, true)
                .await,
            Err(MemberError::LastActiveAdmin)
        ));
        let persisted_admin = Member::find_by_id(&pool, seeded_admin.id)
            .await
            .expect("admin query should succeed")
            .expect("seeded admin should remain");
        assert_eq!(persisted_admin.name, "Admin");
        assert_eq!(persisted_admin.role, MemberRole::Admin);
        assert!(matches!(
            seeded_admin
                .update_details(&pool, "Admin", MemberRole::Admin, false)
                .await,
            Err(MemberError::LastActiveAdmin)
        ));
        assert!(matches!(
            Member::delete(&pool, seeded_admin.id).await,
            Err(MemberError::LastActiveAdmin)
        ));
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn database_member_updates_reject_duplicates_and_missing_members(pool: PgPool) {
        let first_name = format!("Member first {}", Uuid::new_v4());
        let second_name = format!("Member second {}", Uuid::new_v4());
        let mut first = Member::create(&pool, &first_name, MemberRole::Member)
            .await
            .expect("first member should be created");
        let second = Member::create(&pool, &second_name, MemberRole::Member)
            .await
            .expect("second member should be created");

        assert!(matches!(
            Member::create(&pool, "  ", MemberRole::Member).await,
            Err(MemberError::InvalidName)
        ));
        assert!(matches!(
            first
                .update_details(&pool, "  ", MemberRole::Participant, true)
                .await,
            Err(MemberError::InvalidName)
        ));
        assert!(matches!(
            first
                .update_details(
                    &pool,
                    &second_name.to_uppercase(),
                    MemberRole::Participant,
                    true,
                )
                .await,
            Err(MemberError::DuplicateName)
        ));
        let unchanged = Member::find_by_id(&pool, first.id)
            .await
            .expect("member query should succeed")
            .expect("first member should still exist");
        assert_eq!(unchanged.name, first_name);
        assert_eq!(unchanged.role, MemberRole::Member);

        Member::delete(&pool, first.id)
            .await
            .expect("first member should be deleted");
        assert!(matches!(
            first
                .update_details(&pool, "Renamed", MemberRole::Participant, false)
                .await,
            Err(MemberError::NotFound)
        ));
        assert!(matches!(
            Member::delete(&pool, first.id).await,
            Err(MemberError::NotFound)
        ));

        Member::delete(&pool, second.id)
            .await
            .expect("second member should be deleted");
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn database_member_queries_respect_active_status_and_update_login(pool: PgPool) {
        let name = format!("Member queries {}", Uuid::new_v4());
        let mut member = Member::create(&pool, &name, MemberRole::Participant)
            .await
            .expect("member should be created");
        assert!(member.last_login.is_none());

        let updated = member
            .update_last_login(&pool)
            .await
            .expect("last login should update");
        assert!(updated.last_login.is_some());

        member
            .update_details(&pool, &name, MemberRole::Participant, false)
            .await
            .expect("member should deactivate");
        assert!(Member::find_active_by_id(&pool, member.id)
            .await
            .expect("active member lookup should succeed")
            .is_none());
        assert!(Member::find_active_by_name(&pool, &name)
            .await
            .expect("active name lookup should succeed")
            .is_none());
        assert!(
            !Member::find_by_id(&pool, member.id)
                .await
                .expect("member lookup should succeed")
                .expect("inactive member should remain queryable")
                .active
        );
        assert!(Member::list_active(&pool)
            .await
            .expect("active member list should succeed")
            .iter()
            .all(|listed| listed.id != member.id));
        assert!(Member::list_all(&pool)
            .await
            .expect("all member list should succeed")
            .iter()
            .any(|listed| listed.id == member.id && !listed.active));
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn database_allows_admin_changes_when_another_active_admin_remains(pool: PgPool) {
        let name = format!("Second admin {}", Uuid::new_v4());
        let mut admin = Member::create(&pool, &name, MemberRole::Admin)
            .await
            .expect("second admin should be created");

        admin
            .update_details(&pool, &name, MemberRole::Member, true)
            .await
            .expect("one of multiple admins should be demoted");
        admin
            .update_details(&pool, &name, MemberRole::Admin, true)
            .await
            .expect("member should be promotable to admin");
        admin
            .update_details(&pool, &name, MemberRole::Admin, false)
            .await
            .expect("one of multiple admins should be deactivated");
        admin
            .update_details(&pool, &name, MemberRole::Admin, true)
            .await
            .expect("admin should be reactivated");
        Member::delete(&pool, admin.id)
            .await
            .expect("one of multiple admins should be deletable");

        assert!(Member::find_active_by_name(&pool, "Admin")
            .await
            .expect("seeded admin lookup should succeed")
            .is_some());
    }
}
