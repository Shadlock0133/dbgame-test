#![deny(clippy::as_conversions)]

use std::{
    env, fs,
    io::BufReader,
    path::{Path, PathBuf},
    process::{Command, ExitCode, Stdio},
};

use clap::Parser;

const GAME_DIR: &str = "dbgame-test";
const ISO_LABEL: &str = "DBGAME-TEST";

#[cfg(unix)]
const DREAMBOX_NAME: &str = "DreamboxVM";
#[cfg(windows)]
const DREAMBOX_NAME: &str = "DreamboxVM.exe";

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
        .current_dir(GAME_DIR)
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
        primary_volume_name: Some(ISO_LABEL.to_string()),
        input_files: vec![main_wasm.into_std_path_buf()],
    })
    .unwrap();
    fs::write(&iso, iso_content).unwrap();
    Ok(iso.into_std_path_buf())
}

fn run(game: &Path, _opt: RunOpt) -> ExitCode {
    let dreambox_path = env::var_os("DREAMBOX_PATH")
        .expect("Missing DREAMBOX_PATH env variable");
    let dreambox_exe = Path::new(&dreambox_path).join(DREAMBOX_NAME);
    if !dreambox_exe.exists() {
        eprintln!("missing {DREAMBOX_NAME} at {}", dreambox_exe.display());
    }
    let status = Command::new(dreambox_exe)
        .arg("-b")
        .arg("-s")
        .arg(game)
        .current_dir(dreambox_path)
        .status()
        .unwrap();
    if !status.success() {
        eprintln!("{DREAMBOX_NAME} returned non-zero status");
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
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
