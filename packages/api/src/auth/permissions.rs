use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Permission {
    UsersManage,
    BooksManage,
    ResourcesManage,
    NotesCreate,
    NotesEdit,
    NotesDelete,
    ChatUse,
    ArticlesCreate,
    ArticlesPublish,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Role {
    Admin,
    Editor,
    User,
}

impl Role {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "admin" => Some(Self::Admin),
            "editor" => Some(Self::Editor),
            "user" => Some(Self::User),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Admin => "admin",
            Self::Editor => "editor",
            Self::User => "user",
        }
    }

    pub fn permissions(&self) -> &'static [Permission] {
        match self {
            Self::Admin => &[
                Permission::UsersManage,
                Permission::BooksManage,
                Permission::ResourcesManage,
                Permission::NotesCreate,
                Permission::NotesEdit,
                Permission::NotesDelete,
                Permission::ChatUse,
                Permission::ArticlesCreate,
                Permission::ArticlesPublish,
            ],
            Self::Editor => &[
                Permission::ResourcesManage,
                Permission::NotesCreate,
                Permission::NotesEdit,
                Permission::NotesDelete,
                Permission::ChatUse,
                Permission::ArticlesCreate,
                Permission::ArticlesPublish,
            ],
            Self::User => &[
                Permission::NotesCreate,
                Permission::NotesEdit,
                Permission::NotesDelete,
                Permission::ChatUse,
                Permission::ArticlesCreate,
                Permission::ArticlesPublish,
            ],
        }
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
