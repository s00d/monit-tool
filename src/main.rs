mod chart_manager;
mod data_collector;
mod logger;
mod process_manager;

use crate::chart_manager::ChartManager;
use crate::data_collector::DataCollector;
use crate::logger::Logger;
use crate::process_manager::ProcessManager;
use clap::Parser;
use ctrlc;
use std::io::{self};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;
use std::{thread, time::Duration};
use sysinfo::Pid;

const RED: rgb::RGB8 = rgb::RGB8::new(0xFF, 0x00, 0x00);
const GREEN: rgb::RGB8 = rgb::RGB8::new(0x00, 0xFF, 0x00);
const PURPLE: rgb::RGB8 = rgb::RGB8::new(0xE0, 0x80, 0xFF);
const BLUE: rgb::RGB8 = rgb::RGB8::new(0x00, 0x00, 0xFF);
const ORANGE: rgb::RGB8 = rgb::RGB8::new(0xFF, 0xA5, 0x00);

const PRINT_LEN: usize = 500;

struct ProcessItem {
    pid: u32,
    name: String,
}

fn format_duration(duration: Duration) -> String {
    let total_seconds = duration.as_secs();
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}

fn start_process(command: &str, workdir: &str) -> Result<Child, io::Error> {
    let child = if cfg!(target_os = "windows") {
        // Для Windows используем cmd.exe с параметром /c
        Command::new("cmd.exe")
            .arg("/c")
            .arg(command)
            .current_dir(workdir)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?
    } else {
        // Для Unix-подобных систем используем sh с параметром -c
        Command::new("sh")
            .arg("-c")
            .arg(command)
            .current_dir(workdir)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?
    };

    Ok(child)
}
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Name of the process
    #[arg(short, long, default_value_t = String::from(""))]
    name: String,

    /// Command to execute and monitor
    #[arg(short, long, default_value_t = String::from(""))]
    command: String,

    /// Working directory for the command
    #[arg(long, default_value_t = String::from("."))]
    workdir: String,

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

    /// Disable chart output
    #[arg(long, default_value_t = false)]
    nochart: bool,

    /// timeout before refresh
    #[arg(long, default_value_t = 50)]
    sleep: u64,
}

