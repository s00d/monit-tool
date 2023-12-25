use textplots::{Chart, ColorPlot, LabelBuilder, LabelFormat, Shape};
use rgb::RGB8;
use crate::{BLUE, ORANGE, PRINT_LEN, PURPLE};

pub struct ChartManager {
    // Data for CPU, memory, disk write, and disk read
    pub cpu_data: Vec<(f32, f32)>,
    pub memory_data: Vec<(f32, f32)>,
    pub disk_write_data: Vec<(f32, f32)>,
    pub disk_read_data: Vec<(f32, f32)>,

    // Maximum value for the Y-axis
    pub max: f32,

    // X-axis label
    pub x_label: String,

    // CPU and memory usage values
    pub cpu_usage: f32,
    pub memory_usage: f32,

    // Color for memory data
    pub memory_color: RGB8,

    // Flags indicating whether disk write and read data should be displayed
    pub disk_write: bool,
    pub disk_read: bool,
}

impl ChartManager {
    pub fn new() -> Self {
        ChartManager {
            cpu_data: vec![],
            memory_data: vec![],
            disk_write_data: vec![],
            disk_read_data: vec![],
            max: 0.0,
            x_label: String::new(),
            cpu_usage: 0.0,
            memory_usage: 0.0,
            memory_color: RGB8::new(0, 0, 0), // Замените этот цвет на начальное значение
            disk_write: false,
            disk_read: false,
        }
    }

    pub fn set_cpu_data(&mut self, data: &[(f32, f32); PRINT_LEN]) -> &mut ChartManager {
        self.cpu_data = Vec::from(data);
        self
    }

    pub fn set_memory_data(&mut self, data: &[(f32, f32); PRINT_LEN]) -> &mut ChartManager {
        self.memory_data = Vec::from(data);
        self
    }

    pub fn set_disk_write_data(&mut self, data: &[(f32, f32); PRINT_LEN]) -> &mut ChartManager {
        self.disk_write_data = Vec::from(data);
        self
    }

    pub fn set_disk_read_data(&mut self, data: &[(f32, f32); PRINT_LEN]) -> &mut ChartManager {
        self.disk_read_data = Vec::from(data);
        self
    }

    pub fn set_max(&mut self, max: f32) -> &mut ChartManager {
        self.max = max;
        self
    }

    pub fn set_x_label(&mut self, label: String) -> &mut ChartManager {
        self.x_label = label;
        self
    }

    pub fn set_cpu_usage(&mut self, usage: f32) -> &mut ChartManager {
        self.cpu_usage = usage;
        self
    }

    pub fn set_memory_usage(&mut self, usage: f32) -> &mut ChartManager {
        self.memory_usage = usage;
        self
    }

    pub fn set_memory_color(&mut self, color: RGB8) -> &mut ChartManager {
        self.memory_color = color;
        self
    }

    pub fn set_disk_write(&mut self, write: bool) -> &mut ChartManager {
        self.disk_write = write;
        self
    }

    pub fn set_disk_read(&mut self, read: bool) -> &mut ChartManager {
        self.disk_read = read;
        self
    }
}

impl ChartManager {
    // Function to draw the chart based on the configured data
    pub fn draw_chart(&self) {
        // Create a new chart with specified settings
        let mut chart = Chart::new_with_y_range(280, 40, -1.5, self.cpu_data.len() as f32, 0., self.max);
        chart.axis();
        chart.figures();

        // Define shapes for CPU, memory, disk write, and disk read data
        let cpu_shape = Shape::Lines(&*self.cpu_data);
        let memory_shape = Shape::Lines(&*self.memory_data);
        let disk_write_shape = Shape::Lines(&*self.disk_write_data);
        let disk_read_shape = Shape::Lines(&*self.disk_read_data);

        // Create and configure the plot with different line colors
        let mut dots = chart.linecolorplot(&cpu_shape, ORANGE);
        dots = dots.linecolorplot(&memory_shape, self.memory_color);
        if self.disk_write {
            dots = dots.linecolorplot(&disk_write_shape, PURPLE);
        }
        if self.disk_read {
            dots = dots.linecolorplot(&disk_read_shape, BLUE);
        }

        // Configure the x-axis label using a closure
        let xlabel = self.x_label.clone();
        dots = dots.x_label_format(LabelFormat::Custom(Box::new(move |val| {
            if val > 0. {
                return String::new();
            }
            xlabel.clone()
        })));

        // Configure the y-axis label using a closure
        let cpu_usage = self.cpu_usage.clone();
        let memory_usage = self.memory_usage.clone();
        dots = dots.y_label_format(LabelFormat::Custom(Box::new(move |val| {
            if val == 0. {
                return format!("{:.2}% MB", cpu_usage)
            }
            format!("{:.2} MB", memory_usage)
        })));

        // Display the chart
        dots.display();
    }
}