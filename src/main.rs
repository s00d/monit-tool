use sysinfo::{Pid, PidExt, ProcessExt, System, SystemExt};
use std::io::{self};
use std::process::Command;
use std::thread;
use std::time::Duration;
use dialoguer::{Input, Select};
use dialoguer::theme::ColorfulTheme;
use rgb::RGB8;
use textplots::{Chart, ColorPlot, Shape};

struct ProcessItem {
    pid: u32,
    name: String,
}

fn clear_screen() {
    if cfg!(windows) {
        // On Windows, use "cls" to clear the screen
        Command::new("cmd")
            .args(&["/c", "cls"])
            .status()
            .expect("Failed to clear screen");
    } else {
        // On Unix-like systems, use "clear" to clear the screen
        Command::new("clear")
            .status()
            .expect("Failed to clear screen");
    }
}

fn main() -> Result<(), io::Error> {
    let mut system = System::new_all();

    let processes: Vec<ProcessItem> = system.processes()
        .iter()
        .map(|(&pid, proc)| {
            let name = proc.name().to_string();
            let cmdline = proc.cmd().join(" ").to_string(); // Получаем параметры командной строки

            ProcessItem {
                pid: pid.as_u32(),
                name: format!("{} - {}", name, cmdline), // Объединяем имя и параметры командной строки
            }
        })
        .collect();

    let filter: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Enter a filter for the process name")
        .default("".into())
        .interact_text()
        .unwrap();

// Применение фильтра
    let filtered_processes: Vec<&ProcessItem> = processes.iter()
        .filter(|proc| proc.name.to_lowercase().contains(&filter.to_lowercase()))
        .collect();

    if filtered_processes.is_empty() {
        println!("There are no processes matching the filter.");
        return Ok(());
    }

// Подготовка списка строк для интерфейса выбора
    let selection_items: Vec<String> = filtered_processes
        .iter()
        .map(|proc| format!("PID {}: {}", proc.pid, proc.name))
        .collect();

// Выбор процесса из отфильтрованного списка
    let selection_index = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select process")
        .default(0)
        .items(&selection_items)
        .interact_opt()
        .unwrap();

    match selection_index {
        Some(index) => {
            let selected_process = &filtered_processes[index];
            println!("Process selected: PID {} - {}", selected_process.pid, selected_process.name);
            let pid = Pid::from_u32(selected_process.pid);

            let mut cpu_data = vec![]; // Вектор для хранения данных CPU
            let mut memory_data = vec![]; // Вектор для хранения данных памяти

            loop {
                system.refresh_process(pid);
                if let Some(proc) = system.process(pid) {
                    let cpu_usage = proc.cpu_usage() as f32;
                    let memory_usage = proc.memory() as f32 / 1024.0 / 1024.0; // Convert to MB

                    cpu_data.push((cpu_data.len() as f32, cpu_usage)); // Добавляем данные CPU как кортежи
                    memory_data.push((memory_data.len() as f32, memory_usage)); // Добавляем данные памяти как кортежи

                    if cpu_data.len() >= 100 {
                        cpu_data.clear();
                        memory_data.clear();
                    }

                    clear_screen();
                    println!("\nred = CPU (Usage: {:.2} %), green = Memory (Usage: {:.2} MB)\n", cpu_usage, memory_usage);
                    let mut chart = Chart::new(280, 40, -1.0, 100.0);
                    chart.linecolorplot(
                        &Shape::Lines(&cpu_data),
                        RGB8::new(255, 0, 0),
                    ).linecolorplot(
                        &Shape::Lines(&memory_data),
                        RGB8::new(0, 255, 0),
                    ).display();
                }

                thread::sleep(Duration::from_secs(1));
            }
        }
        None => println!("The selection has been cancelled."),
    }
    Ok(())
}
