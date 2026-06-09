use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Role {
    ADMIN,
    STOREKEEPER,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub password: String,
    pub name: String,
    pub role: Role,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Employee {
    pub id: String,
    pub name: String,
    pub position: String,
    pub department: String,
    pub qr_code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub id: String,
    pub name: String,
    pub inventory_number: String,
    pub category_id: String,
    pub total_quantity: i32,
    pub qr_code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issuance {
    pub id: String,
    pub tool_id: String,
    pub employee_id: String,
    pub user_id: String,
    pub quantity: i32,
    pub issued_at: DateTime<Local>,
    pub returned_at: Option<DateTime<Local>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Database {
    pub users: Vec<User>,
    pub employees: Vec<Employee>,
    pub categories: Vec<Category>,
    pub tools: Vec<Tool>,
    pub issuances: Vec<Issuance>,
}

impl Database {
    pub fn load() -> Self {
        let path = Self::get_db_path();

        if path.exists() {
            match fs::read_to_string(&path) {
                Ok(content) => {
                    match serde_json::from_str(&content) {
                        Ok(db) => return db,
                        Err(e) => eprintln!("Error parsing database: {}", e),
                    }
                }
                Err(e) => eprintln!("Error reading database: {}", e),
            }
        }

        Self::create_default()
    }

    pub fn save(&self) {
        let path = Self::get_db_path();

        if let Some(parent) = path.parent() {
            if !parent.exists() {
                let _ = fs::create_dir_all(parent);
            }
        }

        match serde_json::to_string_pretty(self) {
            Ok(json) => {
                if let Err(e) = fs::write(&path, json) {
                    eprintln!("Error saving database: {}", e);
                }
            }
            Err(e) => eprintln!("Error serializing database: {}", e),
        }
    }

    fn get_db_path() -> PathBuf {
        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| PathBuf::from("."));

        exe_dir.join("data").join("data.json")
    }

    fn get_config_path() -> PathBuf {
        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| PathBuf::from("."));

        exe_dir.join("data").join("config.json")
    }

    pub fn load_config() -> (String, String) {
        let path = Self::get_config_path();
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(config) = serde_json::from_str::<serde_json::Value>(&content) {
                    let username = config["last_username"].as_str().unwrap_or("").to_string();
                    let password = config["last_password"].as_str().unwrap_or("").to_string();
                    return (username, password);
                }
            }
        }
        (String::new(), String::new())
    }

    pub fn save_config(username: &str, password: &str) {
        let path = Self::get_config_path();
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                let _ = fs::create_dir_all(parent);
            }
        }
        let config = serde_json::json!({
            "last_username": username,
            "last_password": password
        });
        let _ = fs::write(&path, serde_json::to_string_pretty(&config).unwrap_or_default());
    }

    fn create_default() -> Self {
        let mut db = Database::default();

        // Default users
        db.users.push(User {
            id: uuid::Uuid::new_v4().to_string(),
            username: "admin".to_string(),
            password: "admin123".to_string(),
            name: "Администратор".to_string(),
            role: Role::ADMIN,
        });

        db.users.push(User {
            id: uuid::Uuid::new_v4().to_string(),
            username: "storekeeper".to_string(),
            password: "store123".to_string(),
            name: "Кладовщик".to_string(),
            role: Role::STOREKEEPER,
        });

        // Default categories
        db.categories.push(Category {
            id: "cat1".to_string(),
            name: "Ручной инструмент".to_string(),
        });
        db.categories.push(Category {
            id: "cat2".to_string(),
            name: "Электроинструмент".to_string(),
        });
        db.categories.push(Category {
            id: "cat3".to_string(),
            name: "Измерительный инструмент".to_string(),
        });
        db.categories.push(Category {
            id: "cat4".to_string(),
            name: "Слесарный инструмент".to_string(),
        });

        db.save();
        db
    }

    // User methods
    pub fn authenticate(&self, username: &str, password: &str) -> Option<&User> {
        self.users
            .iter()
            .find(|u| u.username == username && u.password == password)
    }

    // Employee methods
    pub fn add_employee(&mut self, employee: Employee) {
        self.employees.push(employee);
        self.save();
    }

    pub fn update_employee(&mut self, employee: Employee) {
        if let Some(idx) = self.employees.iter().position(|e| e.id == employee.id) {
            self.employees[idx] = employee;
            self.save();
        }
    }

    pub fn delete_employee(&mut self, id: &str) {
        self.employees.retain(|e| e.id != id);
        self.save();
    }

    pub fn get_employee_by_qr(&self, qr_code: &str) -> Option<&Employee> {
        self.employees.iter().find(|e| e.qr_code == qr_code)
    }

    // Tool methods
    pub fn add_tool(&mut self, tool: Tool) {
        self.tools.push(tool);
        self.save();
    }

    pub fn update_tool(&mut self, tool: Tool) {
        if let Some(idx) = self.tools.iter().position(|t| t.id == tool.id) {
            self.tools[idx] = tool;
            self.save();
        }
    }

    pub fn delete_tool(&mut self, id: &str) {
        self.tools.retain(|t| t.id != id);
        self.save();
    }

    pub fn get_tool_by_qr(&self, qr_code: &str) -> Option<&Tool> {
        self.tools.iter().find(|t| t.qr_code == qr_code)
    }

    pub fn get_issued_quantity(&self, tool_id: &str) -> i32 {
        self.issuances
            .iter()
            .filter(|i| i.tool_id == tool_id && i.returned_at.is_none())
            .map(|i| i.quantity)
            .sum()
    }

    pub fn get_available_quantity(&self, tool_id: &str) -> i32 {
        let tool = self.tools.iter().find(|t| t.id == tool_id);
        if let Some(tool) = tool {
            tool.total_quantity - self.get_issued_quantity(tool_id)
        } else {
            0
        }
    }

    // Issuance methods
    pub fn issue_tool(&mut self, issuance: Issuance) -> Result<(), String> {
        let available = self.get_available_quantity(&issuance.tool_id);
        if issuance.quantity > available {
            return Err(format!("Недостаточно инструментов. Доступно: {}", available));
        }

        self.issuances.push(issuance);
        self.save();
        Ok(())
    }

    pub fn return_tool(&mut self, issuance_id: &str) {
        if let Some(issuance) = self.issuances.iter_mut().find(|i| i.id == issuance_id) {
            issuance.returned_at = Some(Local::now());
            self.save();
        }
    }

    // Write off tool (reduce total quantity)
    pub fn write_off_tool(&mut self, tool_id: &str, quantity: i32) -> Result<(), String> {
        // First get the issued quantity (immutable borrow)
        let issued = self.get_issued_quantity(tool_id);
        
        // Find tool and check availability
        let tool = self.tools.iter().find(|t| t.id == tool_id);
        if let Some(tool_ref) = tool {
            let available = tool_ref.total_quantity - issued;
            
            if quantity > available {
                return Err(format!("Нельзя списать больше чем доступно. Доступно: {}", available));
            }
            
            // Now mutate
            if let Some(tool) = self.tools.iter_mut().find(|t| t.id == tool_id) {
                tool.total_quantity -= quantity;
                self.save();
            }
            Ok(())
        } else {
            Err("Инструмент не найден".to_string())
        }
    }

    pub fn get_active_issuances(&self) -> Vec<&Issuance> {
        self.issuances.iter().filter(|i| i.returned_at.is_none()).collect()
    }

    pub fn get_employee_active_issuances(&self, employee_id: &str) -> Vec<&Issuance> {
        self.issuances
            .iter()
            .filter(|i| i.employee_id == employee_id && i.returned_at.is_none())
            .collect()
    }

    // Category methods
    pub fn get_category_name(&self, category_id: &str) -> String {
        self.categories
            .iter()
            .find(|c| c.id == category_id)
            .map(|c| c.name.clone())
            .unwrap_or_else(|| "Без категории".to_string())
    }

    pub fn add_category(&mut self, category: Category) {
        self.categories.push(category);
        self.save();
    }

    pub fn update_category(&mut self, category: Category) {
        if let Some(idx) = self.categories.iter().position(|c| c.id == category.id) {
            self.categories[idx] = category;
            self.save();
        }
    }

    pub fn delete_category(&mut self, id: &str) {
        self.categories.retain(|c| c.id != id);
        self.save();
    }

    // Statistics
    pub fn get_stats(&self) -> HashMap<String, i32> {
        let mut stats = HashMap::new();

        stats.insert("total_employees".to_string(), self.employees.len() as i32);
        stats.insert("total_tools".to_string(), self.tools.len() as i32);
        stats.insert("total_tool_quantity".to_string(), self.tools.iter().map(|t| t.total_quantity).sum());

        let active_issuances: Vec<_> = self.issuances.iter().filter(|i| i.returned_at.is_none()).collect();
        let issued_quantity: i32 = active_issuances.iter().map(|i| i.quantity).sum();

        stats.insert("active_issuances".to_string(), active_issuances.len() as i32);
        stats.insert("issued_quantity".to_string(), issued_quantity);

        stats
    }
}
