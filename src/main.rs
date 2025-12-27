#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;
use egui_extras::{TableBuilder, Column};
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::thread;
use std::sync::{Arc, Mutex};
use std::env;

const MAGIC_HEADER: &[u8] = b"YAAM";
const SCRAMBLE_KEY: u8 = 0x59;
const FILE_EXT: &str = "yaam";

fn main() -> eframe::Result<()> {
    let args: Vec<String> = env::args().collect();
    let startup_path = if args.len() > 1 {
        Some(PathBuf::from(&args[1]))
    } else {
        None
    };

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([600.0, 450.0])
            .with_min_inner_size([400.0, 300.0])
            .with_title("YAAM Archiver")
            .with_icon(eframe::icon_data::from_png_bytes(include_bytes!("icon.png")).unwrap_or_default()),
        ..Default::default()
    };

    eframe::run_native(
        "YAAM",
        options,
        Box::new(move |cc| {
            setup_custom_theme(&cc.egui_ctx);
            let mut app = YaamApp::default();


            if let Some(path) = startup_path {
                if path.is_dir() {

                    app.selected_tab = Tab::Pack;
                    app.pack_input_folder = Some(path);
                } else if path.exists() {

                    app.selected_tab = Tab::ViewAndExtract;
                    app.view_input_file = Some(path);
                    app.start_listing();
                }
            }

            Ok(Box::new(app))
        }),
    )
}


fn setup_custom_theme(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.window_rounding = egui::Rounding::same(10.0);
    visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(20, 20, 25);
    let accent = egui::Color32::from_rgb(0, 150, 255);
    visuals.selection.bg_fill = accent;
    visuals.hyperlink_color = accent;
    visuals.widgets.inactive.weak_bg_fill = egui::Color32::from_rgb(40, 40, 50);
    visuals.widgets.hovered.weak_bg_fill = egui::Color32::from_rgb(60, 60, 80);
    ctx.set_visuals(visuals);
}


struct YaamApp {
    selected_tab: Tab,
    status_message: Arc<Mutex<String>>,
    is_working: Arc<Mutex<bool>>,
    pack_input_folder: Option<PathBuf>,
    view_input_file: Option<PathBuf>,
    file_list: Arc<Mutex<Vec<FileInfo>>>,
}

#[derive(Clone)]
struct FileInfo {
    path: String,
    size: u64,
}

#[derive(PartialEq)]
enum Tab { Pack, ViewAndExtract }

impl Default for YaamApp {
    fn default() -> Self {
        Self {
            selected_tab: Tab::Pack,
            status_message: Arc::new(Mutex::new("Ready.".to_owned())),
            is_working: Arc::new(Mutex::new(false)),
            pack_input_folder: None,
            view_input_file: None,
            file_list: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl eframe::App for YaamApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.add_space(5.0);
            ui.horizontal(|ui| {
                ui.heading("YAAM");
                ui.add_space(20.0);
                if ui.selectable_label(self.selected_tab == Tab::Pack, "📦 Pack").clicked() {
                    self.selected_tab = Tab::Pack;
                }
                if ui.selectable_label(self.selected_tab == Tab::ViewAndExtract, "👁 View & Extract").clicked() {
                    self.selected_tab = Tab::ViewAndExtract;
                }
            });
            ui.add_space(5.0);
        });

        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let status = self.status_message.lock().unwrap();
                ui.label(status.as_str());
                if *self.is_working.lock().unwrap() {
                    ui.spinner();
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            match self.selected_tab {
                Tab::Pack => self.render_pack_tab(ui),
                Tab::ViewAndExtract => self.render_view_tab(ui),
            }
        });
    }
}

impl YaamApp {
    fn render_pack_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("Create Archive");
        ui.add_space(10.0);

        let is_busy = *self.is_working.lock().unwrap();

