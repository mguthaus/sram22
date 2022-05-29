use std::path::PathBuf;

use layout21::{
    gds21::GdsLibrary,
    raw::{Cell, Layers, Library},
    utils::Ptr,
};
use pdkprims::tech::sky130;
use vlsir::{circuit::ExternalModule, reference::To, QualifiedName, Reference};

use crate::{
    mos::{ext_nmos, ext_pmos},
    utils::simple_ext_module,
};

pub const SKY130_DOMAIN: &str = "sky130";
pub const SRAM_SP_CELL: &str = "sram_sp_cell";
pub const SRAM_CONTROL: &str = "sramgen_control";
pub const SRAM_SP_SENSE_AMP: &str = "sramgen_sp_sense_amp";

pub fn sram_sp_cell() -> ExternalModule {
    simple_ext_module(
        SKY130_DOMAIN,
        SRAM_SP_CELL,
        &["BL", "BR", "VDD", "VSS", "WL"],
    )
}

fn cell_gds(
    layers: Ptr<Layers>,
    gds_file: &str,
    cell_name: &str,
) -> Result<Ptr<Cell>, Box<dyn std::error::Error>> {
    let mut path = external_gds_path();
    path.push(gds_file);
    let lib = GdsLibrary::load(&path)?;
    let lib = Library::from_gds(&lib, Some(layers))?;

    let cell = lib
        .cells
        .iter()
        .find(|&x| {
            let x = x.read().unwrap();
            x.name == cell_name
        })
        .unwrap();

    Ok(cell.clone())
}

type CellGdsResult = Result<Ptr<Cell>, Box<dyn std::error::Error>>;

pub fn sram_sp_cell_gds(layers: Ptr<Layers>) -> CellGdsResult {
    cell_gds(
        layers,
        "sram_sp_cell.gds",
        "sky130_fd_bd_sram__sram_sp_cell_opt1",
    )
}

pub fn colend_gds(layers: Ptr<Layers>) -> CellGdsResult {
    cell_gds(
        layers,
        "sram_sp_colend.gds",
        "sky130_fd_bd_sram__sram_sp_colend",
    )
}

pub fn colend_cent_gds(layers: Ptr<Layers>) -> CellGdsResult {
    cell_gds(
        layers,
        "sram_sp_colend_cent.gds",
        "sky130_fd_bd_sram__sram_sp_colend_cent",
    )
}

pub fn colend_p_cent_gds(layers: Ptr<Layers>) -> CellGdsResult {
    cell_gds(
        layers,
        "sram_sp_colend_p_cent.gds",
        "sky130_fd_bd_sram__sram_sp_colend_p_cent",
    )
}

pub fn corner_gds(layers: Ptr<Layers>) -> CellGdsResult {
    cell_gds(
        layers,
        "sram_sp_corner.gds",
        "sky130_fd_bd_sram__sram_sp_corner",
    )
}

pub fn rowend_gds(layers: Ptr<Layers>) -> CellGdsResult {
    cell_gds(
        layers,
        "sram_sp_rowend.gds",
        "sky130_fd_bd_sram__sram_sp_rowend",
    )
}

pub fn wlstrap_gds(layers: Ptr<Layers>) -> CellGdsResult {
    cell_gds(
        layers,
        "sram_sp_wlstrap.gds",
        "sky130_fd_bd_sram__sram_sp_wlstrap",
    )
}

pub fn wlstrap_p_gds(layers: Ptr<Layers>) -> CellGdsResult {
    cell_gds(
        layers,
        "sram_sp_wlstrap_p.gds",
        "sky130_fd_bd_sram__sram_sp_wlstrap_p",
    )
}

pub fn sram_sp_cell_ref() -> Reference {
    Reference {
        to: Some(To::External(QualifiedName {
            domain: SKY130_DOMAIN.to_string(),
            name: SRAM_SP_CELL.to_string(),
        })),
    }
}

pub fn sramgen_control() -> ExternalModule {
    simple_ext_module(
        SKY130_DOMAIN,
        SRAM_CONTROL,
        &[
            "clk",
            "cs",
            "we",
            "pc",
            "pc_b",
            "wl_en",
            "write_driver_en",
            "sense_en",
            "vdd",
            "vss",
        ],
    )
}

pub fn sramgen_control_ref() -> Reference {
    Reference {
        to: Some(To::External(QualifiedName {
            domain: SKY130_DOMAIN.to_string(),
            name: SRAM_CONTROL.to_string(),
        })),
    }
}

pub fn external_gds_path() -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("..");
    p.push("tech/sky130/gds");
    p
}

/// Reference to a single port sense amplifier.
///
/// The SPICE subcircuit definition looks like this:
/// ```spice
/// .SUBCKT AAA_Comp_SA_sense clk inn inp outn outp VDD VSS
/// ```
pub fn sramgen_sp_sense_amp() -> ExternalModule {
    simple_ext_module(
        SKY130_DOMAIN,
        SRAM_SP_SENSE_AMP,
        &["clk", "inn", "inp", "outn", "outp", "VDD", "VSS"],
    )
}

pub fn sramgen_sp_sense_amp_ref() -> Reference {
    Reference {
        to: Some(To::External(QualifiedName {
            domain: SKY130_DOMAIN.to_string(),
            name: SRAM_SP_SENSE_AMP.to_string(),
        })),
    }
}

pub fn all_external_modules() -> Vec<ExternalModule> {
    vec![
        ext_nmos(),
        ext_pmos(),
        sram_sp_cell(),
        sramgen_control(),
        sramgen_sp_sense_amp(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{bbox, Result};

    #[test]
    fn test_colend() -> Result<()> {
        let lib = sky130::pdk_lib("test_colend")?;
        let cell = colend_gds(lib.pdk.layers())?;
        let bbox = bbox(&cell);
        assert_eq!(bbox.width(), 1200);
        assert_eq!(bbox.height(), 2055);
        Ok(())
    }

    #[test]
    fn test_rowend() -> Result<()> {
        let lib = sky130::pdk_lib("test_rowend")?;
        let cell = rowend_gds(lib.pdk.layers())?;
        let bbox = bbox(&cell);
        assert_eq!(bbox.width(), 1300);
        assert_eq!(bbox.height(), 1580);
        Ok(())
    }
}