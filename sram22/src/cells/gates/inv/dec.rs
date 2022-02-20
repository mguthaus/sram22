use magic_vlsi::units::{Distance, Rect, Vec2};
use magic_vlsi::MagicInstance;

use crate::config::TechConfig;
use crate::error::Result;
use crate::layout::{draw_contact, ContactStack};

pub fn generate_inv_dec(m: &mut MagicInstance, tc: &TechConfig) -> Result<()> {
    let nand2_pm_sh = m.load_layout_cell("nand2_pm_sh")?;
    let inv_pm_sh = m.load_layout_cell("inv_pm_sh_2")?;
    let rowend = m.load_layout_cell("rowend")?;

    let cell_name = String::from("inv_dec_auto");
    m.drc_off()?;
    m.load(&cell_name)?;
    m.enable_box()?;
    m.set_snap(magic_vlsi::SnapMode::Internal)?;

    let nand2 = m.place_layout_cell(
        nand2_pm_sh.clone(),
        Vec2::new(-nand2_pm_sh.bbox.width(), Distance::zero()),
    )?;
    let inv = m.place_layout_cell(inv_pm_sh, Vec2::zero())?;

    let nand_out_port = nand2.port_bbox("Y");
    let inv_in_port = inv.port_bbox("A");

    let li_box = Rect::from_dist(
        Distance::zero(),
        nand_out_port.bottom_edge(),
        inv_in_port.right_edge(),
        nand_out_port.top_edge(),
    );
    m.paint_box(li_box, "li")?;

    m.rename_cell_pin(&inv, "A", "A")?;
    m.port_make_default()?;
    m.rename_cell_pin(&inv, "VPWR", "VPWR")?;
    m.port_make_default()?;
    m.rename_cell_pin(&inv, "VGND", "VGND")?;
    m.port_make_default()?;
    m.rename_cell_pin(&inv, "VPB", "VPB")?;
    m.port_make_default()?;
    m.rename_cell_pin(&inv, "Y", "Y")?;
    m.port_make_default()?;

    m.select_cell(&nand2.name)?;
    m.delete()?;

    let mut rowend =
        m.place_layout_cell(rowend, Vec2::new(inv.bbox().right_edge(), Distance::zero()))?;
    m.flip_cell_x(&mut rowend)?;

    let mut inv_out_port = inv.port_bbox("Y");
    inv_out_port.ll.x = inv_out_port.ur.x - tc.layer("ct").width;

    let ctbox = draw_contact(
        m,
        tc,
        ContactStack {
            top: "m1",
            contact_drc: "ct",
            contact_layer: "viali",
            bot: "li",
        },
        inv_out_port,
        true,
    )?;

    let wl_port = rowend.port_bbox("WL");
    let m1_box = Rect::from_dist(
        ctbox.top.left_edge(),
        wl_port.bottom_edge(),
        ctbox.top.right_edge(),
        ctbox.top.top_edge(),
    );
    m.paint_box(m1_box, "m1")?;

    let m2_box = Rect::from_dist(
        m1_box.left_edge(),
        wl_port.bottom_edge(),
        inv.bbox().right_edge(),
        wl_port.top_edge(),
    );
    m.paint_box(m2_box, "m2")?;

    let contact_region = m1_box.overlap(m2_box);
    draw_contact(
        m,
        tc,
        ContactStack {
            top: "m2",
            contact_drc: "via1",
            contact_layer: "via1",
            bot: "m1",
        },
        contact_region,
        false,
    )?;

    m.select_cell(&rowend.name)?;
    m.delete()?;

    m.port_renumber()?;
    m.save(&cell_name)?;

    Ok(())
}