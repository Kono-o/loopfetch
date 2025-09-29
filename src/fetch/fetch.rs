use crate::app::SETTINGS;
use katatui::mlua;
use katatui::mlua::prelude::{LuaResult, LuaTable};
use katatui::*;
use libmacchina::{
   traits::{GeneralReadout as _, ProductReadout as _, ShellFormat, ShellKind},
   GeneralReadout, ProductReadout,
};
use mpris::{Player, PlayerFinder};
use nvml_wrapper::{
   enum_wrappers::device::{Clock, TemperatureSensor},
   Nvml,
};
use std::time::Duration;
use std::{fs, path::Path};
use sysinfo::{Disks, System};

#[derive(Debug, Default)]
pub struct Disk {
   pub mnt: String,
   pub name: String,
   pub mem: Mem,
}

#[derive(Debug, Default)]
pub struct Mem {
   pub avail: u64,
   pub total: u64,
}

impl Mem {
   fn from(avail: u64, total: u64) -> Self {
      Self { avail, total }
   }
}

#[derive(Debug)]
pub struct Media {
   pub player: Player,
   pub name: String,
   pub song: String,
   pub artist: String,
   pub album: String,
   pub art_url: String,
   pub elapsed: Duration,
   pub length: Duration,
   pub paused: bool,
}

pub struct Info {
   pub gen_read: GeneralReadout,
   pub prod_read: ProductReadout,
   pub sys: System,
   pub sys_disks: Disks,
   pub player: Option<PlayerFinder>,

   pub user: String,
   pub host: String,
   pub device: String,
   pub bios: String,
   pub uptime: u64,
   pub os_n: String,
   pub os_v: String,
   pub kern: String,
   pub log_m: String,
   pub desk_e: Option<String>,
   pub win_m: String,
   pub win_p: String,
   pub comp: String,
   pub term: String,
   pub shell: String,
   pub text_e: String,
   pub cpu_n: String,
   pub cpu_c: u8,
   pub cpu_u: u8,
   pub cpu_t: f32,
   pub ram: Mem,
   pub gpu_n: String,
   pub gpu_f: f32,
   pub gpu_t: f32,
   pub vram: Mem,
   pub disks: Vec<Disk>,
   pub media: Vec<Media>,
}

pub trait VecMedia {
   fn active(&self) -> Option<usize>;
}

impl VecMedia for Vec<Media> {
   fn active(&self) -> Option<usize> {
      if self.is_empty() {
         return None;
      }
      let precedence = ["spotify", "vlc", "mpv", "rhythmbox", "firefox", "chrome"];
      let mut best_idx = 0;
      let mut best_priority = precedence.len();

      for (i, media) in self.iter().enumerate() {
         let player_name = &media.name;
         for (p_idx, &pref) in precedence.iter().enumerate() {
            if player_name.contains(pref) && p_idx < best_priority {
               best_idx = i;
               best_priority = p_idx;
               break;
            }
         }
      }
      Some(best_idx)
   }
}

impl Info {
   pub fn fetch(settings: &SETTINGS) -> Self {
      let gen_read = GeneralReadout::new();
      let prod_read = ProductReadout::new();
      let mut sys = System::new_all();
      let mut sys_disks = Disks::new_with_refreshed_list();
      let player = match PlayerFinder::new() {
         Ok(pf) => Some(pf),
         _ => None,
      };
      let (user, host, uptime) = get_user_host_uptime(&gen_read);
      let (device, bios) = get_dev_bios(&prod_read);
      let (os_n, os_v, kern) = get_os_kern();
      let (log_m, win_m, win_p, desk_e) = get_managers(&gen_read);
      let (term, shell, text_e) = get_tools(&gen_read);
      let (cpu_n, cpu_c) = get_cpu(&gen_read);
      let (cpu_u, cpu_t, ram) = get_cpu_stats(&gen_read, &sys);
      let (gpu_n, gpu_f, gpu_t, vram) = get_gpu_stats();
      let disks = get_disks(&mut sys_disks);
      let media = get_media(&player);
      let comp = settings.vars().comp().to_string();

      Self {
         gen_read,
         prod_read,
         sys,
         sys_disks,
         player,
         user,
         host,
         device,
         bios,
         uptime,
         os_n,
         os_v,
         kern,
         log_m,
         desk_e,
         win_m,
         win_p,
         comp,
         term,
         shell,
         text_e,
         cpu_n,
         cpu_c,
         cpu_u,
         cpu_t,
         ram,
         gpu_n,
         gpu_f,
         gpu_t,
         vram,
         disks,
         media,
      }
   }

