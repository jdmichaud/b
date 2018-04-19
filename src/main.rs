#![allow(dead_code)]
#![allow(unused_variables)]
#![feature(path_ancestors)]

#[macro_use] extern crate log;
extern crate simplelog;
extern crate pancurses;
extern crate ncurses;
extern crate itertools;
extern crate chrono;

use std::fs;
use std::path;
use std::fs::DirEntry;
use std::fs::File;
use std::collections::HashMap;
use std::cmp::{min, max};
use std::io::Error;
use itertools::Itertools;
use pancurses::{Input, Attribute, ColorPair, chtype};
use simplelog::{WriteLogger, LevelFilter, Config};
use chrono::offset::Utc;
use chrono::DateTime;

#[derive(PartialEq)]
enum Mode {
  Browsing,
  Roaming,
  Command,
}

struct Model {
  entries: Vec<DirEntry>,
  pointed: usize,
  first: usize,
  mode: Mode,
  cwd: String,
  error: Option<Error>,
  color_scheme: HashMap<String, Vec<u8>>,
  show_hidden: bool,
  show_detail: bool,
  roam_path: String,
  no_match: bool,
  escaped: bool,
  cursor_shift: usize,
  selected_buffer: String,
}

fn format_size(len: u64) -> String {
  if len < 1024 {
    len.to_string() + "B"
  } else if len < 1024 * 1024 {
    format!("{:.2}KB", len as f32 / 1024f32)
  } else if len < 1024 * 1024 * 1024 {
    format!("{:.2}MB", len as f32 / (1024f32 * 1024f32))
  } else if len < 1024 * 1024 * 1024 * 1024 {
    format!("{:.2}GB", len as f32 / (1024f32 * 1024f32 * 1024f32))
  } else if len < 1024 * 1024 * 1024 * 1024 * 1024 {
    format!("{:.2}TB", len as f32 / (1024f32 * 1024f32 * 1024f32 * 1024f32))
  } else {
    "Too big!".to_string()
  }
}

fn display_entry(window: &mut pancurses::Window, model: &Model, entry: &DirEntry) {
  if model.show_detail {
    let path = entry.path();
    let filename = path.as_path().file_name().unwrap().to_str().unwrap();
    let metadata = match entry.metadata() {
      Ok(m) => m,
      Err(error) => {
        error!("There was an error while display the entry: {:?}", path);
        return;
      }
    };
    let size = if path.is_dir() { "/".to_string() } else { format_size(metadata.len()) };
    let last_modified: DateTime<Utc> = metadata.modified().unwrap().into();
    window.printw(&format!("{} {:>10} {}", last_modified.format("%Y-%m-%d %T"), size, filename));
  }
  else {
    window.printw(entry.path().as_path().file_name().unwrap().to_str().unwrap());
  }
}

fn display_list(window: &mut pancurses::Window, model: &Model) {
  if model.mode == Mode::Roaming && model.no_match {
    window.mv(2, 3);
    window.printw("** no match **");
  } else {
    (&model.entries).into_iter()
      .skip(model.first)
      .take(get_height(window))
      .enumerate()
      .for_each(|(index, entry)| {
        let mut attrs: chtype = 0;
        if entry.path().is_dir() {
          match model.color_scheme.get("di") {
            Some(color) if color.len() == 2 => {
              attrs = attrs | chtype::from(ColorPair(color[1]) | Attribute::Bold);
            },
            Some(_) | None => info!("no color for directory"),
          }
        }
        window.attrset(attrs);
        match model.mode {
          Mode::Browsing => {
            if index == model.pointed - model.first {
              attrs = attrs | chtype::from(Attribute::Reverse | Attribute::Bold);
              window.attrset(attrs);
              window.mv(index as i32 + 2, 0);
              window.printw(" > ");
            }
          },
          Mode::Roaming => {
            if index == model.pointed - model.first {
              attrs = attrs | chtype::from(Attribute::Reverse | Attribute::Bold);
              window.attrset(attrs);
              window.mv(index as i32 + 2, 0);
              window.printw(" > ");
            }
          },
          _ => (),
        };
        window.mv(index as i32 + 2, 3);
        display_entry(window, model, &entry);
        window.attrset(Attribute::Normal | ColorPair(0));
      });
  }
}

