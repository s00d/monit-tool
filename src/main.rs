mod process_manager;
mod data_collector;
mod chart_manager;
mod logger;

use sysinfo::{Pid};
use std::io::{self};
use std::{thread, time::Duration};
use clap::Parser;
use crate::chart_manager::ChartManager;
use crate::data_collector::DataCollector;
use crate::logger::Logger;
use crate::process_manager::ProcessManager;

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

    /// timeout before refresh
    #[arg(long, default_value_t = 50)]
    sleep: u64,
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

    let process_manager = ProcessManager::new();
    let processes = process_manager.get_filtered_processes(&args.name);
    let selected_index = process_manager.select_process(&processes);

    let term = console::Term::stdout();
    term.hide_cursor().unwrap();
    term.clear_screen().unwrap();

    match selected_index {
        Some(index) => {
            let selected_process = &processes[index];
            let mut pid = Pid::from_u32(selected_process.pid);
            let selected_cmdline = selected_process.name.clone();

            let mut data_collector = DataCollector::new();

            let mut tick = 0;
            let mut max: f32 = 0.;
            let mut memory_usage_min: f32 = 0.;
            let mut memory_color = GREEN;
            let mut logger = Logger::new(args.logging)?;
            let mut system = process_manager.system;

            let mut chart_manager = ChartManager::new();
            loop {
                system.refresh_all();
                if let Some((cpu_usage, memory_usage, total_written_bytes, total_read_bytes, name, status)) = data_collector.get_process_data(&system, pid) {
                    if memory_usage_min == 0. && memory_usage > 0. {
                        memory_usage_min = memory_usage;
                    }
                    if memory_usage > memory_usage_min * 2. {
                        memory_color = RED;
                    } else if memory_usage <= memory_usage_min * 2. {
                        memory_color = GREEN;
                    }

                    if memory_usage > max {
                        max = memory_usage;
                    }

                    let x_label = x_label_format(tick, cpu_usage, memory_usage, total_written_bytes, total_read_bytes, name.clone(), status.clone(), args.disk_write, args.disk_read);

                    data_collector.update_cpu_data(cpu_usage);
                    data_collector.update_memory_data(memory_usage);
                    data_collector.update_disk_read_data(total_read_bytes);
                    data_collector.update_disk_write_data(total_written_bytes);

                    logger.log(&x_label)?;

                    term.move_cursor_to(0, 0).unwrap();

                    chart_manager
                        .set_cpu_data(&data_collector.cpu_data)
                        .set_memory_data(&data_collector.memory_data)
                        .set_disk_read_data(&data_collector.disk_write_data)
                        .set_disk_write_data(&data_collector.disk_read_data)
                        .set_x_label(x_label)
                        .set_cpu_usage(cpu_usage)
                        .set_memory_usage(memory_usage)
                        .set_memory_color(memory_color)
                        .set_disk_write(args.disk_write)
                        .set_disk_read(args.disk_read)
                        .set_max(max)
                        .draw_chart();

                    tick += 1;
                } else {
                    if !args.watch {
                        break;
                    }
                    let term = console::Term::stdout();
                    term.show_cursor().unwrap();
                    let new_pid = system.processes().iter()
                        .find(|(_, p)| format!("{} - {}", p.name(), p.cmd().join(" ")) == selected_cmdline)
                        .map(|(&pid, _)| pid.as_u32());


                    term.move_cursor_to(0, 0).unwrap();
                    println!("Waiting process... {:?}", selected_cmdline);

                    if let Some(new_pid_value) = new_pid {
                        println!("Process restarted with PID: {}", new_pid_value);
                        pid = Pid::from_u32(new_pid_value);
                    }
                }

                thread::sleep(Duration::from_millis(args.sleep));
            }
        }
        None => println!("The selection has been cancelled."),
    }

    Ok(())
}