fn x_label_format(
    tick: usize,
    cpu_usage: f32,
    memory_usage: f32,
    total_written_bytes: f32,
    total_read_bytes: f32,
    name: String,
    status: String,
    disk_write: bool,
    disk_read: bool,
) -> String {
    let mut label = format!(
        "{} ORANGE = CPU (Usage: {:.2} %), GREEN/RED = Memory (Usage: {:.2} MB)",
        tick, cpu_usage, memory_usage
    );
    if disk_write {
        label += &format!(
            ", PURPLE - disk write (Usage: {:.2} MB)",
            total_written_bytes
        );
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

    // Флаг для отслеживания завершения программы
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    // Захват сигнала Ctrl+C
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl+C handler");

    let process_manager = ProcessManager::new();
    let mut child = if !args.command.is_empty() {
        // Если указана команда, запускаем процесс и получаем его Child
        Some(start_process(&args.command, &args.workdir)?)
    } else {
        None
    };

    let mut pid = if let Some(ref child_process) = child {
        Pid::from_u32(child_process.id())
    } else {
        // Если процесс выбран из списка, используем его PID
        let processes = process_manager.get_filtered_processes(&args.name);
        let selected_index = process_manager.select_process(&processes).unwrap();
        Pid::from_u32(processes[selected_index].pid)
    };

    let term = console::Term::stdout();
    term.hide_cursor().unwrap();
    term.clear_screen().unwrap();

    let mut data_collector = DataCollector::new();
    let mut restart_count = 0;
    let mut tick = 0;
    let mut max: f32 = 0.;
    let mut memory_usage_min: f32 = 0.;
    let mut memory_color = GREEN;
    let mut logger = Logger::new(args.logging)?;
    let mut system = process_manager.system;

    system.refresh_all();

    thread::sleep(Duration::from_millis(500)); // Небольшая задержка

    system.refresh_all();

    let mut chart_manager = ChartManager::new();

    // Переменные для хранения максимальных значений
    let mut max_cpu_usage: f32 = 0.;
    let mut max_memory_usage: f32 = 0.;

    // Время начала работы программы
    let start_time = Instant::now();

    // Основной цикл
    while running.load(Ordering::SeqCst) {
        // Проверяем состояние дочернего процесса, если он был запущен
        if let Some(ref mut child_process) = child {
            if let Ok(Some(status)) = child_process.try_wait() {
                println!("Process exited with status: {}", status);

                if args.watch && !args.command.is_empty() {
                    // Перезапускаем процесс, если включен режим watch и есть команда
                    println!("Restarting process...");
                    child = Some(start_process(&args.command, &args.workdir)?);
                    pid = Pid::from_u32(child.as_ref().unwrap().id());
                    restart_count += 1;
                    continue; // Пропускаем остальную часть цикла и начинаем заново
                } else {
                    break; // Завершаем программу, если процесс завершился и перезапуск не требуется
                }
            }
        }

        // Второе обновление для точного измерения CPU usage
        system.refresh_all();

        if let Some(process_data) = data_collector.get_process_data(&system, pid) {
            if memory_usage_min == 0. && process_data.memory_usage > 0. {
                memory_usage_min = process_data.memory_usage;
            }
            if process_data.memory_usage > memory_usage_min * 2. {
                memory_color = RED;
            } else if process_data.memory_usage <= memory_usage_min * 2. {
                memory_color = GREEN;
            }

            if process_data.memory_usage > max {
                max = process_data.memory_usage;
            }

            // Обновление максимальных значений CPU и памяти
            if process_data.cpu_usage > max_cpu_usage {
                max_cpu_usage = process_data.cpu_usage;
            }
            if process_data.memory_usage > max_memory_usage {
                max_memory_usage = process_data.memory_usage;
            }

            let x_label = x_label_format(
                tick,
                process_data.cpu_usage,
                process_data.memory_usage,
                process_data.total_written_bytes,
                process_data.total_read_bytes,
                process_data.name.clone(),
                process_data.status.clone(),
                args.disk_write,
                args.disk_read,
            );

            data_collector.update_cpu_data(process_data.cpu_usage);
            data_collector.update_memory_data(process_data.memory_usage);
            data_collector.update_disk_read_data(process_data.total_read_bytes);
            data_collector.update_disk_write_data(process_data.total_written_bytes);

            if !args.nochart {
                logger.log(&x_label)?;

                term.move_cursor_to(0, 0).unwrap();

                chart_manager
                    .set_cpu_data(&data_collector.cpu_data)
                    .set_memory_data(&data_collector.memory_data)
                    .set_disk_read_data(&data_collector.disk_write_data)
                    .set_disk_write_data(&data_collector.disk_read_data)
                    .set_x_label(x_label)
                    .set_cpu_usage(process_data.cpu_usage)
                    .set_memory_usage(process_data.memory_usage)
                    .set_memory_color(memory_color)
                    .set_disk_write(args.disk_write)
                    .set_disk_read(args.disk_read)
                    .set_max(max)
                    .draw_chart();
            }

            tick += 1;
        } else {
            if !args.watch {
                break;
            }
            let term = console::Term::stdout();
            term.show_cursor().unwrap();
            let new_pid = system
                .processes()
                .iter()
                .find(|(_, p)| p.pid() == pid)
                .map(|(&pid, _)| pid.as_u32());

            term.move_cursor_to(0, 0).unwrap();
            println!("Waiting process... {:?}", pid);

            if let Some(new_pid_value) = new_pid {
                println!("Process restarted with PID: {}", new_pid_value);
                pid = Pid::from_u32(new_pid_value);
            }
        }

        thread::sleep(Duration::from_millis(args.sleep));
    }

    // Если был запущен процесс через args.command, завершаем его
    if let Some(mut child_process) = child {
        let _ = child_process.kill();
        println!("Process with PID {} has been terminated.", pid);
    }

    // Вывод статистики
    let elapsed_time = start_time.elapsed();
    let min_memory_usage = data_collector
        .memory_data
        .iter()
        .map(|&(_, value)| value) // Извлекаем второе значение из кортежа
        .fold(f32::INFINITY, f32::min); // Находим минимальное значение
                                        // Среднее использование CPU
    let avg_cpu_usage = data_collector
        .cpu_data
        .iter()
        .map(|&(_, value)| value) // Извлекаем второе значение из кортежа
        .sum::<f32>()
        / data_collector.cpu_data.len() as f32;

    // Среднее использование памяти
    let avg_memory_usage = data_collector
        .memory_data
        .iter()
        .map(|&(_, value)| value) // Извлекаем второе значение из кортежа
        .sum::<f32>()
        / data_collector.memory_data.len() as f32;

    // Общее количество записанных данных на диск
    let total_disk_write = data_collector
        .disk_write_data
        .iter()
        .map(|&(_, value)| value) // Извлекаем второе значение из кортежа
        .sum::<f32>();

    // Общее количество прочитанных данных на диск
    let total_disk_read = data_collector
        .disk_read_data
        .iter()
        .map(|&(_, value)| value) // Извлекаем второе значение из кортежа
        .sum::<f32>();

    // Минимальное использование CPU
    let min_cpu_usage = data_collector
        .cpu_data
        .iter()
        .map(|&(_, value)| value) // Извлекаем второе значение из кортежа
        .fold(f32::INFINITY, f32::min); // Находим минимальное значение
    println!("\nProgram finished.");
    if args.watch && !args.command.is_empty() {
        println!("Process restarts: {}", restart_count);
    }
    // println!("CPU cores: {}", system.cpus().len());
    // println!("Total memory: {:.2} MB", system.total_memory() as f32 / 1024.0 / 1024.0);
    println!("Min Memory Usage: {:.2} MB", min_memory_usage);
    println!("Min CPU Usage: {:.2}%", min_cpu_usage);
    println!("Max CPU Usage: {:.2}%", max_cpu_usage);
    println!("Average CPU Usage: {:.2}%", avg_cpu_usage);
    println!("Max Memory Usage: {:.2} MB", max_memory_usage);
    println!("Average Memory Usage: {:.2} MB", avg_memory_usage);
    println!("Total Disk Write: {:.2} MB", total_disk_write);
    println!("Total Disk Read: {:.2} MB", total_disk_read);
    println!("Total runtime: {}", format_duration(elapsed_time));

    Ok(())
}
