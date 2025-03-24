use eframe::{egui, NativeOptions};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::PathBuf;

// Define the structure for a single to-do item
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
struct TodoItem {
    id: u32,
    text: String,
    completed: bool,
}

// Define the main application structure
#[derive(Serialize, Deserialize, Debug, Default)]
struct TodoApp {
    tasks: Vec<TodoItem>,
    new_task_text: String,
    next_id: u32,
    #[serde(skip)] // Don't serialize/deserialize this as it's UI state
    show_error: Option<String>,
}

impl TodoApp {
    // Create a new instance of the TodoApp
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Load tasks from file if it exists
        if let Ok(app) = Self::load() {
            app
        } else {
            Self::default()
        }
    }

    // Save the current state of the app to a JSON file
    fn save(&self) -> Result<(), io::Error> {
        let path = Self::data_file_path();
        let data = serde_json::to_string_pretty(self)?;
        fs::write(path, data)?;
        Ok(())
    }

    // Load the app state from a JSON file
    fn load() -> Result<Self, io::Error> {
        let path = Self::data_file_path();
        let data = fs::read_to_string(path)?;
        let app = serde_json::from_str(&data)?;
        Ok(app)
    }

    // Get the path to the data file
    fn data_file_path() -> PathBuf {
        let mut path = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("rust_todo_app");
        fs::create_dir_all(&path).unwrap_or_else(|_| {
            eprintln!("Error creating data directory");
        });
        path.push("todo_data.json");
        path
    }

    // Add a new task to the list
    fn add_task(&mut self) {
        if !self.new_task_text.trim().is_empty() {
            self.tasks.push(TodoItem {
                id: self.next_id,
                text: self.new_task_text.trim().to_string(),
                completed: false,
            });
            self.next_id += 1;
            self.new_task_text.clear();
            if let Err(e) = self.save() {
                self.show_error = Some(format!("Error saving tasks: {}", e));
            }
        }
    }

    // Toggle the completion status of a task
    fn toggle_task(&mut self, id: u32) {
        if let Some(task) = self.tasks.iter_mut().find(|task| task.id == id) {
            task.completed = !task.completed;
            if let Err(e) = self.save() {
                self.show_error = Some(format!("Error saving tasks: {}", e));
            }
        }
    }

    // Delete a task from the list
    fn delete_task(&mut self, id: u32) {
        self.tasks.retain(|task| task.id != id);
        if let Err(e) = self.save() {
            self.show_error = Some(format!("Error saving tasks: {}", e));
        }
    }
}

// Implement the `eframe::App` trait for our application
impl eframe::App for TodoApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("My To-Do List");
            ui.separator();

            // Input field for adding new tasks
            ui.horizontal(|ui| {
                ui.text_edit_singleline(&mut self.new_task_text);
                if ui.button("Add").clicked() {
                    self.add_task();
                }
            });
            ui.separator();

            // Display the list of tasks
            if self.tasks.is_empty() {
                ui.label("No tasks yet! Add some above.");
            } else {
                for task in &mut self.tasks {
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut task.completed, &task.text).on_hover_text("Mark as completed/incomplete");
                        if ui.button("🗑").clicked() {
                            self.delete_task(task.id);
                        }
                    });
                }
            }
            ui.separator();

            // Display any error messages
            if let Some(error) = &self.show_error {
                ui.colored_label(egui::Color32::RED, error);
            }
        });
    }
}

// Main function to run the application
fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log panics to the console
    let options = NativeOptions::default();
    eframe::run_native(
        "Rust To-Do App",
        options,
        Box::new(|cc| Box::new(TodoApp::new(cc))),
    )
}