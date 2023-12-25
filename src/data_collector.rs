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

    pub fn get_process_data(&self, system: &System, pid: Pid) -> Option<(f32, f32, f32, f32, String, String)> {
        system.process(pid).map(|proc| {
            let cpu_usage = proc.cpu_usage() as f32;
            let memory_usage = proc.memory() as f32 / 1024.0 / 1024.0;
            let total_written_bytes = proc.disk_usage().total_written_bytes as f32 / 1024.0 / 1024.0;
            let total_read_bytes = proc.disk_usage().total_read_bytes as f32 / 1024.0 / 1024.0;
            let status = proc.status().to_string();
            let name = proc.cmd().join(" ");
            (cpu_usage, memory_usage, total_written_bytes, total_read_bytes, name, status)
        })
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