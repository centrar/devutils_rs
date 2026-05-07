//! Team Management Module

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Team member roles
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TeamRole {
    Owner,
    Admin,
    Member,
    Viewer,
}

impl ToString for TeamRole {
    fn to_string(&self) -> String {
        match self {
            TeamRole::Owner => "owner".to_string(),
            TeamRole::Admin => "admin".to_string(),
            TeamRole::Member => "member".to_string(),
            TeamRole::Viewer => "viewer".to_string(),
        }
    }
}

/// Team member
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamMember {
    pub user_id: String,
    pub email: String,
    pub name: String,
    pub role: TeamRole,
    pub joined_at: u64,
    pub last_active: u64,
    pub permissions: Vec<String>,
}

/// Team
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Team {
    pub id: String,
    pub name: String,
    pub description: String,
    pub members: Vec<TeamMember>,
    pub created_at: u64,
    pub settings: TeamSettings,
}

/// Team settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamSettings {
    pub allow_external_sharing: bool,
    pub require_2fa: bool,
    pub audit_logs_enabled: bool,
    pub max_members: u32,
}

impl Default for TeamSettings {
    fn default() -> Self {
        Self {
            allow_external_sharing: false,
            require_2fa: false,
            audit_logs_enabled: true,
            max_members: 100,
        }
    }
}

/// Team manager
pub struct TeamManager {
    teams: HashMap<String, Team>,
}

impl TeamManager {
    pub fn new() -> Self {
        Self {
            teams: HashMap::new(),
        }
    }

    /// Create a new team
    pub fn create_team(&mut self, name: &str, description: &str, _owner_id: &str) -> String {
        let team_id = format!("team_{}", uuid_simple());
        let team = Team {
            id: team_id.clone(),
            name: name.to_string(),
            description: description.to_string(),
            members: vec![],
            created_at: 0,
            settings: TeamSettings::default(),
        };

        self.teams.insert(team_id.clone(), team);
        team_id
    }

    /// Add member to team
    pub fn add_member(
        &mut self,
        team_id: &str,
        user_id: &str,
        email: &str,
        name: &str,
        role: TeamRole,
    ) -> Result<(), String> {
        let team = self
            .teams
            .get_mut(team_id)
            .ok_or("Team not found")?;

        if team.members.len() as u32 >= team.settings.max_members {
            return Err("Team is full".to_string());
        }

        // Check if already a member
        if team.members.iter().any(|m| m.user_id == user_id) {
            return Err("User is already a member".to_string());
        }

        let member = TeamMember {
            user_id: user_id.to_string(),
            email: email.to_string(),
            name: name.to_string(),
            role,
            joined_at: 0,
            last_active: 0,
            permissions: vec![],
        };

        team.members.push(member);
        Ok(())
    }

    /// Remove member from team
    pub fn remove_member(&mut self, team_id: &str, user_id: &str) -> Result<(), String> {
        let team = self
            .teams
            .get_mut(team_id)
            .ok_or("Team not found")?;

        let initial_len = team.members.len();
        team.members.retain(|m| m.user_id != user_id);

        if team.members.len() == initial_len {
            Err("Member not found".to_string())
        } else {
            Ok(())
        }
    }

    /// Update member role
    pub fn update_member_role(
        &mut self,
        team_id: &str,
        user_id: &str,
        role: TeamRole,
    ) -> Result<(), String> {
        let team = self
            .teams
            .get_mut(team_id)
            .ok_or("Team not found")?;

        let member = team
            .members
            .iter_mut()
            .find(|m| m.user_id == user_id)
            .ok_or("Member not found")?;

        member.role = role;
        Ok(())
    }

    /// Get team by ID
    pub fn get_team(&self, team_id: &str) -> Option<&Team> {
        self.teams.get(team_id)
    }

    /// List all teams
    pub fn list_teams(&self) -> Vec<&Team> {
        self.teams.values().collect()
    }