fn display_input_line(window: &mut pancurses::Window, model: &Model) {
  window.mv(window.get_max_y() - 1, 0);
  match model.mode {
    Mode::Roaming => {
      pancurses::curs_set(1);
      window.printw(&format!("{}", model.roam_path));
      window.mv(window.get_max_y() - 1,
        (model.roam_path.len() - model.cursor_shift) as i32);
    }
    _ => {
      pancurses::curs_set(0);
      match &model.error {
        &Some(ref error) => { window.printw(&format!("error: {}", error)); },
        &None => ()
      }
    }
  };
}

fn display(window: &mut pancurses::Window, model: &Model) {
  window.clear();
  window.mv(0, 0);
  window.printw(&format!("DIR: {}", model.cwd));
  display_list(window, model);
  display_input_line(window, model);
}

fn update_model_from_dir(model: &mut Model, path: Option<String>) -> Result<(), ()> {
  let new_path = match path {
    Some(p) => p,
    None => model.cwd.clone(),
  };
  match fs::read_dir(new_path) {
    Ok(entries) => { 
      model.entries = entries
        .map(|entry| entry.unwrap())
        .filter(|entry| !(model.show_hidden == false && entry.file_name().to_str().unwrap().starts_with(".")))
        .collect::<Vec<DirEntry>>();
      model.entries.sort_by(|a, b| a.file_name().to_str().unwrap().cmp(b.file_name().to_str().unwrap()));
      Ok(())
    },
    Err(error) => {
      model.error = Some(error);
      Err(())
    },
  }
}

fn change_cwd(model: &mut Model, path: String) -> Result<(), ()> {
  if model.cwd.len() == 0 { model.cwd = "/".to_string() }
  let res = update_model_from_dir(model, Some(path.clone()));
  match res {
    Ok(()) => {
      model.cwd = path;
      res
    },
    _ => res
  }
}

fn get_height(window: &pancurses::Window) -> usize {
  return window.get_max_y().saturating_sub(3) as usize;
}

fn get_colors_db() -> HashMap<String, Vec<u8>> {
  match std::env::var_os("LS_COLORS") {
    Some(val) => {
      pancurses::init_pair(30, pancurses::COLOR_BLACK, -1);
      pancurses::init_pair(31, pancurses::COLOR_RED, -1);
      pancurses::init_pair(32, pancurses::COLOR_GREEN, -1);
      pancurses::init_pair(33, pancurses::COLOR_YELLOW, -1);
      pancurses::init_pair(34, pancurses::COLOR_BLUE, -1);
      pancurses::init_pair(35, pancurses::COLOR_MAGENTA, -1);
      pancurses::init_pair(36, pancurses::COLOR_CYAN, -1);
      pancurses::init_pair(37, pancurses::COLOR_WHITE, -1);
      val.to_str().unwrap()
        .split(":")
        .flat_map(|entry| entry.split("="))
        .map(|s| String::from(s))
        .tuples()
        .map(|(code, colors)| (code, colors.split(";").map(|s| s.parse::<u8>().unwrap()).collect::<Vec<u8>>()))
        .collect()
    },
    None => HashMap::new()
  }
}

fn roam_model(mut model: &mut Model) {
  let roam_path = model.roam_path.clone();
  let path = path::Path::new(&roam_path);
  let s = String::from("test");
  let valid_parent = match path.ancestors().find(|p| p.is_dir()) {
    Some(p) => p,
    None => path::Path::new("/"),
  };
  let mut valid_size = valid_parent.to_str().unwrap().len();
  if valid_parent.to_str().unwrap().starts_with(path::MAIN_SEPARATOR) { valid_size += 1 }
  change_cwd(&mut model, valid_parent.to_str().unwrap().to_string()).unwrap();

  model.no_match = false;
  model.pointed = 0; // To improve: we shall keep track of our selection.
  if valid_size < model.roam_path.len() {
    let rest = &model.roam_path.clone()[valid_size..];
    let previous_len = model.entries.len();
    model.entries.retain(|e| e.file_name().to_str().unwrap().starts_with(rest));
    if previous_len > 0 && model.entries.len() == 0 {
      model.no_match = true;
    }
  }
}

