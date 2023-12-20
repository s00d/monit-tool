use sysinfo::{Pid, PidExt, ProcessExt, System, SystemExt};
use std::io::{self};
use std::process::Command;
use std::{env, thread};
use std::time::Duration;
use dialoguer::{FuzzySelect};
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
    let args: Vec<String> = env::args().collect();

    // Используем unwrap_or для установки значения по умолчанию (пустая строка)
    let binding = String::from("");
    let filter = args.get(1).unwrap_or(&binding);

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
        .filter(|proc| proc.name.to_lowercase().contains(&filter.to_lowercase()))
        .collect();

// Подготовка списка строк для интерфейса выбора
    let selection_items: Vec<String> = processes
        .iter()
        .map(|proc| format!("PID {}: {}", proc.pid, proc.name))
        .collect();

// Выбор процесса из отфильтрованного списка
    let selection_index = FuzzySelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Select process")
        .default(0)
        .max_length(6)
        .items(&selection_items)
        .interact_opt()
        .unwrap();

    match selection_index {
        Some(index) => {
            let selected_process = &processes[index];
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
                    println!("\nred = CPU (Usage: {:.2} %), green = Memory (Usage: {:.2} MB) - {}\n", cpu_usage, memory_usage, proc.cmd().join(" ").to_string());
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
