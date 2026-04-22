use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Permission {
    UsersManage,
    BooksManage,
    ResourcesManage,
    NotesCreate,
    NotesEdit,
    NotesDelete,
    ArticlesCreate,
    ArticlesLimit1000,
    ArticlesArchiveLimit1000,
    SourcesCreate,
}

impl Permission {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::UsersManage => "users_manage",
            Self::BooksManage => "books_manage",
            Self::ResourcesManage => "resources_manage",
            Self::NotesCreate => "notes_create",
            Self::NotesEdit => "notes_edit",
            Self::NotesDelete => "notes_delete",
            Self::ArticlesCreate => "articles_create",
            Self::ArticlesLimit1000 => "articles_limit_1000",
            Self::ArticlesArchiveLimit1000 => "articles_archive_limit_1000",
            Self::SourcesCreate => "sources_create",
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
}

/// Base permissions shared by all authenticated users.
const BASE_PERMISSIONS: &[Permission] = &[
    Permission::NotesCreate,
    Permission::NotesEdit,
    Permission::NotesDelete,
    Permission::ArticlesCreate,
    Permission::SourcesCreate,
];

/// Elevated article limits for paid tiers and staff.
const ELEVATED_LIMITS: &[Permission] = &[
    Permission::ArticlesLimit1000,
    Permission::ArticlesArchiveLimit1000,
];

impl Role {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "admin" => Some(Self::Admin),
            "editor" => Some(Self::Editor),
            "user" => Some(Self::User),
            "scholiast" => Some(Self::Scholiast),
            "scholiast_benefactor" => Some(Self::ScholiastBenefactor),
            "scholiast_patron" => Some(Self::ScholiastPatron),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Admin => "admin",
            Self::Editor => "editor",
            Self::User => "user",
            Self::Scholiast => "scholiast",
            Self::ScholiastBenefactor => "scholiast_benefactor",
            Self::ScholiastPatron => "scholiast_patron",
        }
    }

    pub fn permissions(&self) -> Vec<Permission> {
        let mut perms: Vec<Permission> = BASE_PERMISSIONS.to_vec();
        match self {
            Self::Admin => {
                perms.push(Permission::UsersManage);
                perms.push(Permission::BooksManage);
                perms.push(Permission::ResourcesManage);
                perms.extend_from_slice(ELEVATED_LIMITS);
            }
            Self::Editor => {
                perms.push(Permission::ResourcesManage);
                perms.extend_from_slice(ELEVATED_LIMITS);
            }
            Self::User => {}
            Self::Scholiast | Self::ScholiastBenefactor | Self::ScholiastPatron => {
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
        if let Some(role) = Role::from_str(name) {
            perms.extend(role.permissions());
        }
    }
    perms
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