fn change_cwd_to_pointed(mut model: &mut Model) {
  if model.entries.len() > model.pointed {
    let new_path = String::from(fs::canonicalize(model.entries[model.pointed].path())
      .unwrap()
      .to_str()
      .unwrap());
    match change_cwd(&mut model, new_path) {
      Ok(()) => {
        model.pointed = 0;
        model.first = 0;
        ()
      },
      _ => ()
    }
  }
}

fn scroll_down(window: &pancurses::Window, model: &mut Model) {
  model.pointed = min(
    min(window.get_max_y().saturating_sub(1) as usize, model.entries.len().saturating_sub(1)),
    model.pointed.saturating_add(1));
  model.first = max(0, model.pointed.saturating_sub(get_height(&window) - 1));
}

fn scroll_up(model: &mut Model) {
  model.pointed = max(0, model.pointed.saturating_sub(1));
  model.first = min(model.first, model.pointed);
}

fn browsing_mode(c: Option<Input>, mut window: &mut pancurses::Window, mut model: &mut Model) {
  match c {
    Some(Input::Character('\u{1b}')) |
    Some(Input::Character('q')) => { // Escape key
      pancurses::endwin();
      // Copy selected to the clipboard
      // Exit
      ::std::process::exit(0);
    },
    Some(Input::KeyUp) |
    Some(Input::Character('k')) => {
      scroll_up(model);
      display(&mut window, &model);
    },
    Some(Input::KeyDown) |
    Some(Input::Character('j')) => {
      scroll_down(window, model);
      display(&mut window, &model);
    },
    Some(Input::KeyPPage) => {
      model.pointed = max(0, model.pointed.saturating_sub(get_height(&window) - 1));
      model.first = min(model.first, model.pointed);
      display(&mut window, &model);
    },
    Some(Input::KeyNPage) => {
      model.pointed = min(
        min(window.get_max_y().saturating_sub(1) as usize, model.entries.len().saturating_sub(1)),
        model.pointed.saturating_add(get_height(&window) - 1));
      model.first = max(0, model.pointed.saturating_sub(get_height(&window) - 1));
      display(&mut window, &model);
    },
    Some(Input::KeyLeft) |
    Some(Input::Character('h')) => {
      let mut new_path = path::PathBuf::from(&model.cwd);
      new_path.pop();
      change_cwd(&mut model, String::from(new_path.to_str().unwrap())).unwrap();
      display(&mut window, &model);
    },
    Some(Input::Character('\r')) |
    Some(Input::KeyRight) |
    Some(Input::Character('l')) => {
      change_cwd_to_pointed( &mut model);
      display(&mut window, &model);
    },
    Some(Input::Character('.')) => {
      model.show_hidden = !model.show_hidden;
      update_model_from_dir(&mut model, None).unwrap();
      display(&mut window, &model);
    },
    Some(Input::Character('d')) => {
      model.show_detail = !model.show_detail;
      display(&mut window, &model);
    },
    Some(Input::Character('`')) => {
      model.mode = Mode::Roaming;
      model.error = None;
      model.roam_path = model.cwd.clone();
      model.roam_path.push('/');
      roam_model(&mut model);
      display(&mut window, &model);
    },
    Some(Input::KeyResize) => {
      info!("window resized");
      display(&mut window, &model);
    },
    Some(Input::Character(' ')) => {
      model.selected_buffer = model.entries[model.pointed].path().to_str().unwrap().to_string();
    }
    _ => { info!("unknown key {:?}", c); },
  }
}

