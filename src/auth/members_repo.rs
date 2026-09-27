use crate::models::{Member, MemberRole, MemberSummary};
use sqlx::PgPool;
use uuid::Uuid;

const ADMIN_MUTATION_LOCK: i64 = 6_204_119_021;

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

pub async fn list_members(pool: &PgPool) -> Result<Vec<Member>, String> {
    let rows = sqlx::query_as::<_, MemberRow>(
        r#"
        SELECT id, name, role, active, created_at, last_login
        FROM members
        ORDER BY active DESC, name ASC
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(|error| error.to_string())?;

    rows.into_iter().map(Member::try_from).collect()
}

pub async fn create_member(
    pool: &PgPool,
    admin_id: Uuid,
    name: &str,
    role: MemberRole,
) -> Result<Member, String> {
    let mut transaction = pool.begin().await.map_err(|error| error.to_string())?;
    lock_admin_mutations(&mut transaction).await?;
    ensure_active_admin(&mut transaction, admin_id).await?;

    let duplicate = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM members WHERE lower(name) = lower($1))",
    )
    .bind(name)
    .fetch_one(&mut *transaction)
    .await
    .map_err(|error| error.to_string())?;

    if duplicate {
        return Err("A member with this name already exists".to_string());
    }

    let row = sqlx::query_as::<_, MemberRow>(
        r#"
        INSERT INTO members (id, name, role)
        VALUES ($1, $2, $3)
        RETURNING id, name, role, active, created_at, last_login
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(name)
    .bind(role.as_str())
    .fetch_one(&mut *transaction)
    .await
    .map_err(|error| error.to_string())?;

    transaction
        .commit()
        .await
        .map_err(|error| error.to_string())?;
    Member::try_from(row)
}

pub async fn set_member_name(
    pool: &PgPool,
    admin_id: Uuid,
    member_id: Uuid,
    name: &str,
) -> Result<(), String> {
    let mut transaction = pool.begin().await.map_err(|error| error.to_string())?;
    lock_admin_mutations(&mut transaction).await?;
    ensure_active_admin(&mut transaction, admin_id).await?;

    let duplicate = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM members WHERE lower(name) = lower($1) AND id <> $2)",
    )
    .bind(name)
    .bind(member_id)
    .fetch_one(&mut *transaction)
    .await
    .map_err(|error| error.to_string())?;

    if duplicate {
        return Err("A member with this name already exists".to_string());
    }

    let result = sqlx::query("UPDATE members SET name = $1 WHERE id = $2")
        .bind(name)
        .bind(member_id)
        .execute(&mut *transaction)
        .await
        .map_err(|error| error.to_string())?;
    if result.rows_affected() == 0 {
        return Err("Member not found".to_string());
    }

    transaction
        .commit()
        .await
        .map_err(|error| error.to_string())
}

pub async fn set_member_role(
    pool: &PgPool,
    admin_id: Uuid,
    member_id: Uuid,
    role: MemberRole,
) -> Result<(), String> {
    let mut transaction = pool.begin().await.map_err(|error| error.to_string())?;
    lock_admin_mutations(&mut transaction).await?;
    ensure_active_admin(&mut transaction, admin_id).await?;

    let current = sqlx::query_as::<_, (String, bool)>(
        "SELECT role, active FROM members WHERE id = $1 FOR UPDATE",
    )
    .bind(member_id)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(|error| error.to_string())?
    .ok_or_else(|| "Member not found".to_string())?;

    if current.0 == MemberRole::Admin.as_str()
        && current.1
        && !role.is_admin()
        && active_admin_count(&mut transaction).await? <= 1
    {
        return Err("At least one active admin must remain".to_string());
    }

    sqlx::query("UPDATE members SET role = $1 WHERE id = $2")
        .bind(role.as_str())
        .bind(member_id)
        .execute(&mut *transaction)
        .await
        .map_err(|error| error.to_string())?;

    transaction
        .commit()
        .await
        .map_err(|error| error.to_string())
}

pub async fn set_member_active(
    pool: &PgPool,
    admin_id: Uuid,
    member_id: Uuid,
    active: bool,
) -> Result<(), String> {
    let mut transaction = pool.begin().await.map_err(|error| error.to_string())?;
    lock_admin_mutations(&mut transaction).await?;
    ensure_active_admin(&mut transaction, admin_id).await?;

    let current = sqlx::query_as::<_, (String, bool)>(
        "SELECT role, active FROM members WHERE id = $1 FOR UPDATE",
    )
    .bind(member_id)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(|error| error.to_string())?
    .ok_or_else(|| "Member not found".to_string())?;

    if current.0 == MemberRole::Admin.as_str()
        && current.1
        && !active
        && active_admin_count(&mut transaction).await? <= 1
    {
        return Err("At least one active admin must remain".to_string());
    }

    sqlx::query("UPDATE members SET active = $1 WHERE id = $2")
        .bind(active)
        .bind(member_id)
        .execute(&mut *transaction)
        .await
        .map_err(|error| error.to_string())?;

    transaction
        .commit()
        .await
        .map_err(|error| error.to_string())
}

async fn lock_admin_mutations(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> Result<(), String> {
    sqlx::query("SELECT pg_advisory_xact_lock($1)")
        .bind(ADMIN_MUTATION_LOCK)
        .execute(&mut **transaction)
        .await
        .map_err(|error| error.to_string())?;
    Ok(())
}

async fn ensure_active_admin(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    admin_id: Uuid,
) -> Result<(), String> {
    let is_admin = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM members WHERE id = $1 AND active AND role = 'admin')",
    )
    .bind(admin_id)
    .fetch_one(&mut **transaction)
    .await
    .map_err(|error| error.to_string())?;

    if is_admin {
        Ok(())
    } else {
        Err("Administrator access required".to_string())
    }
}

async fn active_admin_count(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> Result<i64, String> {
    sqlx::query_scalar("SELECT COUNT(*) FROM members WHERE active AND role = 'admin'")
        .fetch_one(&mut **transaction)
        .await
        .map_err(|error| error.to_string())
}