    /// Get team members
    pub fn get_members(&self, team_id: &str) -> Option<Vec<&TeamMember>> {
        self.teams.get(team_id).map(|t| t.members.iter().collect())
    }

    /// Check if user has permission
    pub fn has_permission(&self, team_id: &str, user_id: &str, permission: &str) -> bool {
        if let Some(team) = self.teams.get(team_id) {
            if let Some(member) = team.members.iter().find(|m| m.user_id == user_id) {
                return member.permissions.contains(&permission.to_string())
                    || member.role == TeamRole::Owner
                    || member.role == TeamRole::Admin;
            }
        }
        false
    }
}

impl Default for TeamManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Generate a simple unique ID
fn uuid_simple() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    format!("{:x}", timestamp)
}

/// CLI command for team management
pub fn run_team_cmd(subcmd: &str, args: &[String]) -> Result<String, String> {
    let mut manager = TeamManager::new();

    match subcmd {
        "create" => {
            let name = args.first().ok_or("Please specify team name")?;
            let description = args.get(1).map(|s| s.as_str()).unwrap_or("Default team");
            let owner = std::env::var("USER").unwrap_or_else(|_| "owner".to_string());

            let team_id = manager.create_team(name, description, &owner);
            Ok(format!("Created team '{}' with ID: {}", name, team_id))
        }
        "list" | "ls" => {
            let teams = manager.list_teams();
            if teams.is_empty() {
                Ok("No teams found".to_string())
            } else {
                let mut output = String::from("Teams:\n");
                for team in teams {
                    output.push_str(&format!(
                        "- {} ({} members)\n",
                        team.name,
                        team.members.len()
                    ));
                }
                Ok(output)
            }
        }
        "add-member" => {
            let team_id = args.first().ok_or("Please specify team ID")?;
            let user_id = args.get(1).ok_or("Please specify user ID")?;
            let email = args.get(2).map(|s| s.as_str()).unwrap_or("user@example.com");
            let name = args.get(3).map(|s| s.as_str()).unwrap_or("User");

            manager
                .add_member(team_id, user_id, email, name, TeamRole::Member)
                .map(|_| "Member added successfully".to_string())
        }
        "remove-member" => {
            let team_id = args.first().ok_or("Please specify team ID")?;
            let user_id = args.get(1).ok_or("Please specify user ID")?;

            manager
                .remove_member(team_id, user_id)
                .map(|_| "Member removed successfully".to_string())
        }
        "members" => {
            let team_id = args.first().ok_or("Please specify team ID")?;

            if let Some(members) = manager.get_members(team_id) {
                let mut output = format!("Team {} members:\n", team_id);
                for member in members {
                    output.push_str(&format!(
                        "- {} ({}) - {:?}\n",
                        member.name, member.email, member.role
                    ));
                }
                Ok(output)
            } else {
                Ok("Team not found".to_string())
            }
        }
        "settings" => {
            let team_id = args.first().ok_or("Please specify team ID")?;

            if let Some(team) = manager.get_team(team_id) {
                Ok(format!(
                    "Team Settings for '{}':\n\
                     - Max Members: {}\n\
                     - External Sharing: {}\n\
                     - Require 2FA: {}\n\
                     - Audit Logs: {}",
                    team.name,
                    team.settings.max_members,
                    team.settings.allow_external_sharing,
                    team.settings.require_2fa,
                    team.settings.audit_logs_enabled
                ))
            } else {
                Ok("Team not found".to_string())
            }
        }
        _ => Err(format!("Unknown team command: {}", subcmd)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_team() {
        let mut manager = TeamManager::new();
        let team_id = manager.create_team("Test Team", "A test team", "user123");
        assert!(team_id.starts_with("team_"));
    }

    #[test]
    fn test_add_member() {
        let mut manager = TeamManager::new();
        let team_id = manager.create_team("Test Team", "A test team", "user123");

        let result = manager.add_member(
            &team_id,
            "user456",
            "user@example.com",
            "Test User",
            TeamRole::Member,
        );

        assert!(result.is_ok());
    }
}