   pub fn refresh(&mut self, settings: &SETTINGS) {
      (self.user, self.host, self.uptime) = get_user_host_uptime(&self.gen_read);
      (self.term, self.shell, self.text_e) = get_tools(&self.gen_read);
      (self.cpu_u, self.cpu_t, self.ram) = get_cpu_stats(&self.gen_read, &self.sys);
      (self.gpu_n, self.gpu_f, self.gpu_t, self.vram) = get_gpu_stats();
      self.disks = get_disks(&mut self.sys_disks);
      self.media = get_media(&self.player);
      self.comp = settings.vars().comp().to_string();
   }

   pub fn to_lua(&self, lua: &mlua::Lua) -> LuaResult<LuaTable> {
      let table = lua.create_table()?;
      table.set("user", &*self.user)?;
      table.set("host", &*self.host)?;
      table.set("device", &*self.device)?;
      table.set("bios", &*self.bios)?;
      table.set("uptime", self.uptime)?;
      table.set("os_n", &*self.os_n)?;
      table.set("os_v", &*self.os_v)?;
      table.set("kern", &*self.kern)?;
      table.set("log_m", &*self.log_m)?;
      table.set("desk_e", self.desk_e.clone())?;
      table.set("win_m", &*self.win_m)?;
      table.set("win_p", &*self.win_p)?;
      table.set("comp", &*self.comp)?;
      table.set("term", &*self.term)?;
      table.set("shell", &*self.shell)?;
      table.set("text_e", &*self.text_e)?;
      table.set("cpu_n", &*self.cpu_n)?;
      table.set("cpu_c", self.cpu_c)?;
      table.set("cpu_u", self.cpu_u)?;
      table.set("cpu_t", self.cpu_t)?;

      let ram = lua.create_table()?;
      ram.set("avail", self.ram.avail)?;
      ram.set("total", self.ram.total)?;
      table.set("ram", ram)?;

      table.set("gpu_n", &*self.gpu_n)?;
      table.set("gpu_f", self.gpu_f)?;
      table.set("gpu_t", self.gpu_t)?;

      let vram = lua.create_table()?;
      vram.set("avail", self.vram.avail)?;
      vram.set("total", self.vram.total)?;
      table.set("vram", vram)?;

      let disks = lua.create_table()?;
      for (i, d) in self.disks.iter().enumerate() {
         let disk = lua.create_table()?;
         disk.set("mnt", &*d.mnt)?;
         disk.set("name", &*d.name)?;
         let mem = lua.create_table()?;
         mem.set("avail", d.mem.avail)?;
         mem.set("total", d.mem.total)?;
         disk.set("mem", mem)?;
         disks.set(i + 1, disk)?;
      }
      table.set("disks", disks)?;

      let media_list = lua.create_table()?;
      for (i, m) in self.media.iter().enumerate() {
         let media = lua.create_table()?;
         media.set("name", &*m.name)?;
         media.set("song", &*m.song)?;
         media.set("artist", &*m.artist)?;
         media.set("album", &*m.album)?;
         media.set("art_url", &*m.art_url)?;
         media.set("elapsed", m.elapsed.as_secs())?;
         media.set("length", m.length.as_secs())?;
         media.set("paused", m.paused)?;
         media_list.set(i + 1, media)?;
      }
      table.set("media", media_list)?;
      Ok(table)
   }
}

const DEFAULT: &str = "unknown";
const COMPOSITOR: &str = "picom";

fn get_user_host_uptime(gen_read: &GeneralReadout) -> (String, String, u64) {
   let user = get_env("USER", "user").to_lowercase();
   let host = gen_read.hostname().unwrap_or("host".into()).to_lowercase();
   let uptime = gen_read.uptime().unwrap_or(0) as u64;
   (user, host, uptime)
}

