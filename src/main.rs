use sysinfo::{Pid, PidExt, ProcessExt, System, SystemExt};
use std::io::{self};
use std::{env, thread};
use std::time::Duration;
use dialoguer::{FuzzySelect};
use dialoguer::theme::ColorfulTheme;
use textplots::{Chart, ColorPlot, LabelBuilder, LabelFormat, Shape};

const PRINT_LEN: usize = 500;
const RED: rgb::RGB8 = rgb::RGB8::new(0xFF, 0x00, 0x00);
const GREEN: rgb::RGB8 = rgb::RGB8::new(0x00, 0xFF, 0x00);

struct ProcessItem {
    pid: u32,
    name: String,
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

    let term = console::Term::stdout();
    term.hide_cursor().unwrap();
    term.clear_screen().unwrap();

    match selection_index {
        Some(index) => {
            let selected_process = &processes[index];
            let mut pid = Pid::from_u32(selected_process.pid);
            let selected_cmdline = selected_process.name.clone();

            let mut cpu_data: [(f32, f32); PRINT_LEN] = [(0., 0.); PRINT_LEN]; // Вектор для хранения данных CPU
            let mut memory_data: [(f32, f32); PRINT_LEN] = [(0., 0.); PRINT_LEN]; // Вектор для хранения данных памяти
            let mut tick = 0; // Вектор для хранения данных памяти

            let mut max: f32 = 0.;

            loop {
                system.refresh_all();
                if let Some(proc) = system.process(pid) {
                    let cpu_usage = proc.cpu_usage() as f32;
                    let memory_usage = proc.memory() as f32 / 1024.0 / 1024.0; // Convert to MB
                    // let disk_write = proc.disk_usage().written_bytes as f32; // Convert to MB
                    let name = proc.cmd().join(" ").to_string(); // Convert to MB

                    if memory_usage > max {
                        max = memory_usage
                    }
                    cpu_data.copy_within(1..PRINT_LEN, 0);
                    memory_data.copy_within(1..PRINT_LEN, 0);
                    cpu_data[PRINT_LEN - 1] = (0., cpu_usage as f32);
                    memory_data[PRINT_LEN - 1] = (0., memory_usage as f32);
                    for index in 0..PRINT_LEN {
                        cpu_data[index].0 += 1.;
                        memory_data[index].0 += 1.;
                    }

                    term.move_cursor_to(0, 0).unwrap();
                    Chart::new_with_y_range(280, 40, -1.5, PRINT_LEN as f32, 0., max)
                        .linecolorplot(&Shape::Lines(&memory_data), GREEN)
                        .linecolorplot(&Shape::Lines(&cpu_data), RED)
                        // .y_label_format(LabelFormat::Value)
                        .x_label_format(LabelFormat::Custom(Box::new(move |val| {
                            if val > 0. {
                                return format!("")
                            }
                            format!("{} red = CPU (Usage: {:.2} %), green = Memory (Usage: {:.2} MB) - {}", tick, cpu_usage, memory_usage, name)
                        })))
                        .y_label_format(LabelFormat::Custom(Box::new(move |val| {
                            if val == 0. {
                                return format!("{:.2}%", cpu_usage)
                            }
                            format!("{:.2} MB", val)
                        })))
                        .display();

                    tick += 1;
                } else {
                    let term = console::Term::stdout();
                    term.show_cursor().unwrap();
                    let new_pid = system.processes().iter()
                        .find(|(_, p)| format!("{} - {}", p.name(), p.cmd().join(" ").to_string()) == selected_cmdline)
                        .map(|(&pid, _)| pid.as_u32());


                    term.move_cursor_to(0, 0).unwrap();
                    println!("Waiting process... {:?}", selected_cmdline);

                    if let Some(new_pid_value) = new_pid {
                        println!("Process restarted with PID: {}", new_pid_value);
                        pid = Pid::from_u32(new_pid_value); // Обновляем PID
                    }
                }


                thread::sleep(Duration::from_millis(50));
            }
        }
        None => println!("The selection has been cancelled."),
    }
    Ok(())
}