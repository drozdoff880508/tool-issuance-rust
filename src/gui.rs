use crate::data::{Database, Employee, Issuance, Role, Tool, User};
use chrono::Local;
use eframe::egui;
use egui::{Color32, FontId, RichText, Vec2};

// Truncate text to max characters with ellipsis
fn truncate_text(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        text.to_string()
    } else {
        let truncated: String = text.chars().take(max_chars - 3).collect();
        format!("{}...", truncated)
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Screen {
    Login,
    Admin(AdminTab),
    Terminal,
}

#[derive(Debug, Clone, PartialEq)]
enum AdminTab {
    Overview,
    Employees,
    Tools,
    Issuances,
}

pub struct App {
    db: Database,
    current_user: Option<User>,
    screen: Screen,

    // Login
    login_username: String,
    login_password: String,
    login_error: String,
    remember_me: bool,

    // Search
    employee_search: String,
    tool_search: String,
    issuance_search: String,

    // Employee form
    employee_name: String,
    employee_position: String,
    employee_department: String,
    editing_employee_id: Option<String>,

    // Tool form
    tool_name: String,
    tool_inventory: String,
    tool_category: String,
    tool_quantity_str: String,
    editing_tool_id: Option<String>,

    // Write off
    write_off_tool_id: Option<String>,
    write_off_quantity_str: String,

    // Delete confirmation dialogs
    delete_employee_id: Option<String>,
    delete_employee_name: String,
    delete_tool_id: Option<String>,
    delete_tool_name: String,

    // Terminal
    scan_input: String,
    scanned_employee: Option<Employee>,
    scanned_tool: Option<Tool>,
    issue_quantity: i32,
    terminal_message: String,
    terminal_error: String,
}

impl App {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let db = Database::load();
        let (saved_username, saved_password) = Database::load_config();
        let remember_me = !saved_username.is_empty();

        Self {
            db,
            current_user: None,
            screen: Screen::Login,
            login_username: saved_username,
            login_password: saved_password,
            login_error: String::new(),
            remember_me,
            employee_search: String::new(),
            tool_search: String::new(),
            issuance_search: String::new(),
            employee_name: String::new(),
            employee_position: String::new(),
            employee_department: String::new(),
            editing_employee_id: None,
            tool_name: String::new(),
            tool_inventory: String::new(),
            tool_category: String::new(),
            tool_quantity_str: "1".to_string(),
            editing_tool_id: None,
            write_off_tool_id: None,
            write_off_quantity_str: "1".to_string(),
            delete_employee_id: None,
            delete_employee_name: String::new(),
            delete_tool_id: None,
            delete_tool_name: String::new(),
            scan_input: String::new(),
            scanned_employee: None,
            scanned_tool: None,
            issue_quantity: 1,
            terminal_message: String::new(),
            terminal_error: String::new(),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        match &self.screen {
            Screen::Login => self.show_login(ctx),
            Screen::Admin(tab) => self.show_admin(ctx, tab.clone()),
            Screen::Terminal => self.show_terminal(ctx),
        }
    }
}

impl App {
    fn show_login(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(80.0);
                ui.heading(RichText::new("Система выдачи инструментов").font(FontId::proportional(36.0)));
                ui.add_space(50.0);

                egui::Frame::none()
                    .fill(Color32::from_rgb(240, 240, 240))
                    .inner_margin(30.0)
                    .show(ui, |ui| {
                        ui.set_min_width(400.0);

                        ui.label(RichText::new("Логин:").font(FontId::proportional(18.0)));
                        ui.add(egui::TextEdit::singleline(&mut self.login_username)
                            .desired_width(300.0)
                            .font(FontId::proportional(18.0)));

                        ui.add_space(15.0);
                        ui.label(RichText::new("Пароль:").font(FontId::proportional(18.0)));
                        ui.add(egui::TextEdit::singleline(&mut self.login_password)
                            .password(true)
                            .desired_width(300.0)
                            .font(FontId::proportional(18.0)));

                        ui.add_space(10.0);
                        ui.checkbox(&mut self.remember_me, "Запомнить меня");

                        if !self.login_error.is_empty() {
                            ui.add_space(10.0);
                            ui.colored_label(Color32::RED, RichText::new(&self.login_error).font(FontId::proportional(16.0)));
                        }

                        ui.add_space(25.0);
                        if ui.add_sized(egui::vec2(150.0, 45.0), egui::Button::new(RichText::new("Войти").font(FontId::proportional(20.0)))).clicked() {
                            if let Some(user) = self.db.authenticate(&self.login_username, &self.login_password) {
                                if self.remember_me {
                                    Database::save_config(&self.login_username, &self.login_password);
                                } else {
                                    Database::save_config("", "");
                                }
                                self.current_user = Some(user.clone());
                                self.login_error.clear();

                                if user.role == Role::ADMIN {
                                    self.screen = Screen::Admin(AdminTab::Overview);
                                } else {
                                    self.screen = Screen::Terminal;
                                }
                            } else {
                                self.login_error = "Неверный логин или пароль".to_string();
                            }
                        }
                    });
            });
        });
    }

    fn show_admin(&mut self, ctx: &egui::Context, tab: AdminTab) {
        egui::TopBottomPanel::top("header").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading(RichText::new("Панель администратора").font(FontId::proportional(22.0)));
                ui.add_space(20.0);

                if ui.selectable_label(tab == AdminTab::Overview, RichText::new("Обзор").font(FontId::proportional(18.0))).clicked() {
                    self.screen = Screen::Admin(AdminTab::Overview);
                }
                if ui.selectable_label(tab == AdminTab::Employees, RichText::new("Сотрудники").font(FontId::proportional(18.0))).clicked() {
                    self.screen = Screen::Admin(AdminTab::Employees);
                }
                if ui.selectable_label(tab == AdminTab::Tools, RichText::new("Инструменты").font(FontId::proportional(18.0))).clicked() {
                    self.screen = Screen::Admin(AdminTab::Tools);
                }
                if ui.selectable_label(tab == AdminTab::Issuances, RichText::new("Выдачи").font(FontId::proportional(18.0))).clicked() {
                    self.screen = Screen::Admin(AdminTab::Issuances);
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(RichText::new("Выход").font(FontId::proportional(18.0))).clicked() {
                        self.current_user = None;
                        self.screen = Screen::Login;
                        let (saved_user, saved_pass) = Database::load_config();
                        self.login_username = saved_user;
                        self.login_password = saved_pass;
                    }
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            match tab {
                AdminTab::Overview => self.show_overview(ui),
                AdminTab::Employees => self.show_employees(ui),
                AdminTab::Tools => self.show_tools(ui),
                AdminTab::Issuances => self.show_issuances(ui),
            }
        });
    }

    fn show_overview(&mut self, ui: &mut egui::Ui) {
        ui.heading(RichText::new("Обзор").font(FontId::proportional(24.0)));
        ui.add_space(20.0);

        let stats = self.db.get_stats();

        egui::Grid::new("stats_grid").num_columns(3).spacing([20.0, 10.0]).show(ui, |ui| {
            ui.group(|ui| {
                ui.set_min_size(Vec2::new(220.0, 120.0));
                ui.vertical_centered(|ui| {
                    ui.add_space(15.0);
                    ui.label(RichText::new("Сотрудников").font(FontId::proportional(18.0)));
                    ui.label(RichText::new(stats.get("total_employees").unwrap_or(&0).to_string())
                        .font(FontId::proportional(40.0)).color(Color32::from_rgb(52, 152, 219)));
                });
            });

            ui.group(|ui| {
                ui.set_min_size(Vec2::new(220.0, 120.0));
                ui.vertical_centered(|ui| {
                    ui.add_space(15.0);
                    ui.label(RichText::new("Видов инструментов").font(FontId::proportional(18.0)));
                    ui.label(RichText::new(stats.get("total_tools").unwrap_or(&0).to_string())
                        .font(FontId::proportional(40.0)).color(Color32::from_rgb(46, 204, 113)));
                });
            });

            ui.group(|ui| {
                ui.set_min_size(Vec2::new(220.0, 120.0));
                ui.vertical_centered(|ui| {
                    ui.add_space(15.0);
                    ui.label(RichText::new("Всего единиц").font(FontId::proportional(18.0)));
                    ui.label(RichText::new(stats.get("total_tool_quantity").unwrap_or(&0).to_string())
                        .font(FontId::proportional(40.0)).color(Color32::from_rgb(155, 89, 182)));
                });
            });

            ui.end_row();

            ui.group(|ui| {
                ui.set_min_size(Vec2::new(220.0, 120.0));
                ui.vertical_centered(|ui| {
                    ui.add_space(15.0);
                    ui.label(RichText::new("Выдано единиц").font(FontId::proportional(18.0)));
                    ui.label(RichText::new(stats.get("issued_quantity").unwrap_or(&0).to_string())
                        .font(FontId::proportional(40.0)).color(Color32::from_rgb(231, 76, 60)));
                });
            });

            ui.group(|ui| {
                ui.set_min_size(Vec2::new(220.0, 120.0));
                ui.vertical_centered(|ui| {
                    ui.add_space(15.0);
                    ui.label(RichText::new("Активных выдач").font(FontId::proportional(18.0)));
                    ui.label(RichText::new(stats.get("active_issuances").unwrap_or(&0).to_string())
                        .font(FontId::proportional(40.0)).color(Color32::from_rgb(230, 126, 34)));
                });
            });
        });

        ui.add_space(30.0);
        ui.heading(RichText::new("Последние выдачи").font(FontId::proportional(22.0)));
        ui.add_space(10.0);

        egui::ScrollArea::horizontal().show(ui, |ui| {
            let active: Vec<_> = self.db.issuances.iter()
                .filter(|i| i.returned_at.is_none())
                .collect();

            if active.is_empty() {
                ui.label(RichText::new("Нет активных выдач").font(FontId::proportional(18.0)));
            } else {
                egui::Grid::new("recent_issuances")
                    .min_col_width(150.0)
                    .spacing([15.0, 10.0])
                    .show(ui, |ui| {
                        ui.label(RichText::new("Инструмент").font(FontId::proportional(18.0)).strong());
                        ui.label(RichText::new("Сотрудник").font(FontId::proportional(18.0)).strong());
                        ui.label(RichText::new("Кол-во").font(FontId::proportional(18.0)).strong());
                        ui.label(RichText::new("Дата выдачи").font(FontId::proportional(18.0)).strong());
                        ui.end_row();

                        for issuance in active.iter().take(20) {
                            let tool = self.db.tools.iter().find(|t| t.id == issuance.tool_id);
                            let employee = self.db.employees.iter().find(|e| e.id == issuance.employee_id);

                            let tool_name = tool.map(|t| t.name.as_str()).unwrap_or("Неизвестно");
                            let emp_name = employee.map(|e| e.name.as_str()).unwrap_or("Неизвестно");
                            
                            let tool_display = truncate_text(tool_name, 35);
                            let emp_display = truncate_text(emp_name, 30);

                            ui.label(RichText::new(&tool_display).font(FontId::proportional(18.0)))
                                .on_hover_text(tool_name);
                            ui.label(RichText::new(&emp_display).font(FontId::proportional(18.0)))
                                .on_hover_text(emp_name);
                            ui.label(RichText::new(issuance.quantity.to_string()).font(FontId::proportional(18.0)));
                            ui.label(RichText::new(issuance.issued_at.format("%d.%m.%Y %H:%M").to_string()).font(FontId::proportional(18.0)));
                            ui.end_row();
                        }
                    });
            }
        });
    }

    fn show_employees(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading(RichText::new("Сотрудники").font(FontId::proportional(24.0)));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button(RichText::new("+ Добавить").font(FontId::proportional(18.0))).clicked() {
                    self.employee_name.clear();
                    self.employee_position.clear();
                    self.employee_department.clear();
                    self.editing_employee_id = None;
                }
            });
        });

        ui.add_space(10.0);

        // Search field
        ui.horizontal(|ui| {
            ui.label(RichText::new("Поиск:").font(FontId::proportional(18.0)));
            ui.add(egui::TextEdit::singleline(&mut self.employee_search)
                .desired_width(300.0)
                .hint_text("Поиск по ФИО, должности, отделу...")
                .font(FontId::proportional(18.0)));
            if ui.button(RichText::new("Очистить").font(FontId::proportional(16.0))).clicked() {
                self.employee_search.clear();
            }
        });

        ui.add_space(10.0);

        // Add/Edit form
        egui::CollapsingHeader::new(RichText::new(if self.editing_employee_id.is_some() { "Редактировать сотрудника" } else { "Добавить сотрудника" }).font(FontId::proportional(20.0)))
            .default_open(self.editing_employee_id.is_some())
            .show(ui, |ui| {
                egui::Grid::new("employee_form").num_columns(2).spacing([20.0, 10.0]).show(ui, |ui| {
                    ui.label(RichText::new("ФИО:").font(FontId::proportional(18.0)));
                    ui.add(egui::TextEdit::singleline(&mut self.employee_name).desired_width(300.0).font(FontId::proportional(18.0)));
                    ui.end_row();

                    ui.label(RichText::new("Должность:").font(FontId::proportional(18.0)));
                    ui.add(egui::TextEdit::singleline(&mut self.employee_position).desired_width(300.0).font(FontId::proportional(18.0)));
                    ui.end_row();

                    ui.label(RichText::new("Отдел:").font(FontId::proportional(18.0)));
                    ui.add(egui::TextEdit::singleline(&mut self.employee_department).desired_width(300.0).font(FontId::proportional(18.0)));
                    ui.end_row();
                });

                ui.add_space(15.0);
                ui.horizontal(|ui| {
                    if ui.button(RichText::new(if self.editing_employee_id.is_some() { "Сохранить" } else { "Добавить" }).font(FontId::proportional(18.0))).clicked() {
                        if !self.employee_name.is_empty() {
                            let id = self.editing_employee_id.clone().unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
                            let qr_code = format!("EMP_{}", &id[..8]);

                            let employee = Employee {
                                id: id.clone(),
                                name: self.employee_name.clone(),
                                position: self.employee_position.clone(),
                                department: self.employee_department.clone(),
                                qr_code,
                            };

                            if self.editing_employee_id.is_some() {
                                self.db.update_employee(employee);
                            } else {
                                self.db.add_employee(employee);
                            }

                            self.employee_name.clear();
                            self.employee_position.clear();
                            self.employee_department.clear();
                            self.editing_employee_id = None;
                        }
                    }

                    if self.editing_employee_id.is_some() && ui.button(RichText::new("Отмена").font(FontId::proportional(18.0))).clicked() {
                        self.employee_name.clear();
                        self.employee_position.clear();
                        self.employee_department.clear();
                        self.editing_employee_id = None;
                    }
                });
            });

        ui.add_space(20.0);

        // Filter employees by search - clone data to avoid borrow issues
        let search_lower = self.employee_search.to_lowercase();
        let filtered_employees: Vec<Employee> = self.db.employees.iter()
            .filter(|e| {
                if search_lower.is_empty() {
                    true
                } else {
                    e.name.to_lowercase().contains(&search_lower) ||
                    e.position.to_lowercase().contains(&search_lower) ||
                    e.department.to_lowercase().contains(&search_lower) ||
                    e.qr_code.to_lowercase().contains(&search_lower)
                }
            })
            .cloned()
            .collect();

        let is_search_empty = self.employee_search.is_empty();
        let is_filtered_empty = filtered_employees.is_empty();

        // Employees table with horizontal scroll
        egui::ScrollArea::horizontal().show(ui, |ui| {
            egui::Grid::new("employees_table")
                .min_col_width(150.0)
                .spacing([15.0, 10.0])
                .show(ui, |ui| {
                    ui.label(RichText::new("ФИО").font(FontId::proportional(18.0)).strong());
                    ui.label(RichText::new("Должность").font(FontId::proportional(18.0)).strong());
                    ui.label(RichText::new("Отдел").font(FontId::proportional(18.0)).strong());
                    ui.label(RichText::new("QR код").font(FontId::proportional(18.0)).strong());
                    ui.label(RichText::new("Действия").font(FontId::proportional(18.0)).strong());
                    ui.end_row();

                    for employee in &filtered_employees {
                        let name_display = truncate_text(&employee.name, 30);
                        let pos_display = truncate_text(&employee.position, 25);
                        let dept_display = truncate_text(&employee.department, 20);
                        
                        ui.label(RichText::new(&name_display).font(FontId::proportional(18.0)))
                            .on_hover_text(&employee.name);
                        ui.label(RichText::new(&pos_display).font(FontId::proportional(18.0)))
                            .on_hover_text(&employee.position);
                        ui.label(RichText::new(&dept_display).font(FontId::proportional(18.0)))
                            .on_hover_text(&employee.department);
                        ui.label(RichText::new(&employee.qr_code).font(FontId::proportional(18.0)));

                        let emp_id = employee.id.clone();
                        let emp_name = employee.name.clone();
                        let emp_pos = employee.position.clone();
                        let emp_dept = employee.department.clone();

                        ui.horizontal(|ui| {
                            if ui.button(RichText::new("Редакт.").font(FontId::proportional(16.0))).clicked() {
                                self.employee_name = emp_name.clone();
                                self.employee_position = emp_pos.clone();
                                self.employee_department = emp_dept.clone();
                                self.editing_employee_id = Some(emp_id.clone());
                            }

                            if ui.button(RichText::new("Удалить").font(FontId::proportional(16.0)).color(Color32::RED)).clicked() {
                                self.delete_employee_id = Some(emp_id);
                                self.delete_employee_name = emp_name;
                            }
                        });

                        ui.end_row();
                    }

                    if is_filtered_empty && !is_search_empty {
                        ui.label(RichText::new("Ничего не найдено").font(FontId::proportional(18.0)));
                    }
                });
        });

        // Delete employee confirmation dialog
        if let Some(emp_id) = &self.delete_employee_id.clone() {
            egui::Window::new(RichText::new("Подтверждение удаления").font(FontId::proportional(20.0)))
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ui.ctx(), |ui| {
                    ui.add_space(10.0);
                    ui.label(RichText::new("Вы уверены, что хотите удалить сотрудника?").font(FontId::proportional(18.0)));
                    ui.add_space(5.0);
                    ui.label(RichText::new(&self.delete_employee_name).font(FontId::proportional(18.0)).strong().color(Color32::from_rgb(231, 76, 60)));
                    ui.add_space(5.0);
                    ui.label(RichText::new("Это действие нельзя отменить!").font(FontId::proportional(16.0)).color(Color32::from_rgb(150, 150, 150)));
                    ui.add_space(20.0);
                    ui.horizontal(|ui| {
                        if ui.add_sized([120.0, 40.0], egui::Button::new(RichText::new("Удалить").font(FontId::proportional(18.0))).fill(Color32::from_rgb(231, 76, 60))).clicked() {
                            self.db.delete_employee(&emp_id);
                            self.delete_employee_id = None;
                            self.delete_employee_name.clear();
                        }
                        if ui.add_sized([120.0, 40.0], egui::Button::new(RichText::new("Отмена").font(FontId::proportional(18.0)))).clicked() {
                            self.delete_employee_id = None;
                            self.delete_employee_name.clear();
                        }
                    });
                });
        }
    }

    fn show_tools(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading(RichText::new("Инструменты").font(FontId::proportional(24.0)));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button(RichText::new("+ Добавить").font(FontId::proportional(18.0))).clicked() {
                    self.tool_name.clear();
                    self.tool_inventory.clear();
                    self.tool_category.clear();
                    self.tool_quantity_str = "1".to_string();
                    self.editing_tool_id = None;
                }
            });
        });

        ui.add_space(10.0);

        // Search field
        ui.horizontal(|ui| {
            ui.label(RichText::new("Поиск:").font(FontId::proportional(18.0)));
            ui.add(egui::TextEdit::singleline(&mut self.tool_search)
                .desired_width(300.0)
                .hint_text("Поиск по названию, инв. номеру, категории...")
                .font(FontId::proportional(18.0)));
            if ui.button(RichText::new("Очистить").font(FontId::proportional(16.0))).clicked() {
                self.tool_search.clear();
            }
        });

        ui.add_space(10.0);

        // Add/Edit form
        egui::CollapsingHeader::new(RichText::new(if self.editing_tool_id.is_some() { "Редактировать инструмент" } else { "Добавить инструмент" }).font(FontId::proportional(20.0)))
            .default_open(self.editing_tool_id.is_some())
            .show(ui, |ui| {
                egui::Grid::new("tool_form").num_columns(2).spacing([20.0, 10.0]).show(ui, |ui| {
                    ui.label(RichText::new("Название:").font(FontId::proportional(18.0)));
                    ui.add(egui::TextEdit::singleline(&mut self.tool_name).desired_width(300.0).font(FontId::proportional(18.0)));
                    ui.end_row();

                    ui.label(RichText::new("Инв. номер:").font(FontId::proportional(18.0)));
                    ui.add(egui::TextEdit::singleline(&mut self.tool_inventory).desired_width(300.0).font(FontId::proportional(18.0)));
                    ui.end_row();

                    ui.label(RichText::new("Категория:").font(FontId::proportional(18.0)));
                    let cat_name = self.db.get_category_name(&self.tool_category);
                    egui::ComboBox::from_label("")
                        .selected_text(if self.tool_category.is_empty() { "Выберите категорию" } else { &cat_name })
                        .show_ui(ui, |ui| {
                            for cat in &self.db.categories {
                                ui.selectable_value(&mut self.tool_category, cat.id.clone(), &cat.name);
                            }
                        });
                    ui.end_row();

                    ui.label(RichText::new("Количество:").font(FontId::proportional(18.0)));
                    ui.add(egui::TextEdit::singleline(&mut self.tool_quantity_str).desired_width(100.0).font(FontId::proportional(18.0)));
                    ui.end_row();
                });

                ui.add_space(15.0);
                ui.horizontal(|ui| {
                    if ui.button(RichText::new(if self.editing_tool_id.is_some() { "Сохранить" } else { "Добавить" }).font(FontId::proportional(18.0))).clicked() {
                        if !self.tool_name.is_empty() {
                            let quantity: i32 = self.tool_quantity_str.parse().unwrap_or(1).max(1);
                            let id = self.editing_tool_id.clone().unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
                            let qr_code = format!("TOOL_{}", &id[..8]);

                            let tool = Tool {
                                id: id.clone(),
                                name: self.tool_name.clone(),
                                inventory_number: self.tool_inventory.clone(),
                                category_id: self.tool_category.clone(),
                                total_quantity: quantity,
                                qr_code,
                            };

                            if self.editing_tool_id.is_some() {
                                self.db.update_tool(tool);
                            } else {
                                self.db.add_tool(tool);
                            }

                            self.tool_name.clear();
                            self.tool_inventory.clear();
                            self.tool_category.clear();
                            self.tool_quantity_str = "1".to_string();
                            self.editing_tool_id = None;
                        }
                    }

                    if self.editing_tool_id.is_some() && ui.button(RichText::new("Отмена").font(FontId::proportional(18.0))).clicked() {
                        self.tool_name.clear();
                        self.tool_inventory.clear();
                        self.tool_category.clear();
                        self.tool_quantity_str = "1".to_string();
                        self.editing_tool_id = None;
                    }
                });
            });

        // Write off dialog
        if let Some(tool_id) = &self.write_off_tool_id.clone() {
            let tool = self.db.tools.iter().find(|t| t.id == *tool_id).cloned();
            if let Some(tool) = tool {
                egui::Window::new(RichText::new("Списать инструмент").font(FontId::proportional(20.0)))
                    .collapsible(false)
                    .resizable(false)
                    .show(ui.ctx(), |ui| {
                        ui.label(RichText::new(format!("Инструмент: {}", tool.name)).font(FontId::proportional(18.0)));
                        let issued = self.db.get_issued_quantity(&tool.id);
                        let available = tool.total_quantity - issued;
                        ui.label(RichText::new(format!("Доступно для списания: {}", available)).font(FontId::proportional(18.0)));

                        ui.add_space(10.0);
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Количество:").font(FontId::proportional(18.0)));
                            ui.add(egui::TextEdit::singleline(&mut self.write_off_quantity_str).desired_width(80.0).font(FontId::proportional(18.0)));
                        });

                        ui.add_space(15.0);
                        ui.horizontal(|ui| {
                            if ui.button(RichText::new("Списать").font(FontId::proportional(18.0))).clicked() {
                                let qty: i32 = self.write_off_quantity_str.parse().unwrap_or(0);
                                if qty > 0 {
                                    match self.db.write_off_tool(tool_id, qty) {
                                        Ok(()) => {
                                            self.write_off_tool_id = None;
                                            self.write_off_quantity_str = "1".to_string();
                                        }
                                        Err(e) => {
                                            self.write_off_quantity_str = e;
                                        }
                                    }
                                }
                            }
                            if ui.button(RichText::new("Отмена").font(FontId::proportional(18.0))).clicked() {
                                self.write_off_tool_id = None;
                                self.write_off_quantity_str = "1".to_string();
                            }
                        });
                    });
            }
        }

        ui.add_space(20.0);

        // Filter tools by search - clone to avoid borrow issues
        let search_lower = self.tool_search.to_lowercase();
        let categories: std::collections::HashMap<String, String> = self.db.categories.iter()
            .map(|c| (c.id.clone(), c.name.clone()))
            .collect();
        
        let filtered_tools: Vec<Tool> = self.db.tools.iter()
            .filter(|t| {
                if search_lower.is_empty() {
                    true
                } else {
                    let category_name = categories.get(&t.category_id).cloned().unwrap_or_default();
                    t.name.to_lowercase().contains(&search_lower) ||
                    t.inventory_number.to_lowercase().contains(&search_lower) ||
                    category_name.to_lowercase().contains(&search_lower) ||
                    t.qr_code.to_lowercase().contains(&search_lower)
                }
            })
            .cloned()
            .collect();

        let is_search_empty = self.tool_search.is_empty();
        let is_filtered_empty = filtered_tools.is_empty();

        // Tools table with horizontal scroll
        egui::ScrollArea::horizontal().show(ui, |ui| {
            egui::Grid::new("tools_table")
                .min_col_width(100.0)
                .spacing([12.0, 10.0])
                .show(ui, |ui| {
                    ui.label(RichText::new("Название").font(FontId::proportional(18.0)).strong());
                    ui.label(RichText::new("Инв. номер").font(FontId::proportional(18.0)).strong());
                    ui.label(RichText::new("Категория").font(FontId::proportional(18.0)).strong());
                    ui.label(RichText::new("Всего").font(FontId::proportional(18.0)).strong());
                    ui.label(RichText::new("Выдано").font(FontId::proportional(18.0)).strong());
                    ui.label(RichText::new("Доступно").font(FontId::proportional(18.0)).strong());
                    ui.label(RichText::new("QR").font(FontId::proportional(18.0)).strong());
                    ui.label(RichText::new("Действия").font(FontId::proportional(18.0)).strong());
                    ui.end_row();

                    for tool in &filtered_tools {
                        let category_name = categories.get(&tool.category_id).cloned().unwrap_or_default();
                        let issued = self.db.get_issued_quantity(&tool.id);
                        let available = tool.total_quantity - issued;

                        let name_display = truncate_text(&tool.name, 35);
                        let inv_display = truncate_text(&tool.inventory_number, 15);
                        let cat_display = truncate_text(&category_name, 20);

                        ui.label(RichText::new(&name_display).font(FontId::proportional(18.0)))
                            .on_hover_text(&tool.name);
                        ui.label(RichText::new(&inv_display).font(FontId::proportional(18.0)))
                            .on_hover_text(&tool.inventory_number);
                        ui.label(RichText::new(&cat_display).font(FontId::proportional(18.0)))
                            .on_hover_text(&category_name);
                        ui.label(RichText::new(tool.total_quantity.to_string()).font(FontId::proportional(18.0)));
                        ui.label(RichText::new(issued.to_string()).font(FontId::proportional(18.0)).color(Color32::from_rgb(231, 76, 60)));
                        ui.label(RichText::new(available.to_string()).font(FontId::proportional(18.0)).color(Color32::from_rgb(46, 204, 113)));
                        ui.label(RichText::new(&tool.qr_code).font(FontId::proportional(16.0)));

                        let tool_id = tool.id.clone();
                        let tool_name = tool.name.clone();
                        let tool_inv = tool.inventory_number.clone();
                        let tool_cat = tool.category_id.clone();
                        let tool_qty = tool.total_quantity;

                        ui.horizontal(|ui| {
                            if ui.button(RichText::new("Редакт.").font(FontId::proportional(14.0))).clicked() {
                                self.tool_name = tool_name;
                                self.tool_inventory = tool_inv;
                                self.tool_category = tool_cat;
                                self.tool_quantity_str = tool_qty.to_string();
                                self.editing_tool_id = Some(tool_id.clone());
                            }

                            if ui.button(RichText::new("Удалить").font(FontId::proportional(14.0)).color(Color32::RED)).clicked() {
                                self.delete_tool_id = Some(tool_id);
                                self.delete_tool_name = tool_name;
                            }

                            if ui.button(RichText::new("Списать").font(FontId::proportional(14.0))).clicked() {
                                self.write_off_tool_id = Some(tool_id);
                                self.write_off_quantity_str = "1".to_string();
                            }
                        });

                        ui.end_row();
                    }

                    if is_filtered_empty && !is_search_empty {
                        ui.label(RichText::new("Ничего не найдено").font(FontId::proportional(18.0)));
                    }
                });
        });

        // Delete tool confirmation dialog
        if let Some(tool_id) = &self.delete_tool_id.clone() {
            egui::Window::new(RichText::new("Подтверждение удаления").font(FontId::proportional(20.0)))
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ui.ctx(), |ui| {
                    ui.add_space(10.0);
                    ui.label(RichText::new("Вы уверены, что хотите удалить инструмент?").font(FontId::proportional(18.0)));
                    ui.add_space(5.0);
                    ui.label(RichText::new(&self.delete_tool_name).font(FontId::proportional(18.0)).strong().color(Color32::from_rgb(231, 76, 60)));
                    ui.add_space(5.0);
                    ui.label(RichText::new("Это действие нельзя отменить!").font(FontId::proportional(16.0)).color(Color32::from_rgb(150, 150, 150)));
                    ui.add_space(20.0);
                    ui.horizontal(|ui| {
                        if ui.add_sized([120.0, 40.0], egui::Button::new(RichText::new("Удалить").font(FontId::proportional(18.0))).fill(Color32::from_rgb(231, 76, 60))).clicked() {
                            self.db.delete_tool(&tool_id);
                            self.delete_tool_id = None;
                            self.delete_tool_name.clear();
                        }
                        if ui.add_sized([120.0, 40.0], egui::Button::new(RichText::new("Отмена").font(FontId::proportional(18.0)))).clicked() {
                            self.delete_tool_id = None;
                            self.delete_tool_name.clear();
                        }
                    });
                });
        }
    }

    fn show_issuances(&mut self, ui: &mut egui::Ui) {
        ui.heading(RichText::new("Выдачи").font(FontId::proportional(24.0)));

        ui.add_space(10.0);

        // Search field
        ui.horizontal(|ui| {
            ui.label(RichText::new("Поиск:").font(FontId::proportional(18.0)));
            ui.add(egui::TextEdit::singleline(&mut self.issuance_search)
                .desired_width(300.0)
                .hint_text("Поиск по инструменту или сотруднику...")
                .font(FontId::proportional(18.0)));
            if ui.button(RichText::new("Очистить").font(FontId::proportional(16.0))).clicked() {
                self.issuance_search.clear();
            }
        });

        ui.add_space(10.0);

        let active: Vec<_> = self.db.issuances.iter()
            .filter(|i| i.returned_at.is_none())
            .collect();

        ui.label(RichText::new(format!("Активных выдач: {}", active.len())).font(FontId::proportional(18.0)));

        ui.add_space(10.0);

        // Filter issuances by search - clone data
        let search_lower = self.issuance_search.to_lowercase();
        
        // Build lookup maps
        let tool_names: std::collections::HashMap<String, String> = self.db.tools.iter()
            .map(|t| (t.id.clone(), t.name.clone())).collect();
        let tool_inventories: std::collections::HashMap<String, String> = self.db.tools.iter()
            .map(|t| (t.id.clone(), t.inventory_number.clone())).collect();
        let employee_names: std::collections::HashMap<String, String> = self.db.employees.iter()
            .map(|e| (e.id.clone(), e.name.clone())).collect();

        let filtered_issuances: Vec<Issuance> = self.db.issuances.iter()
            .filter(|i| {
                if search_lower.is_empty() {
                    true
                } else {
                    let tool_name = tool_names.get(&i.tool_id).cloned().unwrap_or_default();
                    let employee_name = employee_names.get(&i.employee_id).cloned().unwrap_or_default();
                    let inventory = tool_inventories.get(&i.tool_id).cloned().unwrap_or_default();

                    tool_name.to_lowercase().contains(&search_lower) ||
                    employee_name.to_lowercase().contains(&search_lower) ||
                    inventory.to_lowercase().contains(&search_lower)
                }
            })
            .cloned()
            .collect();

        let is_search_empty = self.issuance_search.is_empty();
        let is_filtered_empty = filtered_issuances.is_empty();

        // Issuances table with horizontal scroll
        egui::ScrollArea::horizontal().show(ui, |ui| {
            egui::Grid::new("issuances_table")
                .min_col_width(120.0)
                .spacing([15.0, 10.0])
                .show(ui, |ui| {
                    ui.label(RichText::new("Инструмент").font(FontId::proportional(18.0)).strong());
                    ui.label(RichText::new("Сотрудник").font(FontId::proportional(18.0)).strong());
                    ui.label(RichText::new("Кол-во").font(FontId::proportional(18.0)).strong());
                    ui.label(RichText::new("Выдано").font(FontId::proportional(18.0)).strong());
                    ui.label(RichText::new("Статус").font(FontId::proportional(18.0)).strong());
                    ui.label(RichText::new("Действие").font(FontId::proportional(18.0)).strong());
                    ui.end_row();

                    for issuance in &filtered_issuances {
                        let tool_name = tool_names.get(&issuance.tool_id).cloned().unwrap_or_else(|| "Неизвестно".to_string());
                        let employee_name = employee_names.get(&issuance.employee_id).cloned().unwrap_or_else(|| "Неизвестно".to_string());

                        let tool_display = truncate_text(&tool_name, 35);
                        let emp_display = truncate_text(&employee_name, 30);

                        ui.label(RichText::new(&tool_display).font(FontId::proportional(18.0)))
                            .on_hover_text(&tool_name);
                        ui.label(RichText::new(&emp_display).font(FontId::proportional(18.0)))
                            .on_hover_text(&employee_name);
                        ui.label(RichText::new(issuance.quantity.to_string()).font(FontId::proportional(18.0)));
                        ui.label(RichText::new(issuance.issued_at.format("%d.%m.%Y %H:%M").to_string()).font(FontId::proportional(18.0)));

                        if issuance.returned_at.is_none() {
                            ui.label(RichText::new("Выдано").font(FontId::proportional(18.0)).color(Color32::from_rgb(231, 76, 60)));

                            let issuance_id = issuance.id.clone();
                            if ui.button(RichText::new("Вернуть").font(FontId::proportional(18.0))).clicked() {
                                self.db.return_tool(&issuance_id);
                            }
                        } else {
                            ui.label(RichText::new("Возвращено").font(FontId::proportional(18.0)).color(Color32::from_rgb(46, 204, 113)));
                            ui.label(RichText::new(issuance.returned_at.unwrap().format("%d.%m.%Y %H:%M").to_string()).font(FontId::proportional(18.0)));
                        }

                        ui.end_row();
                    }

                    if is_filtered_empty && !is_search_empty {
                        ui.label(RichText::new("Ничего не найдено").font(FontId::proportional(18.0)));
                    }
                });
        });
    }

    fn show_terminal(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("terminal_header").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading(RichText::new("Терминал выдачи").font(FontId::proportional(24.0)));

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(RichText::new("Выход").font(FontId::proportional(18.0))).clicked() {
                        self.current_user = None;
                        self.screen = Screen::Login;
                        let (saved_user, saved_pass) = Database::load_config();
                        self.login_username = saved_user;
                        self.login_password = saved_pass;
                        self.scanned_employee = None;
                        self.scanned_tool = None;
                        self.scan_input.clear();
                    }
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(15.0);

            // Scan input
            ui.horizontal(|ui| {
                ui.label(RichText::new("Сканировать:").font(FontId::proportional(20.0)));
                let response = ui.add(egui::TextEdit::singleline(&mut self.scan_input)
                    .desired_width(400.0)
                    .hint_text("Отсканируйте QR код...")
                    .font(FontId::proportional(20.0)));

                if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    self.process_scan();
                    self.scan_input.clear();
                }
            });

            ui.add_space(10.0);

            if !self.terminal_error.is_empty() {
                ui.colored_label(Color32::RED, RichText::new(&self.terminal_error).font(FontId::proportional(18.0)));
            }

            if !self.terminal_message.is_empty() {
                ui.colored_label(Color32::from_rgb(46, 204, 113), RichText::new(&self.terminal_message).font(FontId::proportional(18.0)));
            }

            ui.add_space(25.0);

            // Scanned items
            egui::Grid::new("scanned_items").num_columns(2).spacing([40.0, 10.0]).show(ui, |ui| {
                // Employee
                ui.group(|ui| {
                    ui.set_min_size(Vec2::new(400.0, 250.0));
                    ui.vertical(|ui| {
                        ui.heading(RichText::new("Сотрудник").font(FontId::proportional(22.0)));

                        if let Some(emp) = &self.scanned_employee {
                            ui.label(RichText::new(&emp.name).font(FontId::proportional(22.0)).strong());
                            ui.label(RichText::new(format!("Должность: {}", emp.position)).font(FontId::proportional(18.0)));
                            ui.label(RichText::new(format!("Отдел: {}", emp.department)).font(FontId::proportional(18.0)));

                            let issued = self.db.get_employee_active_issuances(&emp.id);
                            ui.add_space(10.0);
                            ui.label(RichText::new(format!("Выдано инструментов: {}", issued.len())).font(FontId::proportional(18.0)));
                        } else {
                            ui.label(RichText::new("Отсканируйте QR код сотрудника").font(FontId::proportional(18.0)));
                        }
                    });
                });

                // Tool
                ui.group(|ui| {
                    ui.set_min_size(Vec2::new(400.0, 250.0));
                    ui.vertical(|ui| {
                        ui.heading(RichText::new("Инструмент").font(FontId::proportional(22.0)));

                        if let Some(tool) = &self.scanned_tool {
                            ui.label(RichText::new(&tool.name).font(FontId::proportional(22.0)).strong());
                            ui.label(RichText::new(format!("Инв. номер: {}", tool.inventory_number)).font(FontId::proportional(18.0)));

                            let issued = self.db.get_issued_quantity(&tool.id);
                            let available = tool.total_quantity - issued;

                            ui.add_space(10.0);
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(format!("Всего: {}", tool.total_quantity)).font(FontId::proportional(18.0)));
                                ui.label(RichText::new(format!("Выдано: {}", issued)).font(FontId::proportional(18.0)).color(Color32::from_rgb(231, 76, 60)));
                                ui.label(RichText::new(format!("Доступно: {}", available)).font(FontId::proportional(18.0)).color(Color32::from_rgb(46, 204, 113)));
                            });

                            if available > 0 && self.scanned_employee.is_some() {
                                ui.add_space(15.0);
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new("Количество:").font(FontId::proportional(18.0)));
                                    if ui.button(RichText::new(" - ").font(FontId::proportional(20.0))).clicked() && self.issue_quantity > 1 {
                                        self.issue_quantity -= 1;
                                    }
                                    ui.label(RichText::new(self.issue_quantity.to_string()).font(FontId::proportional(20.0)));
                                    if ui.button(RichText::new(" + ").font(FontId::proportional(20.0))).clicked() && self.issue_quantity < available {
                                        self.issue_quantity += 1;
                                    }
                                });

                                ui.add_space(15.0);
                                if ui.button(RichText::new("Выдать").font(FontId::proportional(20.0))).clicked() {
                                    self.issue_tool();
                                }
                            }
                        } else {
                            ui.label(RichText::new("Отсканируйте QR код инструмента").font(FontId::proportional(18.0)));
                        }
                    });
                });
            });

            ui.add_space(25.0);

            // Active issuances for scanned employee
            if let Some(emp) = &self.scanned_employee {
                ui.heading(RichText::new("Выданные инструменты:").font(FontId::proportional(22.0)));
                ui.add_space(10.0);

                let issuances: Vec<_> = self.db.get_employee_active_issuances(&emp.id).into_iter().cloned().collect();

                if issuances.is_empty() {
                    ui.label(RichText::new("Нет выданных инструментов").font(FontId::proportional(18.0)));
                } else {
                    egui::Grid::new("employee_issuances").spacing([20.0, 10.0]).show(ui, |ui| {
                        ui.label(RichText::new("Инструмент").font(FontId::proportional(18.0)).strong());
                        ui.label(RichText::new("Кол-во").font(FontId::proportional(18.0)).strong());
                        ui.label(RichText::new("Выдано").font(FontId::proportional(18.0)).strong());
                        ui.label(RichText::new("").font(FontId::proportional(18.0)));
                        ui.end_row();

                        for issuance in &issuances {
                            let tool = self.db.tools.iter().find(|t| t.id == issuance.tool_id);
                            ui.label(RichText::new(tool.map(|t| t.name.as_str()).unwrap_or("Неизвестно")).font(FontId::proportional(18.0)));
                            ui.label(RichText::new(issuance.quantity.to_string()).font(FontId::proportional(18.0)));
                            ui.label(RichText::new(issuance.issued_at.format("%d.%m.%Y %H:%M").to_string()).font(FontId::proportional(18.0)));

                            let issuance_id = issuance.id.clone();
                            if ui.button(RichText::new("Вернуть").font(FontId::proportional(18.0))).clicked() {
                                self.db.return_tool(&issuance_id);
                                self.terminal_message = "Инструмент возвращен".to_string();
                            }
                            ui.end_row();
                        }
                    });
                }
            }
        });
    }

    fn process_scan(&mut self) {
        self.terminal_error.clear();
        self.terminal_message.clear();

        let code = self.scan_input.trim();

        if code.starts_with("EMP_") {
            if let Some(emp) = self.db.get_employee_by_qr(code) {
                self.scanned_employee = Some(emp.clone());
                self.terminal_message = format!("Сотрудник: {}", emp.name);
            } else {
                self.terminal_error = "Сотрудник не найден".to_string();
            }
        } else if code.starts_with("TOOL_") {
            if let Some(tool) = self.db.get_tool_by_qr(code) {
                self.scanned_tool = Some(tool.clone());
                self.terminal_message = format!("Инструмент: {}", tool.name);
            } else {
                self.terminal_error = "Инструмент не найден".to_string();
            }
        } else {
            self.terminal_error = "Неверный формат QR кода".to_string();
        }
    }

    fn issue_tool(&mut self) {
        self.terminal_error.clear();
        self.terminal_message.clear();

        if let (Some(emp), Some(tool)) = (&self.scanned_employee, &self.scanned_tool) {
            let issuance = Issuance {
                id: uuid::Uuid::new_v4().to_string(),
                tool_id: tool.id.clone(),
                employee_id: emp.id.clone(),
                user_id: self.current_user.as_ref().unwrap().id.clone(),
                quantity: self.issue_quantity,
                issued_at: Local::now(),
                returned_at: None,
            };

            match self.db.issue_tool(issuance) {
                Ok(()) => {
                    self.terminal_message = format!("Выдано {} x{}", tool.name, self.issue_quantity);
                    self.scanned_tool = None;
                    self.issue_quantity = 1;
                }
                Err(e) => {
                    self.terminal_error = e;
                }
            }
        }
    }
}
