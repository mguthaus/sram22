use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Eq, PartialEq, Clone, Hash, Serialize, Deserialize)]
pub struct SramConfig {
    pub num_words: i32,
    pub data_width: i32,
    pub mux_ratio: i32,
    pub write_size: i32,
    pub control: ControlMode,
}

#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash, Serialize, Deserialize)]
pub enum ControlMode {
    Simple,
    SimpleChipSelect,
    Replica,
}

#[derive(Debug)]
pub struct SramParams {
    pub name: String,
    pub wmask_width: usize,

    // Schematic
    pub row_bits: usize,
    pub col_bits: usize,
    pub col_select_bits: usize,

    // Layout
    pub rows: usize,
    pub cols: usize,
    pub mux_ratio: usize,

    // Verilog
    pub num_words: usize,
    pub data_width: usize,
    pub addr_width: usize,

    pub control: ControlMode,
}

pub fn parse_sram_config(path: impl AsRef<Path>) -> Result<SramConfig> {
    let contents = fs::read_to_string(path)?;
    let data = toml::from_str(&contents)?;
    Ok(data)
}
