use crate::models::{Member, MemberRole, MemberSummary};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, sqlx::FromRow)]
struct MemberRow {
    id: Uuid,
    name: String,
    role: String,
    active: bool,
    created_at: chrono::DateTime<chrono::Utc>,
    last_login: Option<chrono::DateTime<chrono::Utc>>,
}

impl TryFrom<MemberRow> for Member {
    type Error = String;

    fn try_from(row: MemberRow) -> Result<Self, Self::Error> {
        let role = MemberRole::parse(&row.role)
            .ok_or_else(|| format!("unknown member role: {}", row.role))?;
        Ok(Member {
            id: row.id,
            name: row.name,
            role,
            active: row.active,
            created_at: row.created_at,
            last_login: row.last_login,
        })
    }
}

pub async fn list_active_members(pool: &PgPool) -> Result<Vec<MemberSummary>, sqlx::Error> {
    let rows = sqlx::query_as::<_, MemberRow>(
        r#"
        SELECT id, name, role, active, created_at, last_login
        FROM members
        WHERE active = TRUE
        ORDER BY name ASC
        "#,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .filter_map(|row| Member::try_from(row).ok())
        .map(MemberSummary::from)
        .collect())
}

pub async fn find_active_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Member>, sqlx::Error> {
    let row = sqlx::query_as::<_, MemberRow>(
        r#"
        SELECT id, name, role, active, created_at, last_login
        FROM members
        WHERE id = $1 AND active = TRUE
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    Ok(row.and_then(|r| Member::try_from(r).ok()))
}

pub async fn find_active_by_name(pool: &PgPool, name: &str) -> Result<Option<Member>, sqlx::Error> {
    let row = sqlx::query_as::<_, MemberRow>(
        r#"
        SELECT id, name, role, active, created_at, last_login
        FROM members
        WHERE lower(name) = lower($1) AND active = TRUE
        "#,
    )
    .bind(name)
    .fetch_optional(pool)
    .await?;

    Ok(row.and_then(|r| Member::try_from(r).ok()))
}

pub async fn touch_last_login(pool: &PgPool, id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE members
        SET last_login = NOW()
        WHERE id = $1
        "#,
    )
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}
