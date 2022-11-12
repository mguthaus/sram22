use layout21::raw::align::AlignRect;
use layout21::raw::geom::Dir;
use layout21::raw::{
    Abstract, AbstractPort, BoundBoxTrait, Cell, Instance, Layout, Point, Rect, Shape, Span,
    TransformTrait,
};
use layout21::utils::Ptr;
use pdkprims::mos::{Intent, MosDevice, MosParams, MosType};
use pdkprims::PdkLib;

use super::array::*;
use super::common::{draw_two_level_contact, TwoLevelContactParams};
use crate::layout::bank::GateList;
use crate::layout::common::{MergeArgs, NWELL_COL_SIDE_EXTEND, NWELL_COL_VERT_EXTEND};
use crate::layout::route::{ContactBounds, Router, VertDir};
use crate::schematic::precharge::{PrechargeArrayParams, PrechargeParams};
use crate::Result;

fn draw_precharge(lib: &mut PdkLib, args: PrechargeParams) -> Result<Ptr<Cell>> {
    let name = "precharge".to_string();

    let mut layout = Layout {
        name: name.clone(),
        insts: vec![],
        elems: vec![],
        annotations: vec![],
    };

    let mut abs = Abstract::new(&name);

    let mut params = MosParams::new();
    params
        .dnw(false)
        .direction(Dir::Horiz)
        .add_device(MosDevice {
            mos_type: MosType::Pmos,
            width: args.equalizer_width,
            length: args.length,
            fingers: 1,
            intent: Intent::Svt,
            skip_sd_metal: vec![],
        })
        .add_device(MosDevice {
            mos_type: MosType::Pmos,
            width: args.pull_up_width,
            length: args.length,
            fingers: 1,
            intent: Intent::Svt,
            skip_sd_metal: vec![],
        })
        .add_device(MosDevice {
            mos_type: MosType::Pmos,
            width: args.pull_up_width,
            length: args.length,
            fingers: 1,
            intent: Intent::Svt,
            skip_sd_metal: vec![],
        });
    let ptx = lib.draw_mos(params)?;

    let inst = Instance {
        inst_name: "mos".to_string(),
        cell: ptx.cell.clone(),
        loc: Point::new(0, 0),
        angle: Some(90f64),
        reflect_vert: false,
    };
    let xform = inst.transform();

    let mut port = ptx.gate_port(0).unwrap();
    port.set_net("pc_b");
    let port = port.transform(&xform);
    abs.add_port(port);

    let mut port = ptx.sd_port(0, 0).unwrap();
    port.set_net("br0");
    let port = port.transform(&xform);
    abs.add_port(port);

    let mut port = ptx.sd_port(0, 1).unwrap();
    port.set_net("bl0");
    let port = port.transform(&xform);
    abs.add_port(port);

    let mut port = ptx.sd_port(1, 0).unwrap();
    port.set_net("br1");
    let port = port.transform(&xform);
    abs.add_port(port);

    let mut port = ptx.sd_port(1, 1).unwrap();
    port.set_net("vdd0");
    let port = port.transform(&xform);
    abs.add_port(port);

    let mut port = ptx.sd_port(2, 0).unwrap();
    port.set_net("vdd1");
    let port = port.transform(&xform);
    abs.add_port(port);

    let mut port = ptx.sd_port(2, 1).unwrap();
    port.set_net("bl1");
    let port = port.transform(&xform);
    abs.add_port(port);

    abs.add_port(ptx.merged_vpb_port(0).transform(&xform));

    layout.insts.push(inst);

    let cell = Cell {
        name,
        abs: Some(abs),
        layout: Some(layout),
    };

    let ptr = Ptr::new(cell);
    lib.lib.cells.push(ptr.clone());

    Ok(ptr)
}

pub fn draw_tap_cell(lib: &mut PdkLib) -> Result<Ptr<Cell>> {
    let params = TwoLevelContactParams::builder()
        .name("pc_tap_cell")
        .bot_stack("ntap")
        .top_stack("viali")
        .bot_rows(10)
        .top_rows(10)
        .build()?;
    let contact = draw_two_level_contact(lib, params)?;
    Ok(contact)
}