fn get_dev_bios(prod_read: &ProductReadout) -> (String, String) {
   let device = prod_read.product().unwrap_or(DEFAULT.into()).to_uppercase();
   let bios = if Path::new("/sys/firmware/efi").exists() {
      "UEFI"
   } else {
      "BIOS"
   }
   .to_string();
   (device, bios)
}

fn get_os_kern() -> (String, String, String) {
   let mut os_n = System::name().unwrap_or(DEFAULT.into()).to_lowercase();
   let os_v = System::os_version().unwrap_or(DEFAULT.into());
   let kern = System::kernel_version().unwrap_or(DEFAULT.into());
   if os_n.contains("linux") {
      os_n = os_n.replace("linux", "").trim().to_string();
   }
   (os_n, os_v, kern)
}

fn get_managers(gen_read: &GeneralReadout) -> (String, String, String, Option<String>) {
   let log_path = "/etc/systemd/system/display-manager.service";
   let log_m = fs::read_link(log_path)
      .ok()
      .and_then(|p| p.file_name().map(|s| s.to_string_lossy().into_owned()))
      .map(|name| name.trim_end_matches(".service").to_string())
      .unwrap_or_else(|| DEFAULT.into());
   let win_m = gen_read
      .window_manager()
      .unwrap_or(DEFAULT.into())
      .to_lowercase();
   let win_p = gen_read.session().unwrap_or(DEFAULT.into()).to_lowercase();
   let desk_e = match gen_read.desktop_environment() {
      Ok(mut de) => {
         de = de.to_lowercase();
         match de != win_m {
            true => Some(de),
            _ => None,
         }
      }
      Err(_) => None,
   };
   (log_m, win_m, win_p, desk_e)
}

fn get_tools(gen_read: &GeneralReadout) -> (String, String, String) {
   let term = gen_read.terminal().unwrap_or(DEFAULT.into()).to_lowercase();
   let term = (&term[0..term.len() - 1]).to_string();
   let shell = gen_read
      .shell(ShellFormat::Relative, ShellKind::Default)
      .unwrap_or(DEFAULT.into())
      .to_lowercase();
   let text_e = get_env("EDITOR", "none").to_lowercase();
   (term, shell, text_e)
}

fn get_env(var: &str, default: &str) -> String {
   std::env::var(var.to_string()).unwrap_or(default.into())
}

fn get_cpu(gen_read: &GeneralReadout) -> (String, u8) {
   let raw = gen_read.cpu_model_name().unwrap_or(DEFAULT.into());
   let mut s = raw.to_string();
   for pat in ["(R)", "(TM)", "CPU", "Processor"] {
      s = s.replace(pat, "");
   }
   for v in ["Intel", "AMD", "Apple"] {
      s = s.replace(v, "");
   }
   if let Some(pos) = s.find('@') {
      s = s[..pos].to_string();
   }
   let parts: Vec<&str> = s
      .split_whitespace()
      .filter(|w| {
         let lw = w.to_lowercase();
         !(lw.ends_with("-core") || lw == "core")
      })
      .collect();
   let cpu_n = parts.join(" ").trim().to_string().to_lowercase();
   let cpu_c = gen_read.cpu_cores().unwrap() as u8;
   (cpu_n, cpu_c)
}

fn get_cpu_stats(gen_read: &GeneralReadout, sys: &System) -> (u8, f32, Mem) {
   let cpu_u = gen_read.cpu_usage().unwrap_or(0) as u8;
   let mut cpu_t: f32 = 0.0;
   let ram = Mem::from(sys.used_memory(), sys.total_memory());

   let hwmon = "/sys/class/hwmon/";
   if let Ok(entries) = fs::read_dir(hwmon) {
      for entry in entries.flatten() {
         let path = entry.path();
         if let Ok(name) = fs::read_to_string(path.join("name")) {
            if name.contains("coretemp") || name.contains("k10temp") || name.contains("zenpower") {
               if let Ok(val) = fs::read_to_string(path.join("temp1_input")) {
                  if let Ok(milli_c) = val.trim().parse::<f32>() {
                     cpu_t = milli_c / 1000.0;
                     break;
                  }
               }
            }
         }
      }
   }
   (cpu_u, cpu_t, ram)
}

