use crate::ProcessItem;
use dialoguer::theme::ColorfulTheme;
use dialoguer::FuzzySelect;
use sysinfo::System;

pub struct ProcessManager {
    pub(crate) system: System,
}

impl ProcessManager {
    pub fn new() -> Self {
        ProcessManager {
            system: System::new_all(),
        }
    }

    pub fn get_filtered_processes(&self, filter: &str) -> Vec<ProcessItem> {
        self.system
            .processes()
            .iter()
            .map(|(&pid, proc)| ProcessItem {
                pid: pid.as_u32(),
                name: format!(
                    "{} - {}",
                    proc.name().to_string_lossy(),
                    proc.cmd()
                        .iter()
                        .map(|s| s.to_string_lossy())
                        .collect::<Vec<_>>()
                        .join(" ")
                ),
            })
            .filter(|proc| proc.name.to_lowercase().contains(&filter.to_lowercase()))
            .collect()
    }

    pub fn select_process(&self, processes: &[ProcessItem]) -> Option<usize> {
        let selection_items: Vec<String> = processes
            .iter()
            .map(|proc| format!("PID {}: {}", proc.pid, proc.name))
            .collect();

        FuzzySelect::with_theme(&ColorfulTheme::default())
            .with_prompt("Select process")
            .default(0)
            .max_length(6)
            .items(&selection_items)
            .interact_opt()
            .unwrap()
    }
}