fn roaming_mode(c: Option<Input>, mut window: &mut pancurses::Window, mut model: &mut Model) {
  match c {
    Some(Input::Character('\u{1b}')) => {
      if model.escaped {
        model.mode = Mode::Browsing;
        model.cursor_shift = 0;
        display(&mut window, &model);
      } else {
        model.escaped = true;
      }
    },
    _ => {
      match c {
        Some(Input::Character('`')) => {
          model.mode = Mode::Browsing;
          model.cursor_shift = 0;
          display(&mut window, &model);
        },
        Some(Input::KeyUp) => {
          scroll_up(model);
          display(&mut window, &model);
        },
        Some(Input::Character('\t')) |
        Some(Input::KeyDown) => {
          scroll_down(window, model);
          display(&mut window, &model);
        },
        Some(Input::Character('\r')) => {
          change_cwd_to_pointed( &mut model);
          model.roam_path = model.cwd.clone() + "/";
          display(&mut window, &model);
        }
        Some(Input::Character(' ')) => {
          model.selected_buffer = model.entries[model.pointed].path().to_str().unwrap().to_string();
        }
        Some(Input::Character(l)) => {
          let roam_path_size = model.roam_path.len();
          model.roam_path.insert(roam_path_size.saturating_sub(model.cursor_shift).into(), l);
          roam_model(&mut model);
          display(&mut window, &model);
        }
        Some(Input::KeyLeft) => {
          model.cursor_shift = min(model.cursor_shift.saturating_add(1), model.roam_path.len());
          display(&mut window, &model);
        },
        Some(Input::KeyRight) => {
          model.cursor_shift = model.cursor_shift.saturating_sub(1);
          display(&mut window, &model);
        },
        Some(Input::KeyEnd) => {
          model.cursor_shift = 0;
          display(&mut window, &model);
        }
        Some(Input::KeyHome) => {
          model.cursor_shift = model.roam_path.len();
          display(&mut window, &model);
        }
        Some(Input::KeyBackspace) => {
          let roam_path_size = model.roam_path.len();
          let position = roam_path_size.saturating_sub(model.cursor_shift);
          if model.escaped {
            let mut s: String;
            {
              let mut v = model.roam_path.split("/").filter(|d| d != &"").collect::<Vec<&str>>();
              v.pop();
              s = "/".to_string() + &v.join("/");
            }
            model.roam_path = s;
          } else {
            model.roam_path.remove(position.saturating_sub(1).into());
          }
          roam_model(&mut model);
          display(&mut window, &model);
        }
        Some(Input::KeyResize) => {
          info!("window resized");
          display(&mut window, &model);
        },
        _ => { info!("unknown key {:?}", c); },
      }
      model.escaped = false;
    }
  }
}

fn main() {
  WriteLogger::init(LevelFilter::Info, Config::default(), File::create("b.log").unwrap()).unwrap();

  let mut window = pancurses::initscr();
  ncurses::set_escdelay(0);
  pancurses::cbreak();
  pancurses::nonl();
  pancurses::noecho();
  pancurses::start_color();
  pancurses::use_default_colors();
  window.keypad(true);
  let mut model = Model {
    entries: vec![],
    pointed: 0,
    first: 0,
    mode: Mode::Browsing,
    cwd: String::from(fs::canonicalize(
      std::env::args()
        .skip(1)
        .next()
        .unwrap_or(String::from("."))
      ).unwrap().to_str().unwrap()
    ),
    error: None,
    color_scheme: get_colors_db(),
    show_hidden: false,
    show_detail: false,
    roam_path: "".to_string(),
    no_match: true,
    escaped: false,
    cursor_shift: 0,
    selected_buffer: "".to_string(),
  };

  update_model_from_dir(&mut model, None).unwrap();
  display(&mut window, &model);

  loop {
    let c = window.getch();
    match model.mode {
      Mode::Browsing => browsing_mode(c, &mut window, &mut model),
      Mode::Roaming => roaming_mode(c, &mut window, &mut model),
      _ => (),
    }
  }
}
