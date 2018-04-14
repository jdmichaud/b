#![allow(dead_code)]
#![allow(unused_variables)]

#[macro_use] extern crate log;
extern crate simplelog;
extern crate pancurses;
extern crate ncurses;
extern crate itertools;

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

enum Mode {
  Browsing,
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
}

fn display_entry(window: &mut pancurses::Window, model: &Model, entry: &DirEntry) {
  window.printw(entry.path().as_path().file_name().unwrap().to_str().unwrap());
}

fn display_list(window: &mut pancurses::Window, model: &Model) {
  (&model.entries).into_iter()
    .skip(model.first)
    .take(get_height(window))
    .enumerate()
    .for_each(|(index, entry)| {
      let mut attrs: chtype = 0;
      if entry.path().is_dir() {
        info!("a directory!");
        match model.color_scheme.get("di") {
          Some(color) if color.len() == 2 => {
            attrs = attrs | chtype::from(ColorPair(color[1]) | Attribute::Bold);
            info!("apply {:?}", ColorPair(color[1]));
          },
          Some(_) | None => info!("no color for directory"),
        }
      }
      window.attrset(attrs);
      if index == model.pointed - model.first {
        attrs = attrs | chtype::from(Attribute::Reverse | Attribute::Bold);
        window.attrset(attrs);
        window.mv(index as i32 + 2, 0);
        window.printw(" > ");
      }
      window.mv(index as i32 + 2, 3);
      display_entry(window, model, &entry);
      window.attrset(Attribute::Normal | ColorPair(0));
    });
}

fn display(window: &mut pancurses::Window, model: &Model) {
  window.clear();
  window.mv(0, 0);
  window.printw(&format!("DIR: {}", model.cwd));
  display_list(window, model);
  window.mv(window.get_max_y() - 1, 0);
  match &model.error {
    &Some(ref error) => { window.printw(&format!("error: {}", error)); },
    &None => ()
  }
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
  };

  update_model_from_dir(&mut model, None).unwrap();
  display(&mut window, &model);

  loop {
    let c = window.getch();
    match c {
      Some(Input::Character('\u{1b}')) |
      Some(Input::Character('q')) => { // Escape key
        pancurses::endwin();
        ::std::process::exit(0);
      },
      Some(Input::KeyUp) |
      Some(Input::Character('k')) => {
        model.pointed = max(0, model.pointed.saturating_sub(1));
        model.first = min(model.first, model.pointed);
        display(&mut window, &model);
      },
      Some(Input::KeyDown) |
      Some(Input::Character('j')) => {
        model.pointed = min(
          min(window.get_max_y().saturating_sub(1) as usize, model.entries.len().saturating_sub(1)),
          model.pointed.saturating_add(1));
        model.first = max(0, model.pointed.saturating_sub(get_height(&window) - 1));
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
      }
      Some(Input::Character('\r')) |
      Some(Input::KeyRight) |
      Some(Input::Character('l')) => {
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
          display(&mut window, &model);
        }
      },
      Some(Input::KeyResize) => { display(&mut window, &model); },
      _ => { info!("unknown key {:?}", c); },
    }
  }
}