        ui.group(|ui| {
            ui.label("1. Select a folder to compress:");
            if ui.button("📂 Browse Folders...").clicked() && !is_busy {
                if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                    self.pack_input_folder = Some(folder);
                }
            }
            if let Some(path) = &self.pack_input_folder {
                ui.monospace(format!("{}", path.display()));
            }
        });

        ui.add_space(10.0);

        if ui.add_enabled(self.pack_input_folder.is_some() && !is_busy,
            egui::Button::new("🚀 COMPRESS TO .YAAM").min_size(egui::vec2(150.0, 40.0))
        ).clicked() {
            self.start_packing();
        }
    }

    fn render_view_tab(&mut self, ui: &mut egui::Ui) {
        let is_busy = *self.is_working.lock().unwrap();

        ui.horizontal(|ui| {
            if ui.button("📂 Open Archive...").clicked() && !is_busy {
                if let Some(file) = rfd::FileDialog::new()
                    .add_filter("Archives", &[FILE_EXT, "zip", "7z"])
                    .pick_file()
                {
                    self.view_input_file = Some(file);
                    self.start_listing();
                }
            }
            if let Some(path) = &self.view_input_file {
                ui.label(path.file_name().unwrap().to_string_lossy());
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.add_enabled(self.view_input_file.is_some() && !is_busy, egui::Button::new("⬇ Extract All")).clicked() {
                    self.start_extracting();
                }
            });
        });

        ui.separator();

        let file_list = self.file_list.lock().unwrap();

        if file_list.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label("Open an archive to view contents.");
            });
        } else {
            TableBuilder::new(ui)
                .striped(true)
                .column(Column::remainder().resizable(true))
                .column(Column::exact(100.0))
                .header(20.0, |mut header| {
                    header.col(|ui| { ui.strong("File Name"); });
                    header.col(|ui| { ui.strong("Size"); });
                })
                .body(|mut body| {
                    for file in file_list.iter() {
                        body.row(18.0, |mut row| {
                            row.col(|ui| { ui.label(&file.path); });
                            row.col(|ui| { ui.label(format_size(file.size)); });
                        });
                    }
                });
        }
    }

    fn start_packing(&self) {
        let folder = self.pack_input_folder.clone().unwrap();
        let status = self.status_message.clone();
        let busy = self.is_working.clone();

        *busy.lock().unwrap() = true;
        thread::spawn(move || {
            let name = folder.file_name().unwrap().to_string_lossy();
            let out = folder.parent().unwrap().join(format!("{}.{}", name, FILE_EXT));

            match pack_yaam(&folder, &out) {
                Ok(_) => *status.lock().unwrap() = "Packed successfully!".to_string(),
                Err(e) => *status.lock().unwrap() = format!("Error: {}", e),
            }
            *busy.lock().unwrap() = false;
        });
    }

    fn start_listing(&self) {
        if self.view_input_file.is_none() { return; }

        let file = self.view_input_file.clone().unwrap();
        let list_store = self.file_list.clone();
        let busy = self.is_working.clone();

        *busy.lock().unwrap() = true;
        thread::spawn(move || {
            let res = list_archive_contents(&file);
            match res {
                Ok(files) => *list_store.lock().unwrap() = files,
                Err(_) => *list_store.lock().unwrap() = vec![],
            }
            *busy.lock().unwrap() = false;
        });
    }

    fn start_extracting(&self) {
        let file = self.view_input_file.clone().unwrap();
        let status = self.status_message.clone();
        let busy = self.is_working.clone();

        *busy.lock().unwrap() = true;
        thread::spawn(move || {
            let stem = file.file_stem().unwrap();
            let out_dir = file.parent().unwrap().join(stem);

            let res = if let Some(ext) = file.extension().and_then(|s| s.to_str()) {
                match ext {
                    "zip" => extract_zip(&file, &out_dir),
                    "7z" => extract_7z(&file, &out_dir),
                    _ => extract_yaam(&file, &out_dir),
                }
            } else {
                extract_yaam(&file, &out_dir)
            };

            match res {
                Ok(_) => *status.lock().unwrap() = "Extracted successfully!".to_string(),
                Err(e) => *status.lock().unwrap() = format!("Error: {}", e),
            }
            *busy.lock().unwrap() = false;
        });
    }
}


fn format_size(bytes: u64) -> String {
    if bytes < 1024 { format!("{} B", bytes) }
    else if bytes < 1024 * 1024 { format!("{:.1} KB", bytes as f64 / 1024.0) }
    else { format!("{:.1} MB", bytes as f64 / 1024.0 / 1024.0) }
}

struct XorWriter<W: Write> { inner: W }
impl<W: Write> Write for XorWriter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let s: Vec<u8> = buf.iter().map(|&b| b ^ SCRAMBLE_KEY).collect();
        self.inner.write(&s)
    }
    fn flush(&mut self) -> io::Result<()> { self.inner.flush() }
}
struct XorReader<R: Read> { inner: R }
impl<R: Read> Read for XorReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let n = self.inner.read(buf)?;
        for i in 0..n { buf[i] ^= SCRAMBLE_KEY; }
        Ok(n)
    }
}

fn pack_yaam(folder: &Path, output: &Path) -> anyhow::Result<()> {
    let mut file = File::create(output)?;
    file.write_all(MAGIC_HEADER)?;
    let scrambler = XorWriter { inner: file };
    let encoder = zstd::stream::Encoder::new(scrambler, 3)?;
    let mut tar = tar::Builder::new(encoder);
    tar.append_dir_all(".", folder)?;
    tar.into_inner()?.finish()?;
    Ok(())
}

fn extract_yaam(input: &Path, output: &Path) -> anyhow::Result<()> {
    let mut file = File::open(input)?;
    let mut buf = [0u8; 4];
    file.read_exact(&mut buf)?;
    if buf != MAGIC_HEADER { anyhow::bail!("Invalid YAAM file"); }
    let descrambler = XorReader { inner: file };
    let decoder = zstd::stream::Decoder::new(descrambler)?;
    let mut archive = tar::Archive::new(decoder);
    archive.unpack(output)?;
    Ok(())
}

fn extract_zip(input: &Path, output: &Path) -> anyhow::Result<()> {
    let file = File::open(input)?;
    let mut archive = zip::ZipArchive::new(file)?;
    archive.extract(output)?;
    Ok(())
}

fn extract_7z(input: &Path, output: &Path) -> anyhow::Result<()> {
    sevenz_rust::decompress_file(input, output)?;
    Ok(())
}

fn list_archive_contents(path: &Path) -> anyhow::Result<Vec<FileInfo>> {
    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
    let mut files = Vec::new();

    if ext == "zip" {
        let f = File::open(path)?;
        let mut archive = zip::ZipArchive::new(f)?;
        for i in 0..archive.len() {
            let file = archive.by_index(i)?;
            files.push(FileInfo {
                path: file.name().to_string(),
                size: file.size()
            });
        }
    } else if ext == "7z" {
        files.push(FileInfo { path: "Scanning 7z not fully supported yet".into(), size: 0 });
    } else {
        let mut file = File::open(path)?;
        let mut buf = [0u8; 4];
        file.read_exact(&mut buf)?;
        if buf == MAGIC_HEADER {
            let descrambler = XorReader { inner: file };
            let decoder = zstd::stream::Decoder::new(descrambler)?;
            let mut archive = tar::Archive::new(decoder);

            for entry in archive.entries()? {
                let entry = entry?;
                files.push(FileInfo {
                    path: entry.path()?.to_string_lossy().to_string(),
                    size: entry.header().size().unwrap_or(0),
                });
            }
        }
    }
    Ok(files)
}