use crate::config::sram::SramConfig;
use std::path::{Path, PathBuf};

pub fn out_sram(config: &SramConfig) -> String {
    let &SramConfig {
        num_words,
        data_width,
        mux_ratio,
        write_size,
        ..
    } = config;
    format!("sramgen_sram_{num_words}x{data_width}m{mux_ratio}w{write_size}_simple")
}

pub fn out_bin(work_dir: impl AsRef<Path>, name: &str) -> PathBuf {
    PathBuf::from(work_dir.as_ref()).join(format!("{}.pb.bin", name))
}

pub fn out_gds(work_dir: impl AsRef<Path>, name: &str) -> PathBuf {
    PathBuf::from(work_dir.as_ref()).join(format!("{}.gds", name))
}

pub fn out_verilog(work_dir: impl AsRef<Path>, name: &str) -> PathBuf {
    PathBuf::from(work_dir.as_ref()).join(format!("{}.v", name))
}

#[cfg(feature = "abstract_lef")]
pub fn out_lef(work_dir: impl AsRef<Path>, name: &str) -> PathBuf {
    PathBuf::from(work_dir.as_ref()).join(format!("{}.lef", name))
}
