use std::{
    env, fs,
    io::BufReader,
    path::{Path, PathBuf},
    process::{Command, ExitCode, Stdio},
};

use clap::Parser;

#[derive(Parser)]
enum Opt {
    Build(BuildOpt),
    Run(RunOpt),
}

#[derive(Parser)]
struct BuildOpt {
    #[clap(short, long)]
    release: bool,
}

#[derive(Parser)]
struct RunOpt {
    #[clap(short, long)]
    release: bool,
}

fn build(opt: BuildOpt) -> Result<PathBuf, ()> {
    let profile = if opt.release { "release" } else { "dev" };
    let mut command = Command::new("cargo");
    let mut child = command
        .current_dir("dbgame-test")
        .args(["build", "--message-format=json-render-diagnostics"])
        .args(["--profile", profile])
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let reader = BufReader::new(child.stdout.take().unwrap());
    let mut wasm = None;
    for message in cargo_metadata::Message::parse_stream(reader) {
        match message.unwrap() {
            cargo_metadata::Message::CompilerArtifact(artifact) => {
                let mut filenames = artifact.filenames;
                if filenames.len() == 1 {
                    wasm = filenames.pop();
                }
            }
            cargo_metadata::Message::BuildFinished(build_finished) => {
                match build_finished.success {
                    true => break,
                    false => panic!(),
                }
            }
            _ => (),
        }
    }
    child.wait().unwrap();
    let wasm = wasm.unwrap();
    let main_wasm = wasm.with_file_name("main.wasm");
    let iso = wasm.with_extension("iso");

    fs::copy(wasm, &main_wasm).unwrap();
    let iso_content = iso::create_iso(&iso::option::Opt {
        eltorito_opt: iso::option::ElToritoOpt {
            eltorito_boot: None,
            no_emu_boot: true,
            no_boot: true,
            boot_info_table: false,
            grub2_boot_info: false,
        },
        embedded_boot: None,
        grub2_mbr: None,
        boot_load_size: 0,
        protective_msdos_label: false,
        input_files: vec![main_wasm.into_std_path_buf()],
    })
    .unwrap();
    fs::write(&iso, iso_content).unwrap();
    Ok(iso.into_std_path_buf())
}

fn run(game: &Path, _opt: RunOpt) -> ExitCode {
    let dreambox_path = env::var_os("DREAMBOX_PATH")
        .expect("Missing DREAMBOX_PATH env variable");
    let status = Command::new("./DreamboxVM")
        .arg("-b")
        .arg("-s")
        .arg(game)
        .current_dir(dreambox_path)
        .status();
    eprintln!("{status:?}");
    match status {
        Ok(s) if !s.success() => ExitCode::FAILURE,
        Err(_) => ExitCode::FAILURE,
        Ok(_) => ExitCode::SUCCESS,
    }
}

fn main() -> ExitCode {
    let args = Opt::parse();

    match args {
        Opt::Build(opt) => {
            build(opt).unwrap();
            ExitCode::SUCCESS
        }
        Opt::Run(opt) => {
            let game = build(BuildOpt {
                release: opt.release,
            })
            .unwrap();
            run(&game, opt)
        }
    }
}
