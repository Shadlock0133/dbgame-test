use std::path::PathBuf;

pub struct Opt {
    pub eltorito_opt: ElToritoOpt,
    pub embedded_boot: Option<String>,
    pub grub2_mbr: Option<String>,
    pub boot_load_size: u32,
    pub protective_msdos_label: bool,
    pub input_files: Vec<PathBuf>,
}

pub struct ElToritoOpt {
    pub eltorito_boot: Option<String>,
    pub no_emu_boot: bool,
    pub no_boot: bool,
    pub boot_info_table: bool,
    pub grub2_boot_info: bool,
}
