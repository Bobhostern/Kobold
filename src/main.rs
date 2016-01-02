extern crate argparse;

mod compiler;
#[cfg(test)]
mod tests;

use compiler::Lexer;
use compiler::Parser;
use compiler::{Module, ModuleManager};
use compiler::ast::Expression;
use std::io::BufReader;
use std::fs::File;
use std::path::Path;
use std::fs;

use argparse::{ArgumentParser, Print, List};

struct Options {
    classpath: Vec<String>,
    excludes: Vec<String>,
}

fn select_files_in_directory(dir: &Path, excl:&Vec<&Path>, list: &mut Vec<String>) -> std::io::Result<()> {
    if try!(fs::metadata(dir)).is_dir() {
        for entry in try!(fs::read_dir(dir)) {
            let entry = try!(entry);
            if try!(fs::metadata(entry.path())).is_dir() {
                select_files_in_directory(&entry.path(), excl, list).ok().unwrap();
            } else {
                let p = entry.path();
                let mut accepted_extensions = vec!["kbld", "klb", "kobold", "ksc"];
                accepted_extensions.sort();
                if let Some(ext) = p.extension() {
                    if let Some(ext) = ext.to_str() {
                        if accepted_extensions.binary_search(&ext).is_ok() {
                            if classpath_filtering(excl, &p) {
                                list.push(p.to_str().unwrap().to_string());
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

fn select_files(cp: &Vec<&Path>, excl: &Vec<&Path>, list: &mut Vec<String>) {
    for fdir in cp {
        match fs::metadata(fdir) {
            Ok(dir) => {
                if dir.is_dir() {
                    select_files_in_directory(fdir, excl, list).ok().unwrap();
                } else {
                    panic!("Not a directory: {:?}", fdir);
                }
            },
            Err(e) => {
                panic!("IO Error: {}", e);
            }
        }
    }
}

fn classpath_filtering(excludes: &Vec<&Path>, a: &Path) -> bool {
    for e in excludes.iter() {
        if a.starts_with(e) {
            return false
        }
    };
    true
}

fn load_modules(flst: &Vec<String>, mman: &mut ModuleManager) {
    for file in flst {
        println!("File: {}", file);
        let fs: File = File::open(file.clone()).ok().unwrap();
        let rdr = BufReader::new(fs);
        let mut lex = Lexer::new(&file, rdr);
        let ts = lex.process();
        // println!("{:#?}", ts);
        let mut parser = Parser::new(&file, ts);
        let module_code = parser.parse_top();
        // println!("{:?}", module_code);

        match *module_code[0].clone() {
            Expression::ModuleDeclaration(name) => {
                // Add exports field... (maybe? after v1.0?)
                // Check the rest of the code, and prevent duplicate declarations
                let nmod_code: Vec<Box<Expression>> = module_code.iter().skip(1).cloned().collect();
                for inst in nmod_code.iter().cloned() {
                    match *inst {
                        Expression::ModuleDeclaration(aname) => {
                            panic!("Cannot declare module as {} and {} in file {}", name, aname, file);
                        },
                        _ => {},
                    }
                }
                let nmod = Module::new(&name, nmod_code);
                mman.add_module(&name, nmod);
            },
            _=>{panic!("Must declare module name.")}
        }
    }
}

fn main() {
    let mut opts = Options {
        classpath: vec![],
        excludes: vec![],
    };
    // Add classpathing...
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("Kobold example parser");
        ap.refer(&mut opts.classpath).add_option(&["-c", "--classpath"], List, "Module Path (Default: .)");
        ap.refer(&mut opts.excludes).add_option(&["-e", "--excludes"], List, "Excludes from classpath");
        ap.add_option(&["-v", "--version"], Print(env!("CARGO_PKG_VERSION").to_string()), "Program version");
        ap.parse_args_or_exit();
    }

    opts.excludes.sort();

    if opts.classpath.len() == 0 {
        opts.classpath = vec![".".to_string()];
    }

    let excludes: Vec<&Path> = opts.excludes.iter().map({|a| Path::new(&*a)}).collect();
    for e in excludes.iter() {
        println!("{}", e.display());
    }
    let classpath: Vec<&Path> = opts.classpath.iter().map({|a| Path::new(&*a)}).collect();

    println!("{:?}", classpath);

    let mut files_list: Vec<String> = vec![];
    select_files(&classpath, &excludes, &mut files_list);

    let mut mman = ModuleManager::new();
    load_modules(&files_list, &mut mman);
    println!("{:#?}", mman);
}
