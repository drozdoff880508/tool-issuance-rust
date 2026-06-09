use crate::data::{Database, Employee, Issuance, Role, Tool, User};
use chrono::Local;
use eframe::egui;
use egui::{Color32, FontId, RichText, Vec2};

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

    // Employee form
    employee_name: String,
    employee_position: String,
    employee_department: String,
    editing_employee_id: Option<String>,

    // Tool form
    tool_name: String,
    tool_inventory: String,
    tool_category: String,
    tool_quantity: i32,
    editing_tool_id: Option<String>,

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

        Self {
            db,
            current_user: None,
            screen: Screen::Login,
            login_username: String::new(),
            login_password: String::new(),
            login_error: String::new(),
            employee_name: String::new(),
            employee_position: String::new(),
            employee_department: String::new(),
            editing_employee_id: None,
            tool_name: String::new(),
            tool_inventory: String::new(),
            tool_category: String::new(),
            tool_quantity: 1,
            editing_tool_id: None,
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
                ui.add_space(100.0);
                ui.heading(RichText::new("Система выдачи инструментов").font(FontId::proportional(28.0)));
                ui.add_space(40.0);

                egui::Frame::none()
                    .fill(Color32::from_rgb(240, 240, 240))
                    .inner_margin(20.0)
                    .corner_radius(10.0)
                    .show(ui, |ui| {
                        ui.set_min_width(300.0);

                        ui.label("Логин:");
                        ui.add(egui::TextEdit::singleline(&mut self.login_username).desired_width(250.0));

                        ui.add_space(10.0);
                        ui.label("Пароль:");
                        ui.add(egui::TextEdit::singleline(&mut self.login_password).password(true).desired_width(250.0));

                        if !self.login_error.is_empty() {
                            ui.add_space(10.0);
                            ui.colored_label(Color32::RED, &self.login_error);
                        }

                        ui.add_space(20.0);
                        if ui.button(RichText::new("Войти").font(FontId::proportional(16.0))).clicked() {
                            if let Some(user) = self.db.authenticate(&self.login_username, &self.login_password) {
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
                ui.heading("Панель администратора");
                ui.add_space(20.0);

                if ui.selectable_label(tab == AdminTab::Overview, "Обзор").clicked() {
                    self.screen = Screen::Admin(AdminTab::Overview);
                }
                if ui.selectable_label(tab == AdminTab::Employees, "Сотрудники").clicked() {
                    self.screen = Screen::Admin(AdminTab::Employees);
                }
                if ui.selectable_label(tab == AdminTab::Tools, "Инструменты").clicked() {
                    self.screen = Screen::Admin(AdminTab::Tools);
                }
                if ui.selectable_label(tab == AdminTab::Issuances, "Выдачи").clicked() {
                    self.screen = Screen::Admin(AdminTab::Issuances);
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Выход").clicked() {
                        self.current_user = None;
                        self.screen = Screen::Login;
                        self.login_username.clear();
                        self.login_password.clear();
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
        ui.heading("Обзор");
        ui.add_space(20.0);

        let stats = self.db.get_stats();

        egui::Grid::new("stats_grid").num_columns(3).spacing([20.0, 10.0]).show(ui, |ui| {
            ui.group(|ui| {
                ui.set_min_size(Vec2::new(200.0, 100.0));
                ui.vertical_centered(|ui| {
                    ui.add_space(10.0);
                    ui.label(RichText::new("Сотрудников").font(FontId::proportional(14.0)));
                    ui.label(RichText::new(stats.get("total_employees").unwrap_or(&0).to_string())
                        .font(FontId::proportional(32.0)).color(Color32::from_rgb(52, 152, 219)));
                });
            });

            ui.group(|ui| {
                ui.set_min_size(Vec2::new(200.0, 100.0));
                ui.vertical_centered(|ui| {
                    ui.add_space(10.0);
                    ui.label(RichText::new("Видов инструментов").font(FontId::proportional(14.0)));
                    ui.label(RichText::new(stats.get("total_tools").unwrap_or(&0).to_string())
                        .font(FontId::proportional(32.0)).color(Color32::from_rgb(46, 204, 113)));
                });
            });

            ui.group(|ui| {
                ui.set_min_size(Vec2::new(200.0, 100.0));
                ui.vertical_centered(|ui| {
                    ui.add_space(10.0);
                    ui.label(RichText::new("Всего единиц").font(FontId::proportional(14.0)));
                    ui.label(RichText::new(stats.get("total_tool_quantity").unwrap_or(&0).to_string())
                        .font(FontId::proportional(32.0)).color(Color32::from_rgb(155, 89, 182)));
                });
            });

            ui.end_row();

            ui.group(|ui| {
                ui.set_min_size(Vec2::new(200.0, 100.0));
                ui.vertical_centered(|ui| {
                    ui.add_space(10.0);
                    ui.label(RichText::new("Выдано единиц").font(FontId::proportional(14.0)));
                    ui.label(RichText::new(stats.get("issued_quantity").unwrap_or(&0).to_string())
                        .font(FontId::proportional(32.0)).color(Color32::from_rgb(231, 76, 60)));
                });
            });

            ui.group(|ui| {
                ui.set_min_size(Vec2::new(200.0, 100.0));
                ui.vertical_centered(|ui| {
                    ui.add_space(10.0);
                    ui.label(RichText::new("Активных выдач").font(FontId::proportional(14.0)));
                    ui.label(RichText::new(stats.get("active_issuances").unwrap_or(&0).to_string())
                        .font(FontId::proportional(32.0)).color(Color32::from_rgb(230, 126, 34)));
                });
            });
        });

        ui.add_space(30.0);
        ui.heading("Последние выдачи");
        ui.add_space(10.0);

        egui::ScrollArea::vertical().max_height(300.0).show(ui, |ui| {

            let active: Vec<_> = self.db.issuances.iter()
                .filter(|i| i.returned_at.is_none())
                .collect();

            if active.is_empty() {
                ui.label("Нет активных выдач");
            } else {
                egui::Grid::new("recent_issuances").show(ui, |ui| {
                    ui.label(RichText::new("Инструмент").strong());
                    ui.label(RichText::new("Сотрудник").strong());
                    ui.label(RichText::new("Кол-во").strong());
                    ui.label(RichText::new("Дата выдачи").strong());
                    ui.end_row();

                    for issuance in active.iter().take(20) {
                        let tool = self.db.tools.iter().find(|t| t.id == issuance.tool_id);
                        let employee = self.db.employees.iter().find(|e| e.id == issuance.employee_id);

                        ui.label(tool.map(|t| t.name.as_str()).unwrap_or("Неизвестно"));
                        ui.label(employee.map(|e| e.name.as_str()).unwrap_or("Неизвестно"));
                        ui.label(issuance.quantity.to_string());
                        ui.label(issuance.issued_at.format("%d.%m.%Y %H:%M").to_string());
                        ui.end_row();
                    }
                });
            }
        });
    }

    fn show_employees(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading("Сотрудники");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("+ Добавить").clicked() {
                    self.employee_name.clear();
                    self.employee_position.clear();
                    self.employee_department.clear();
                    self.editing_employee_id = None;
                }
            });
        });

        ui.add_space(10.0);

        // Add/Edit form
        egui::CollapsingHeader::new(if self.editing_employee_id.is_some() { "Редактировать сотрудника" } else { "Добавить сотрудника" })
            .default_open(self.editing_employee_id.is_some())
            .show(ui, |ui| {
                egui::Grid::new("employee_form").num_columns(2).show(ui, |ui| {
                    ui.label("ФИО:");
                    ui.add(egui::TextEdit::singleline(&mut self.employee_name).desired_width(200.0));
                    ui.end_row();

                    ui.label("Должность:");
                    ui.add(egui::TextEdit::singleline(&mut self.employee_position).desired_width(200.0));
                    ui.end_row();

                    ui.label("Отдел:");
                    ui.add(egui::TextEdit::singleline(&mut self.employee_department).desired_width(200.0));
                    ui.end_row();
                });

                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    if ui.button(if self.editing_employee_id.is_some() { "Сохранить" } else { "Добавить" }).clicked() {
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

                    if self.editing_employee_id.is_some() && ui.button("Отмена").clicked() {
                        self.employee_name.clear();
                        self.employee_position.clear();
                        self.employee_department.clear();
                        self.editing_employee_id = None;
                    }
                });
            });

        ui.add_space(20.0);

        // Employees table
        egui::ScrollArea::vertical().max_height(400.0).show(ui, |ui| {
            egui::Grid::new("employees_table").show(ui, |ui| {
                ui.label(RichText::new("ФИО").strong());
                ui.label(RichText::new("Должность").strong());
                ui.label(RichText::new("Отдел").strong());
                ui.label(RichText::new("QR код").strong());
                ui.label(RichText::new("Действия").strong());
                ui.end_row();

                for employee in self.db.employees.clone() {
                    ui.label(&employee.name);
                    ui.label(&employee.position);
                    ui.label(&employee.department);
                    ui.label(&employee.qr_code);

                    ui.horizontal(|ui| {
                        if ui.button("✏️").clicked() {
                            self.employee_name = employee.name.clone();
                            self.employee_position = employee.position.clone();
                            self.employee_department = employee.department.clone();
                            self.editing_employee_id = Some(employee.id.clone());
                        }

                        if ui.button("🗑️").clicked() {
                            self.db.delete_employee(&employee.id);
                        }
                    });

                    ui.end_row();
                }
            });
        });
    }

    fn show_tools(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading("Инструменты");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("+ Добавить").clicked() {
                    self.tool_name.clear();
                    self.tool_inventory.clear();
                    self.tool_category.clear();
                    self.tool_quantity = 1;
                    self.editing_tool_id = None;
                }
            });
        });

        ui.add_space(10.0);

        // Add/Edit form
        egui::CollapsingHeader::new(if self.editing_tool_id.is_some() { "Редактировать инструмент" } else { "Добавить инструмент" })
            .default_open(self.editing_tool_id.is_some())
            .show(ui, |ui| {
                egui::Grid::new("tool_form").num_columns(2).show(ui, |ui| {
                    ui.label("Название:");
                    ui.add(egui::TextEdit::singleline(&mut self.tool_name).desired_width(200.0));
                    ui.end_row();

                    ui.label("Инв. номер:");
                    ui.add(egui::TextEdit::singleline(&mut self.tool_inventory).desired_width(200.0));
                    ui.end_row();

                    ui.label("Категория:");
                    let cat_name = self.db.get_category_name(&self.tool_category);
                    egui::ComboBox::from_label("")
                        .selected_text(if self.tool_category.is_empty() { "Выберите категорию" } else { &cat_name })
                        .show_ui(ui, |ui| {
                            for cat in &self.db.categories {
                                ui.selectable_value(&mut self.tool_category, cat.id.clone(), &cat.name);
                            }
                        });
                    ui.end_row();

                    ui.label("Количество:");
                    ui.add(egui::Slider::new(&mut self.tool_quantity, 1..=1000));
                    ui.end_row();
                });

                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    if ui.button(if self.editing_tool_id.is_some() { "Сохранить" } else { "Добавить" }).clicked() {
                        if !self.tool_name.is_empty() {
                            let id = self.editing_tool_id.clone().unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
                            let qr_code = format!("TOOL_{}", &id[..8]);

                            let tool = Tool {
                                id: id.clone(),
                                name: self.tool_name.clone(),
                                inventory_number: self.tool_inventory.clone(),
                                category_id: self.tool_category.clone(),
                                total_quantity: self.tool_quantity,
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
                            self.tool_quantity = 1;
                            self.editing_tool_id = None;
                        }
                    }

                    if self.editing_tool_id.is_some() && ui.button("Отмена").clicked() {
                        self.tool_name.clear();
                        self.tool_inventory.clear();
                        self.tool_category.clear();
                        self.tool_quantity = 1;
                        self.editing_tool_id = None;
                    }
                });
            });

        ui.add_space(20.0);

        // Tools table
        egui::ScrollArea::vertical().max_height(400.0).show(ui, |ui| {
            egui::Grid::new("tools_table").show(ui, |ui| {
                ui.label(RichText::new("Название").strong());
                ui.label(RichText::new("Инв. номер").strong());
                ui.label(RichText::new("Категория").strong());
                ui.label(RichText::new("Всего").strong());
                ui.label(RichText::new("Выдано").strong());
                ui.label(RichText::new("Доступно").strong());
                ui.label(RichText::new("QR").strong());
                ui.label(RichText::new("Действия").strong());
                ui.end_row();

                for tool in self.db.tools.clone() {
                    ui.label(&tool.name);
                    ui.label(&tool.inventory_number);
                    ui.label(self.db.get_category_name(&tool.category_id));

                    let issued = self.db.get_issued_quantity(&tool.id);
                    let available = tool.total_quantity - issued;

                    ui.label(tool.total_quantity.to_string());
                    ui.label(RichText::new(issued.to_string()).color(Color32::from_rgb(231, 76, 60)));
                    ui.label(RichText::new(available.to_string()).color(Color32::from_rgb(46, 204, 113)));

                    ui.label(&tool.qr_code);

                    ui.horizontal(|ui| {
                        if ui.button("✏️").clicked() {
                            self.tool_name = tool.name.clone();
                            self.tool_inventory = tool.inventory_number.clone();
                            self.tool_category = tool.category_id.clone();
                            self.tool_quantity = tool.total_quantity;
                            self.editing_tool_id = Some(tool.id.clone());
                        }

                        if ui.button("🗑️").clicked() {
                            self.db.delete_tool(&tool.id);
                        }
                    });

                    ui.end_row();
                }
            });
        });
    }

    fn show_issuances(&mut self, ui: &mut egui::Ui) {
        ui.heading("Выдачи");

        ui.add_space(10.0);

        let active: Vec<_> = self.db.issuances.iter()
            .filter(|i| i.returned_at.is_none())
            .collect();

        ui.label(format!("Активных выдач: {}", active.len()));

        ui.add_space(10.0);

        egui::ScrollArea::vertical().max_height(500.0).show(ui, |ui| {
            egui::Grid::new("issuances_table").show(ui, |ui| {
                ui.label(RichText::new("Инструмент").strong());
                ui.label(RichText::new("Сотрудник").strong());
                ui.label(RichText::new("Кол-во").strong());
                ui.label(RichText::new("Выдано").strong());
                ui.label(RichText::new("Статус").strong());
                ui.label(RichText::new("Действие").strong());
                ui.end_row();

                let issuances: Vec<_> = self.db.issuances.clone();
                for issuance in issuances.iter() {
                    let tool = self.db.tools.iter().find(|t| t.id == issuance.tool_id);
                    let employee = self.db.employees.iter().find(|e| e.id == issuance.employee_id);

                    ui.label(tool.map(|t| t.name.as_str()).unwrap_or("Неизвестно"));
                    ui.label(employee.map(|e| e.name.as_str()).unwrap_or("Неизвестно"));
                    ui.label(issuance.quantity.to_string());
                    ui.label(issuance.issued_at.format("%d.%m.%Y %H:%M").to_string());

                    if issuance.returned_at.is_none() {
                        ui.label(RichText::new("Выдано").color(Color32::from_rgb(231, 76, 60)));

                        let issuance_id = issuance.id.clone();
                        if ui.button("Вернуть").clicked() {
                            self.db.return_tool(&issuance_id);
                        }
                    } else {
                        ui.label(RichText::new("Возвращено").color(Color32::from_rgb(46, 204, 113)));
                        ui.label(issuance.returned_at.unwrap().format("%d.%m.%Y %H:%M").to_string());
                    }

                    ui.end_row();
                }
            });
        });
    }

    fn show_terminal(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("terminal_header").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Терминал выдачи");

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Выход").clicked() {
                        self.current_user = None;
                        self.screen = Screen::Login;
                        self.login_username.clear();
                        self.login_password.clear();
                        self.scanned_employee = None;
                        self.scanned_tool = None;
                        self.scan_input.clear();
                    }
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(10.0);

            // Scan input
            ui.horizontal(|ui| {
                ui.label("Сканировать:");
                let response = ui.add(egui::TextEdit::singleline(&mut self.scan_input).desired_width(300.0).hint_text("Отсканируйте QR код..."));

                if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    self.process_scan();
                    self.scan_input.clear();
                }
            });

            ui.add_space(10.0);

            if !self.terminal_error.is_empty() {
                ui.colored_label(Color32::RED, &self.terminal_error);
            }

            if !self.terminal_message.is_empty() {
                ui.colored_label(Color32::from_rgb(46, 204, 113), &self.terminal_message);
            }

            ui.add_space(20.0);

            // Scanned items
            egui::Grid::new("scanned_items").num_columns(2).spacing([40.0, 10.0]).show(ui, |ui| {
                // Employee
                ui.group(|ui| {
                    ui.set_min_size(Vec2::new(350.0, 200.0));
                    ui.vertical(|ui| {
                        ui.heading("Сотрудник");

                        if let Some(emp) = &self.scanned_employee {
                            ui.label(RichText::new(&emp.name).font(FontId::proportional(18.0)).strong());
                            ui.label(format!("Должность: {}", emp.position));
                            ui.label(format!("Отдел: {}", emp.department));

                            let issued = self.db.get_employee_active_issuances(&emp.id);
                            ui.add_space(10.0);
                            ui.label(format!("Выдано инструментов: {}", issued.len()));
                        } else {
                            ui.label("Отсканируйте QR код сотрудника");
                        }
                    });
                });

                // Tool
                ui.group(|ui| {
                    ui.set_min_size(Vec2::new(350.0, 200.0));
                    ui.vertical(|ui| {
                        ui.heading("Инструмент");

                        if let Some(tool) = &self.scanned_tool {
                            ui.label(RichText::new(&tool.name).font(FontId::proportional(18.0)).strong());
                            ui.label(format!("Инв. номер: {}", tool.inventory_number));

                            let issued = self.db.get_issued_quantity(&tool.id);
                            let available = tool.total_quantity - issued;

                            ui.add_space(10.0);
                            ui.horizontal(|ui| {
                                ui.label(format!("Всего: {}", tool.total_quantity));
                                ui.label(RichText::new(format!("Выдано: {}", issued)).color(Color32::from_rgb(231, 76, 60)));
                                ui.label(RichText::new(format!("Доступно: {}", available)).color(Color32::from_rgb(46, 204, 113)));
                            });

                            if available > 0 && self.scanned_employee.is_some() {
                                ui.add_space(10.0);
                                ui.horizontal(|ui| {
                                    ui.label("Количество:");
                                    if ui.button("-").clicked() && self.issue_quantity > 1 {
                                        self.issue_quantity -= 1;
                                    }
                                    ui.label(self.issue_quantity.to_string());
                                    if ui.button("+").clicked() && self.issue_quantity < available {
                                        self.issue_quantity += 1;
                                    }
                                });

                                ui.add_space(10.0);
                                if ui.button(RichText::new("Выдать").font(FontId::proportional(16.0))).clicked() {
                                    self.issue_tool();
                                }
                            }
                        } else {
                            ui.label("Отсканируйте QR код инструмента");
                        }
                    });
                });
            });

            ui.add_space(20.0);

            // Active issuances for scanned employee
            if let Some(emp) = &self.scanned_employee {
                ui.heading("Выданные инструменты:");
                ui.add_space(10.0);

                let issuances: Vec<_> = self.db.get_employee_active_issuances(&emp.id).into_iter().cloned().collect();

                if issuances.is_empty() {
                    ui.label("Нет выданных инструментов");
                } else {
                    egui::Grid::new("employee_issuances").show(ui, |ui| {
                        ui.label(RichText::new("Инструмент").strong());
                        ui.label(RichText::new("Кол-во").strong());
                        ui.label(RichText::new("Выдано").strong());
                        ui.label(RichText::new("").strong());
                        ui.end_row();

                        for issuance in &issuances {
                            let tool = self.db.tools.iter().find(|t| t.id == issuance.tool_id);
                            ui.label(tool.map(|t| t.name.as_str()).unwrap_or("Неизвестно"));
                            ui.label(issuance.quantity.to_string());
                            ui.label(issuance.issued_at.format("%d.%m.%Y %H:%M").to_string());

                            let issuance_id = issuance.id.clone();
                            if ui.button("Вернуть").clicked() {
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
                let issued = self.db.get_issued_quantity(&tool.id);
                let available = tool.total_quantity - issued;
                self.issue_quantity = 1.min(available);
                self.terminal_message = format!("Инструмент: {}", tool.name);
            } else {
                self.terminal_error = "Инструмент не найден".to_string();
            }
        } else {
            self.terminal_error = "Неизвестный QR код".to_string();
        }
    }

    fn issue_tool(&mut self) {
        if let (Some(employee), Some(tool), Some(user)) = (&self.scanned_employee, &self.scanned_tool, &self.current_user) {
            let issuance = Issuance {
                id: uuid::Uuid::new_v4().to_string(),
                tool_id: tool.id.clone(),
                employee_id: employee.id.clone(),
                user_id: user.id.clone(),
                quantity: self.issue_quantity,
                issued_at: Local::now(),
                returned_at: None,
            };

            match self.db.issue_tool(issuance) {
                Ok(()) => {
                    self.terminal_message = format!("Выдано: {} x{}", tool.name, self.issue_quantity);
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
