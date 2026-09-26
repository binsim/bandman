use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Band member role used for authorization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemberRole {
    Admin,
    Member,
}

impl MemberRole {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Admin => "admin",
            Self::Member => "member",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "admin" => Some(Self::Admin),
            "member" => Some(Self::Member),
            _ => None,
        }
    }

    pub fn is_admin(self) -> bool {
        matches!(self, Self::Admin)
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

/// Public view of a member suitable for the login picker (no timestamps required).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemberSummary {
    pub id: Uuid,
    pub name: String,
    pub role: MemberRole,
}

impl From<Member> for MemberSummary {
    fn from(member: Member) -> Self {
        Self {
            id: member.id,
            name: member.name,
            role: member.role,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn role_roundtrips() {
        assert_eq!(MemberRole::parse("admin"), Some(MemberRole::Admin));
        assert_eq!(MemberRole::parse("member"), Some(MemberRole::Member));
        assert_eq!(MemberRole::parse("nope"), None);
        assert!(MemberRole::Admin.is_admin());
        assert!(!MemberRole::Member.is_admin());
        assert_eq!(MemberRole::Admin.as_str(), "admin");
    }

    #[test]
    fn summary_from_member_keeps_identity() {
        let id = Uuid::nil();
        let member = Member {
            id,
            name: "Ada".into(),
            role: MemberRole::Member,
            active: true,
            created_at: Utc::now(),
            last_login: None,
        };
        let summary = MemberSummary::from(member);
        assert_eq!(summary.id, id);
        assert_eq!(summary.name, "Ada");
        assert_eq!(summary.role, MemberRole::Member);
    }
}
