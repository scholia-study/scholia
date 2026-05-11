use std::collections::HashSet;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Permission {
    UsersManage,
    BooksManage,
    ResourcesManage,
    AdminPanel,
    NotesCreate,
    NotesEdit,
    NotesDelete,
    ArticlesCreate,
    ArticlesLimit1000,
    ArticlesArchiveLimit1000,
    QuotationsLimit10000,
    NotesLimit10000,
    SourcesCreate,
    ArticleLabelsManage,
}

impl Permission {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::UsersManage => "users_manage",
            Self::BooksManage => "books_manage",
            Self::ResourcesManage => "resources_manage",
            Self::AdminPanel => "admin_panel",
            Self::NotesCreate => "notes_create",
            Self::NotesEdit => "notes_edit",
            Self::NotesDelete => "notes_delete",
            Self::ArticlesCreate => "articles_create",
            Self::ArticlesLimit1000 => "articles_limit_1000",
            Self::ArticlesArchiveLimit1000 => "articles_archive_limit_1000",
            Self::QuotationsLimit10000 => "quotations_limit_10000",
            Self::NotesLimit10000 => "notes_limit_10000",
            Self::SourcesCreate => "sources_create",
            Self::ArticleLabelsManage => "article_labels_manage",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Role {
    Admin,
    Editor,
    User,
    Scholiast,
    ScholiastBenefactor,
    ScholiastPatron,
    /// Comp tier: granted manually by admins (e.g. as a thank-you to
    /// contributors). Same elevated limits as the paid tiers. Stripe
    /// webhook role-sync never touches this role, so a user keeps
    /// honorary access independent of any subscription state.
    Honorary,
}

/// Base permissions shared by all authenticated users.
const BASE_PERMISSIONS: &[Permission] = &[
    Permission::NotesCreate,
    Permission::NotesEdit,
    Permission::NotesDelete,
    Permission::ArticlesCreate,
    Permission::SourcesCreate,
];

/// Elevated limits for paid tiers and staff.
const ELEVATED_LIMITS: &[Permission] = &[
    Permission::ArticlesLimit1000,
    Permission::ArticlesArchiveLimit1000,
    Permission::QuotationsLimit10000,
    Permission::NotesLimit10000,
];

impl FromStr for Role {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "admin" => Ok(Self::Admin),
            "editor" => Ok(Self::Editor),
            "user" => Ok(Self::User),
            "scholiast" => Ok(Self::Scholiast),
            "scholiast_benefactor" => Ok(Self::ScholiastBenefactor),
            "scholiast_patron" => Ok(Self::ScholiastPatron),
            "honorary" => Ok(Self::Honorary),
            _ => Err(()),
        }
    }
}

impl Role {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Admin => "admin",
            Self::Editor => "editor",
            Self::User => "user",
            Self::Scholiast => "scholiast",
            Self::ScholiastBenefactor => "scholiast_benefactor",
            Self::ScholiastPatron => "scholiast_patron",
            Self::Honorary => "honorary",
        }
    }

    pub fn permissions(&self) -> Vec<Permission> {
        let mut perms: Vec<Permission> = BASE_PERMISSIONS.to_vec();
        match self {
            Self::Admin => {
                perms.push(Permission::UsersManage);
                perms.push(Permission::BooksManage);
                perms.push(Permission::ResourcesManage);
                perms.push(Permission::AdminPanel);
                perms.push(Permission::ArticleLabelsManage);
                perms.extend_from_slice(ELEVATED_LIMITS);
            }
            Self::Editor => {
                perms.push(Permission::ResourcesManage);
                perms.push(Permission::ArticleLabelsManage);
                perms.extend_from_slice(ELEVATED_LIMITS);
            }
            Self::User => {}
            Self::Scholiast | Self::ScholiastBenefactor | Self::ScholiastPatron => {
                perms.extend_from_slice(ELEVATED_LIMITS);
            }
            Self::Honorary => {
                perms.extend_from_slice(ELEVATED_LIMITS);
            }
        }
        perms
    }
}

/// Resolve all permissions from a set of role names.
pub fn resolve_permissions(role_names: &[String]) -> HashSet<Permission> {
    let mut perms = HashSet::new();
    for name in role_names {
        if let Ok(role) = Role::from_str(name) {
            perms.extend(role.permissions());
        }
    }
    perms
}

/// Roles that should appear as public chips on profile pages and
/// article bylines. Excludes `admin` (operational only) and `user`
/// (default role; would show for everyone).
const PUBLIC_ROLES: &[&str] = &[
    "editor",
    "scholiast",
    "scholiast_benefactor",
    "scholiast_patron",
];

/// Filter a role list to those that should render as public chips.
/// Returned in a stable order matching `PUBLIC_ROLES`.
pub fn filter_public_roles(roles: &[String]) -> Vec<String> {
    PUBLIC_ROLES
        .iter()
        .filter(|r| roles.iter().any(|owned| owned == *r))
        .map(|s| (*s).to_string())
        .collect()
}

/// Resolve all permissions from role names as a sorted list of string names.
pub fn resolve_permission_names(role_names: &[String]) -> Vec<String> {
    let mut names: Vec<String> = resolve_permissions(role_names)
        .into_iter()
        .map(|p| p.as_str().to_string())
        .collect();
    names.sort();
    names
}