pub fn draw_precharge_array(lib: &mut PdkLib, args: PrechargeArrayParams) -> Result<Ptr<Cell>> {
    let PrechargeArrayParams {
        width,
        instance_params,
        name,
    } = args;
    assert!(width >= 2);
    let pc = draw_precharge(lib, instance_params)?;

    let core = draw_cell_array(
        ArrayCellParams {
            name: "precharge_pc_array".to_string(),
            num: width,
            cell: pc,
            spacing: Some(2_500),
            flip: FlipMode::AlternateFlipHorizontal,
            flip_toggle: false,
            direction: Dir::Horiz,
        },
        lib,
    )?;

    let tap = draw_tap_cell(lib)?;

    let taps = draw_cell_array(
        ArrayCellParams {
            name: "precharge_tap_array".to_string(),
            num: width + 1,
            cell: tap,
            spacing: Some(2_500),
            flip: FlipMode::AlternateFlipHorizontal,
            flip_toggle: false,
            direction: Dir::Horiz,
        },
        lib,
    )?;

    let mut cell = Cell::empty(name);
    let core = Instance {
        inst_name: "pc_array".to_string(),
        cell: core.cell,
        reflect_vert: false,
        angle: None,
        loc: Point::new(0, 0),
    };
    let mut taps = Instance {
        inst_name: "tap_array".to_string(),
        cell: taps.cell,
        reflect_vert: false,
        angle: None,
        loc: Point::new(0, 0),
    };
    taps.align_centers_gridded(core.bbox(), lib.pdk.grid());

    cell.abs_mut().ports.append(&mut core.ports());

    let iter = taps.ports().into_iter().enumerate().map(|(i, mut p)| {
        p.set_net(format!("vdd_{}", i));
        p
    });
    cell.abs_mut().ports.extend(iter);
    cell.abs_mut().ports.append(&mut taps.ports());

    let m0 = lib.pdk.metal(0);
    let m2 = lib.pdk.metal(2);

    let mut router = Router::new("precharge_array_route", lib.pdk.clone());
    router.cfg().line(2);

    let pc_b_0 = core.port("pc_b_0").largest_rect(m0).unwrap();

    let span = Span::new(
        pc_b_0.left(),
        core.port(format!("pc_b_{}", width - 1))
            .largest_rect(m0)
            .unwrap()
            .right(),
    );
    let top = pc_b_0.bottom();
    let rect = Rect::span_builder()
        .with(Dir::Horiz, span)
        .with(Dir::Vert, Span::new(top - 3 * router.cfg().line(2), top))
        .build();

    router.trace(rect, 2);

    let mut port = AbstractPort::new("pc_b");
    port.add_shape(m2, Shape::Rect(rect));
    cell.abs_mut().add_port(port);

    for i in 0..width {
        let src = core.port(format!("pc_b_{i}")).largest_rect(m0).unwrap();
        let mut trace = router.trace(src, 0);
        trace.place_cursor(Dir::Vert, false).vert_to(rect.bottom());

        let intersect = trace.rect().intersection(&rect.bbox()).into_rect();
        trace.contact_up(rect).increment_layer().contact_on(
            intersect,
            VertDir::Above,
            ContactBounds::FillDir {
                dir: Dir::Vert,
                size: rect.height(),
                layer: lib.pdk.metal(1),
            },
        );
    }

    let nwell = lib.pdk.get_layerkey("nwell").unwrap();

    let elt = MergeArgs::builder()
        .layer(nwell)
        .insts(GateList::Array(&core, width))
        .port_name("vpb")
        .top_overhang(NWELL_COL_VERT_EXTEND)
        .bot_overhang(NWELL_COL_VERT_EXTEND)
        .left_overhang(NWELL_COL_SIDE_EXTEND + 200)
        .right_overhang(NWELL_COL_SIDE_EXTEND + 200)
        .build()?
        .element();
    cell.layout_mut().add(elt);

    cell.layout_mut().add_inst(core);
    cell.layout_mut().add_inst(taps);
    cell.layout_mut().add_inst(router.finish());

    let ptr = Ptr::new(cell);
    lib.lib.cells.push(ptr.clone());

    Ok(ptr)
}

#[cfg(test)]
mod tests {
    use pdkprims::tech::sky130;

    use crate::utils::test_gds_path;

    use super::*;

    #[test]
    fn test_sky130_precharge() -> Result<()> {
        let mut lib = sky130::pdk_lib("test_sky130_precharge")?;
        draw_precharge(
            &mut lib,
            PrechargeParams {
                name: "test_sky130_precharge".to_string(),
                length: 150,
                pull_up_width: 1_200,
                equalizer_width: 1_000,
            },
        )?;

        lib.save_gds(test_gds_path(&lib))?;

        Ok(())
    }

    #[test]
    fn test_sky130_precharge_array() -> Result<()> {
        let mut lib = sky130::pdk_lib("test_sky130_precharge_array")?;
        draw_precharge_array(
            &mut lib,
            PrechargeArrayParams {
                width: 32,
                instance_params: PrechargeParams {
                    name: "precharge".to_string(),
                    length: 150,
                    pull_up_width: 1_200,
                    equalizer_width: 1_000,
                },
                name: "precharge_array".to_string(),
            },
        )?;

        lib.save_gds(test_gds_path(&lib))?;

        Ok(())
    }
}
