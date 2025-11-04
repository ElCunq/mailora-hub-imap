// RBAC: Roller ve izinler
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Role {
    pub name: String,
    pub permissions: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct User {
    pub id: String,
    pub roles: Vec<Role>,
}

pub fn assign_role(user: &mut User, role: Role) {
    user.roles.push(role);
}
