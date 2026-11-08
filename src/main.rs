#![windows_subsystem = "windows"] // Menghilangkan layar console (cmd) hitam saat exe dibuka

use eframe::egui;
use ini::Ini;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Default)]
struct ApiResponse {
    schedules: HashMap<String, String>,
}

struct SalatData {
    nama: String,
    waktu: String,
    jumlah: i32,
}

struct QadhaTracker {
    daftar_salat: Vec<SalatData>,
    ini_file: String,
    kota_list: Vec<(String, String)>,
    selected_kota_idx: usize,
    is_setup: bool,
}

impl QadhaTracker {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let ini_file = "data_qadha.ini".to_string();
        
        // Membaca isi daftar_kota.md saat compile.
        let raw_kota = include_str!("../daftar_kota.md");
        let mut kota_list = Vec::new();
        
        // Parsing string manual lebih efisien dan ringan untuk ukuran .exe dibanding memakai library Regex eksternal
        for line in raw_kota.lines() {
            if let (Some(start), Some(mid), Some(end)) = (line.find("- "), line.find(" (Value: "), line.find(")")) {
                let nama = line[start + 2..mid].trim().to_string();
                let kode = line[mid + 9..end].trim().to_string();
                kota_list.push((nama, kode));
            }
        }

        let mut app = Self {
            daftar_salat: Vec::new(),
            ini_file: ini_file.clone(),
            kota_list,
            selected_kota_idx: 0,
            is_setup: false,
        };

        // Jika file INI dan kode area sudah ada, langsung fetch API dan masuk mode Tracker
        let conf = Ini::load_from_file(&ini_file).unwrap_or_else(|_| Ini::new());
        if let Some(area_code) = conf.get_from(Some("Area"), "kode") {
            app.load_api_data(area_code);
            app.is_setup = true;
        }

        app
    }

    // Dipisah dari 'new' agar bisa dipanggil saat tombol "Simpan" di UI Setup diklik
    fn load_api_data(&mut self, area_code: &str) {
        let conf = Ini::load_from_file(&self.ini_file).unwrap_or_else(|_| Ini::new());
        let url = format!("https://jadwalsholat.org/adzan/ajax/next/ajax.daily.php?id={}", area_code);
        let schedules = ureq::get(&url).call().ok()
            .and_then(|r| r.into_json::<ApiResponse>().ok())
            .map(|res| res.schedules)
            .unwrap_or_default();

        let pemetaan = [
            ("Subuh", "shubuh"),
            ("Dzuhur", "dzuhur"),
            ("Ashar", "ashr"),
            ("Maghrib", "maghrib"),
            ("Isya", "isya"),
        ];

        self.daftar_salat.clear();
        for (nama, json_key) in pemetaan {
            let jumlah: i32 = conf.get_from(Some("Jumlah"), nama)
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
            
            let waktu = schedules.get(json_key).cloned().unwrap_or_else(|| "--:--".to_string());

            self.daftar_salat.push(SalatData {
                nama: nama.to_string(),
                waktu,
                jumlah,
            });
        }
    }

    fn simpan_qadha(&self) {
        let mut conf = Ini::load_from_file(&self.ini_file).unwrap_or_else(|_| Ini::new());
        for salat in &self.daftar_salat {
            conf.with_section(Some("Jumlah"))
                .set(&salat.nama, salat.jumlah.to_string());
        }
        let _ = conf.write_to_file(&self.ini_file);
    }
}

impl eframe::App for QadhaTracker {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Pencatat Qadha");
            ui.separator();

            // --- KONDISI 1: UI Pilih Kota (Jika belum ada data_qadha.ini) ---
            if !self.is_setup {
                ui.label("Pilih Kota Anda:");
                ui.add_space(5.0);
                
                let selected_nama = self.kota_list[self.selected_kota_idx].0.clone();
                
                // Dropdown / ComboBox
                egui::ComboBox::from_id_source("pilih_kota")
                    .selected_text(selected_nama)
                    .width(ui.available_width())
                    .show_ui(ui, |ui| {
                        for (idx, (nama, _)) in self.kota_list.iter().enumerate() {
                            ui.selectable_value(&mut self.selected_kota_idx, idx, nama);
                        }
                    });

                ui.add_space(10.0);
                
                if ui.button("Simpan & Lanjutkan").clicked() {
                    let kode = &self.kota_list[self.selected_kota_idx].1.clone();
                    
                    // Simpan kode kota ke file INI
                    let mut conf = Ini::new();
                    conf.with_section(Some("Area")).set("kode", kode);
                    let _ = conf.write_to_file(&self.ini_file);
                    
                    // Ambil jadwal salat berdasarkan kota lalu ubah tampilan ke Tracker
                    self.load_api_data(kode);
                    self.is_setup = true;
                }
                return; // Stop eksekusi agar UI tracker di bawah tidak ikut tergambar
            }

            // --- KONDISI 2: UI Tracker Qadha (Berjalan normal jika file sudah disetup) ---
            let mut ada_perubahan = false;

            for salat in &mut self.daftar_salat {
                ui.horizontal(|ui| {
                    ui.label(&salat.waktu);
                    ui.add_space(10.0);
                    ui.label(&salat.nama);
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("+").clicked() {
                            salat.jumlah += 1;
                            ada_perubahan = true;
                        }
                        
                        ui.label(salat.jumlah.to_string());
                        
                        if ui.button("-").clicked() && salat.jumlah > 0 {
                            salat.jumlah -= 1;
                            ada_perubahan = true;
                        }
                    });
                });
            }

            if ada_perubahan {
                self.simpan_qadha();
            }
        });
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([250.0, 220.0])
            .with_always_on_top(),
        ..Default::default()
    };
    
    eframe::run_native(
        "QadhaTracker",
        options,
        Box::new(|cc| Box::new(QadhaTracker::new(cc))),
    )
}