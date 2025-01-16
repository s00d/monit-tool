use std::collections::HashSet;
use sysinfo::{Pid, System};
use crate::PRINT_LEN;


pub struct DataCollector {
    pub(crate) cpu_data: [(f32, f32); PRINT_LEN],
    pub(crate) memory_data: [(f32, f32); PRINT_LEN],
    pub(crate) disk_write_data: [(f32, f32); PRINT_LEN],
    pub(crate) disk_read_data: [(f32, f32); PRINT_LEN],
}

impl DataCollector {
    pub fn new() -> Self {
        DataCollector {
            cpu_data: [(0., 0.); PRINT_LEN],
            memory_data: [(0., 0.); PRINT_LEN],
            disk_write_data: [(0., 0.); PRINT_LEN],
            disk_read_data: [(0., 0.); PRINT_LEN],
        }
    }

    fn collect_process_data(&self, system: &System, pid: Pid, visited: &mut HashSet<Pid>) -> (f32, f32, f32, f32, String, String) {
        let mut total_cpu_usage = 0.0;
        let mut total_memory_usage = 0.0;
        let mut total_written_bytes = 0.0;
        let mut total_read_bytes = 0.0;
        let mut name = String::new();
        let mut status = String::new();

        if visited.contains(&pid) {
            return (0.0, 0.0, 0.0, 0.0, String::new(), String::new());
        }
        visited.insert(pid);

        if let Some(proc) = system.process(pid) {
            // Добавляем данные текущего процесса
            total_cpu_usage += proc.cpu_usage() as f32;
            total_memory_usage += proc.memory() as f32 / 1024.0 / 1024.0;
            total_written_bytes += proc.disk_usage().written_bytes as f32 / 1024.0 / 1024.0;
            total_read_bytes += proc.disk_usage().read_bytes as f32 / 1024.0 / 1024.0;
            name = proc.cmd().iter().map(|s| s.to_string_lossy()).collect::<Vec<_>>().join(" ");
            status = proc.status().to_string();

            // Рекурсивно собираем данные о дочерних процессах
            for child_pid in system.processes().values()
                .filter(|p| p.parent().map_or(false, |parent_pid| parent_pid == pid))
            {
                let child_data = self.collect_process_data(system, child_pid.pid(), visited);
                total_cpu_usage += child_data.0;
                total_memory_usage += child_data.1;
                total_written_bytes += child_data.2;
                total_read_bytes += child_data.3;
            }
        }

        (total_cpu_usage, total_memory_usage, total_written_bytes, total_read_bytes, name, status)
    }

    pub fn get_process_data(&self, system: &System, pid: Pid) -> Option<(f32, f32, f32, f32, String, String)> {
        // Собираем данные для указанного PID и всех его дочерних процессов
        let mut visited = HashSet::new();
        let (cpu_usage, memory_usage, total_written_bytes, total_read_bytes, name, status) = self.collect_process_data(system, pid, &mut visited);

        Some((cpu_usage, memory_usage, total_written_bytes, total_read_bytes, name, status))
    }

    pub fn update_cpu_data(&mut self, new_value: f32) -> &mut DataCollector {
        let len = self.cpu_data.len();
        self.cpu_data.copy_within(1..len, 0);
        self.cpu_data[len - 1] = (0., new_value);
        for point in self.cpu_data.iter_mut() {
            point.0 += 1.;
        }

        self
    }

    pub fn update_memory_data(&mut self, new_value: f32) -> &mut DataCollector {
        let len = self.memory_data.len();
        self.memory_data.copy_within(1..len, 0);
        self.memory_data[len - 1] = (0., new_value);
        for point in self.memory_data.iter_mut() {
            point.0 += 1.;
        }
        self
    }

    pub fn update_disk_write_data(&mut self, new_value: f32) -> &mut DataCollector {
        let len = self.disk_write_data.len();
        self.disk_write_data.copy_within(1..len, 0);
        self.disk_write_data[len - 1] = (0., new_value);
        for point in self.disk_write_data.iter_mut() {
            point.0 += 1.;
        }
        self
    }

    pub fn update_disk_read_data(&mut self, new_value: f32) -> &mut DataCollector {
        let len = self.disk_read_data.len();
        self.disk_read_data.copy_within(1..len, 0);
        self.disk_read_data[len - 1] = (0., new_value);
        for point in self.disk_read_data.iter_mut() {
            point.0 += 1.;
        }
        self
    }
}