use sysinfo::{Pid, System};
use std::io::{self, Write};
use std::{thread, time::Duration};
use std::fs::File;
use chrono::Local;
use clap::Parser;
use dialoguer::{FuzzySelect, theme::ColorfulTheme};
use textplots::{Chart, ColorPlot, LabelBuilder, LabelFormat, Shape};

const PRINT_LEN: usize = 500;
const RED: rgb::RGB8 = rgb::RGB8::new(0xFF, 0x00, 0x00);
const GREEN: rgb::RGB8 = rgb::RGB8::new(0x00, 0xFF, 0x00);
const PURPLE: rgb::RGB8 = rgb::RGB8::new(0xE0, 0x80, 0xFF);
const BLUE: rgb::RGB8 = rgb::RGB8::new(0x00, 0x00, 0xFF);

const ORANGE: rgb::RGB8 = rgb::RGB8::new(0xFF, 0xA5, 0x00);

struct ProcessItem {
    pid: u32,
    name: String,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Name of the process
    #[arg(short, long, default_value_t = String::from(""))]
    name: String,

    /// Enable process watch mode
    #[arg(short, long, default_value_t = false)]
    watch: bool,

    /// Enable logging to file
    #[arg(short, long, default_value_t = false)]
    logging: bool,

    /// Enable disk write info
    #[arg(long, default_value_t = false)]
    disk_write: bool,

    /// Enable disk read info
    #[arg(long, default_value_t = false)]
    disk_read: bool,
}
fn update_data_array(data_array: &mut [(f32, f32)], new_value: f32) -> &mut [(f32, f32)] {
    data_array.copy_within(1..PRINT_LEN, 0);
    data_array[PRINT_LEN - 1] = (0., new_value);
    for point in data_array.iter_mut() {
        point.0 += 1.;
    }
    data_array
}

fn get_process_data(system: &System, pid: Pid) -> Option<(f32, f32, f32, f32, String, String)> {
    system.process(pid).map(|proc| {
        let cpu_usage = proc.cpu_usage() as f32;
        let memory_usage = proc.memory() as f32 / 1024.0 / 1024.0; // Convert to MB
        let total_written_bytes = proc.disk_usage().total_written_bytes as f32 / 1024.0 / 1024.0; // Convert to MB
        let total_read_bytes = proc.disk_usage().total_read_bytes as f32 / 1024.0 / 1024.0; // Convert to MB
        let status = proc.status().to_string();
        let name = proc.cmd().join(" ").to_string(); // Convert to MB
        (cpu_usage, memory_usage, total_written_bytes, total_read_bytes, name, status)
    })
}

fn x_label_format(tick: usize, cpu_usage: f32, memory_usage: f32, total_written_bytes: f32, total_read_bytes: f32, name: String, status: String, disk_write: bool, disk_read: bool) -> String {
    let mut label = format!("{} ORANGE = CPU (Usage: {:.2} %), GREEN/RED = Memory (Usage: {:.2} MB)", tick, cpu_usage, memory_usage);
    if disk_write {
        label += &format!(", PURPLE - disk write (Usage: {:.2} MB)", total_written_bytes);
    }
    if disk_read {
        label += &format!(", BLUE - disk read (Usage: {:.2} MB)", total_read_bytes);
    }
    label += &format!(" - {}", name);
    label += &format!(" ({})", status);
    label
}

fn main() -> Result<(), io::Error> {
    let args = Args::parse();

    let log_file = if args.logging {
        let now = Local::now();
        let file_name = format!("{}_{}.log", now.format("%Y-%m-%d"), now.format("%H-%M-%S"));
        Some(File::create(file_name)?)
    } else {
        None
    };

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
        .filter(|proc| proc.name.to_lowercase().contains(&args.name.to_lowercase()))
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
            let mut disk_write_data: [(f32, f32); PRINT_LEN] = [(0., 0.); PRINT_LEN];
            let mut disk_read_data: [(f32, f32); PRINT_LEN] = [(0., 0.); PRINT_LEN];
            let mut tick = 0; // Вектор для хранения данных памяти

            let mut max: f32 = 0.;

            let mut memory_usage_last: f32 = 0.;
            let mut memory_color = GREEN;
            loop {
                system.refresh_all();
                if let Some((cpu_usage, memory_usage, total_written_bytes, total_read_bytes, name, status)) = get_process_data(&system, pid) {
                    if memory_usage_last == 0. {
                        memory_usage_last = memory_usage;
                    }

                    if memory_usage > memory_usage_last * 2. {
                        memory_color = RED;
                    } else if memory_usage <= memory_usage_last {
                        memory_color = GREEN;
                    }

                    let cpu_data_clone = update_data_array(&mut cpu_data, cpu_usage);
                    let memory_data_clone = update_data_array(&mut memory_data, memory_usage);
                    let disk_write_data_clone = update_data_array(&mut disk_write_data, total_written_bytes);
                    let disk_read_data_clone = update_data_array(&mut disk_read_data, total_read_bytes);

                    if memory_usage > max {
                        max = memory_usage
                    }

                    let x_label = x_label_format(tick, cpu_usage, memory_usage, total_written_bytes, total_read_bytes, name.clone(), status.clone(), args.disk_write, args.disk_read);

                    if let Some(mut file) = log_file.as_ref().map(|f| f.try_clone().expect("Failed to clone file")) {
                        let now = Local::now();
                        let timestamp = now.format("%Y-%m-%d %H:%M:%S").to_string();
                        let log_message = format!("[{}] {} \n", timestamp, x_label.clone());
                        file.write_all(log_message.as_bytes())?;
                    }

                    term.move_cursor_to(0, 0).unwrap();
                    let mut chart = Chart::new_with_y_range(280, 40, -1.5, PRINT_LEN as f32, 0., max);


                    chart.axis();
                    chart.figures();

                    let cpu_shape = Shape::Lines(&cpu_data_clone);
                    let memory_shape = Shape::Lines(&memory_data_clone);
                    let disk_write_shape = Shape::Lines(&disk_write_data_clone);
                    let disk_read_shape = Shape::Lines(&disk_read_data_clone);

                    let mut dots = chart.linecolorplot(&cpu_shape, ORANGE);
                    dots = dots.linecolorplot(&memory_shape, memory_color);
                    if args.disk_write {
                        dots = dots.linecolorplot(&disk_write_shape, PURPLE);
                    }
                    if args.disk_read {
                        dots = dots.linecolorplot(&disk_read_shape, BLUE);
                    }

                    dots = dots.x_label_format(LabelFormat::Custom(Box::new(move |val| {
                        if val > 0. {
                            return format!("")
                        }
                        x_label.clone()
                    })));

                    dots = dots.y_label_format(LabelFormat::Custom(Box::new(move |val| {
                        if val == 0. {
                            return format!("{:.2}% MB", cpu_usage)
                        }
                        format!("{:.2} MB", memory_usage)
                    })));

                    dots.display();

                    // println!("{}", dots.to_string());


                    // text_to_gif(dots.to_string(),  "output.gif").unwrap();

                    tick += 1;
                } else {
                    if !args.watch {
                        break;
                    }
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