fn get_gpu_stats() -> (String, f32, f32, Mem) {
   let nvml = Nvml::init().unwrap();
   let gpu = nvml.device_by_index(0).unwrap();
   let gpu_f = gpu.clock_info(Clock::Graphics).unwrap_or(0) as f32;
   let gpu_t = gpu.temperature(TemperatureSensor::Gpu).unwrap_or(0) as f32;
   let vram = gpu.memory_info().unwrap();
   let vram = Mem::from(vram.used, vram.total);

   let raw = gpu.name().unwrap_or(DEFAULT.into());
   let mut s = raw.to_string();
   for pat in [
      "NVIDIA", "GeForce", "AMD", "Radeon", "Intel", "Graphics", "Series", "Laptop", "GPU", "(R)",
      "(TM)",
   ] {
      s = s.replace(pat, "");
   }
   let parts: Vec<&str> = s.split_whitespace().filter(|p| !p.is_empty()).collect();
   let mut name: Vec<String> = Vec::new();
   for part in parts {
      if let Some(prev) = name.last_mut() {
         if prev.chars().all(|c| c.is_ascii_digit())
            && part.chars().all(|c| c.is_ascii_alphabetic())
         {
            prev.push_str(part);
            continue;
         }
      }
      name.push(part.to_string());
   }
   let gpu_n = name.join(" ").to_lowercase();
   (gpu_n, gpu_f, gpu_t, vram)
}

fn get_disks(sys_disks: &mut Disks) -> Vec<Disk> {
   sys_disks.refresh(true);
   let mut disks = Vec::new();
   for disk in sys_disks {
      let mut mnt_p = disk.mount_point();
      let mnt = format!("{}", mnt_p.display());
      let skips = [
         "/run", "/boot", "/dev", "/proc", "/sys", "/tmp", "/var", "/snap",
      ];
      let mut plz_skip = false;
      for skip in skips {
         if mnt.starts_with(skip) {
            plz_skip = true;
         }
      }
      let name = if mnt_p == Path::new("/") {
         "root".to_string()
      } else {
         format!(
            "{}",
            mnt_p.file_name().unwrap_or("unknown".as_ref()).display()
         )
      };
      if plz_skip {
         continue;
      }
      disks.push(Disk {
         mnt,
         name,
         mem: Mem::from(disk.available_space(), disk.total_space()),
      });
   }
   disks
}

fn get_media(pf: &Option<PlayerFinder>) -> Vec<Media> {
   let players = match pf {
      Some(pf) => match pf.find_all() {
         Ok(a) => Some(a),
         Err(_) => None,
      },
      _ => None,
   };
   let mut medias = Vec::new();
   match players {
      Some(vp) => {
         for player in vp {
            let m = player.get_metadata();
            match m {
               Ok(m) => {
                  let song = m.title().unwrap_or(DEFAULT).to_string();
                  let artist = m
                     .artists()
                     .and_then(|a| {
                        if a.is_empty() {
                           None
                        } else {
                           Some(a.join(", "))
                        }
                     })
                     .unwrap_or(DEFAULT.into());
                  let name = player.identity().to_lowercase();
                  let album = m.album_name().unwrap_or(DEFAULT).to_string();
                  let art_url = m.art_url().unwrap_or(DEFAULT).to_string();
                  let length = m.length().unwrap_or(Duration::from_micros(0));
                  let elapsed = player.get_position().unwrap_or(Duration::from_micros(0));
                  let paused = player
                     .get_playback_status()
                     .map(|s| s == mpris::PlaybackStatus::Paused)
                     .unwrap_or(true);
                  let media = Media {
                     player,
                     name,
                     song,
                     artist,
                     album,
                     art_url,
                     elapsed,
                     length,
                     paused,
                  };
                  medias.push(media);
               }
               _ => {}
            }
         }
      }
      _ => {}
   }
   medias
}
