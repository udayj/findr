use crate::EntryType::*;
use clap::{App, Arg};
use regex::Regex;
use std::error::Error;
use walkdir::{WalkDir, DirEntry};

type MyResult<T> = Result<T, Box< dyn Error>>;

#[derive(Debug, Eq, PartialEq)]
enum EntryType {
    Dir,
    File,
    Link
}

#[derive(Debug)]
pub struct Config {

    paths: Vec<String>,
    names: Vec<Regex>,
    entry_types: Vec<EntryType>,
}

pub fn get_args() -> MyResult<Config> {

    let matches = App::new("findr")
                    .version("0.1.0")
                    .author("udayj")
                    .about("Rust find")
                    .arg(
                        Arg::with_name("paths")
                            .value_name("PATH")
                            .help("Search Paths")
                            .multiple(true)
                            .default_value(".")
                    )
                    .arg(
                        Arg::with_name("names")
                            .short("n")
                            .long("name")
                            .value_name("NAME")
                            .help("Search patterns")
                            .takes_value(true)
                            .multiple(true)

                    )
                    .arg(
                        Arg::with_name("types")
                            .short("t")
                            .long("type")
                            .value_name("TYPE")
                            .help("Restrict search to these types")
                            .possible_values(&["f", "d", "l"])
                            .takes_value(true)
                            .multiple(true)
                    )
                    .get_matches();
    
    let names = matches.values_of_lossy("names")
                                            .map(|vals| {
                                                vals.into_iter()
                                                .map(|name| {
                                                    Regex::new(&name)
                                                    .map_err(|_| format!(
                                                        "Invalid --name \"{}\"", name))
                                            })
                                            .collect::<Result<Vec<_>,_>>()
                                        })
                                        .transpose()?
                                        .unwrap_or_default();
    

    let entry_types = matches.values_of_lossy("types")
                            .map(|vals| {
                                vals.iter()
                                .map(
                                    |val| {
                                        match val.as_str() {
                                            "d" => Dir,
                                            "f" => File,
                                            "l" => Link,
                                            _ => unreachable!("Invalid type"),
                                        }
                                    }
                                )
                                .collect()
                            })
                            .unwrap_or_default();
    
    Ok(
        Config {
            paths: matches.values_of_lossy("paths").unwrap(),
            names,
            entry_types
        }
    )
}

pub fn run(config: Config) -> MyResult<()> {

    let mut entries: Vec<DirEntry> = Vec::new();
    for path in config.paths {
        for entry in WalkDir::new(path) {
            match entry {
                Err(e) => eprintln!("{}",e),
                Ok(entry) => {
                    //println!("{}", entry.path().display());
                    
                    entries.push(entry);
                }
                    
                
            }
        }
    }

    let actual_entry_types:Vec<EntryType>;

    if config.entry_types.len() > 0 {
        actual_entry_types = config.entry_types;
    }
    else {
        actual_entry_types = Vec::from([Dir, File, Link]);
    }

    //println!("{:?}", actual_entry_types);

    let final_entries = entries.into_iter().filter(|val| {
        
        
        if val.file_type().is_dir() && actual_entry_types.contains(&Dir) {
            
            return true;
        }
        if val.file_type().is_file() && actual_entry_types.contains(&File) {
           
            return true;
        }

        if val.file_type().is_symlink() && actual_entry_types.contains(&Link) {
            
            return true;
        }
        false
    }).filter(|val| {
        
        if config.names.len() == 0 {
            return true;
        }

        for name in &config.names {
            if name.is_match(val.path().file_name().unwrap().to_str().unwrap()) {
                return true;
            }
        }
        false

    }).collect::<Vec<DirEntry>>();

    for entry in final_entries {
        println!("{}", entry.path().display());
        
    }
        
    Ok(())
}