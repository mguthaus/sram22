use std::collections::HashMap;

use pdkprims::mos::MosType;
use serde::{Deserialize, Serialize};
use vlsir::circuit::{port, Instance, Module, Port};

use crate::config::gate::{AndParams, GateParams, Size};
use crate::schematic::conns::{conn_map, port_inout, port_input, port_output, sig_conn, signal};
use crate::schematic::local_reference;
use crate::schematic::mos::Mosfet;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize, Hash)]
pub struct Gate {
    pub gate_type: GateType,
    pub size: Size,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize, Hash)]
pub enum GateType {
    Inv,
    Nand2,
    Nand3,
}

impl GateType {
    pub fn num_inputs(&self) -> usize {
        match *self {
            GateType::Inv => 1,
            GateType::Nand2 => 2,
            GateType::Nand3 => 3,
        }
    }
}

impl Gate {
    #[inline]
    pub fn num_inputs(&self) -> usize {
        self.gate_type.num_inputs()
    }

    #[inline]
    pub fn new(gate_type: GateType, size: Size) -> Self {
        Self { gate_type, size }
    }
}

impl From<GateType> for fanout::GateType {
    fn from(x: GateType) -> Self {
        match x {
            GateType::Inv => fanout::GateType::INV,
            GateType::Nand2 => fanout::GateType::NAND2,
            GateType::Nand3 => fanout::GateType::NAND3,
        }
    }
}

pub fn and2(params: &AndParams) -> Vec<Module> {
    let vdd = signal("vdd");
    let a = signal("a");
    let b = signal("b");
    let y = signal("y");
    let vss = signal("vss");

    let ports = vec![
        port_input(&a),
        port_input(&b),
        port_output(&y),
        port_inout(&vdd),
        port_inout(&vss),
    ];

    let mut m = Module {
        name: params.name.clone(),
        ports,
        signals: vec![],
        instances: vec![],
        parameters: vec![],
    };

    let nand_name = format!("{}_nand", &params.name);
    let nand = nand2(&GateParams {
        name: nand_name.clone(),
        size: params.nand.size,
        length: params.nand.length,
    });
    let inv_name = format!("{}_inv", &params.name);
    let inv = inv(&GateParams {
        name: inv_name.clone(),
        size: params.inv.size,
        length: params.inv.length,
    });

    let tmp = signal("tmp");

    // nand
    let mut conns = HashMap::new();
    conns.insert("vdd", sig_conn(&vdd));
    conns.insert("gnd", sig_conn(&vss));
    conns.insert("a", sig_conn(&a));
    conns.insert("b", sig_conn(&b));
    conns.insert("y", sig_conn(&tmp));
    m.instances.push(Instance {
        name: "nand".to_string(),
        module: local_reference(nand_name),
        connections: conn_map(conns),
        parameters: HashMap::new(),
    });

    // inv
    let mut conns = HashMap::new();
    conns.insert("vdd", sig_conn(&vdd));
    conns.insert("gnd", sig_conn(&vss));
    conns.insert("din", sig_conn(&tmp));
    conns.insert("din_b", sig_conn(&y));
    m.instances.push(Instance {
        name: "inv".to_string(),
        module: local_reference(inv_name),
        connections: conn_map(conns),
        parameters: HashMap::new(),
    });

    vec![nand, inv, m]
}

pub fn nand2(params: &GateParams) -> Module {
    let length = params.length;
    let size = params.size;

    let mut m = Module {
        name: params.name.clone(),
        ports: vec![],
        signals: vec![],
        instances: vec![],
        parameters: vec![],
    };
    let gnd = signal("gnd");
    let vdd = signal("vdd");
    let a = signal("a");
    let b = signal("b");
    let y = signal("y");
    let x = signal("x");

    for sig in [gnd.clone(), vdd.clone()] {
        m.ports.push(Port {
            signal: Some(sig),
            direction: port::Direction::Inout as i32,
        });
    }
    for sig in [a.clone(), b.clone()] {
        m.ports.push(Port {
            signal: Some(sig),
            direction: port::Direction::Input as i32,
        });
    }
    m.ports.push(Port {
        signal: Some(y.clone()),
        direction: port::Direction::Output as i32,
    });

    m.instances.push(
        Mosfet {
            name: "n1".to_string(),
            width: size.nmos_width,
            length,
            drain: sig_conn(&x),
            source: sig_conn(&gnd),
            gate: sig_conn(&a),
            body: sig_conn(&gnd),
            mos_type: MosType::Nmos,
        }
        .into(),
    );
    m.instances.push(
        Mosfet {
            name: "n2".to_string(),
            width: size.nmos_width,
            length,
            drain: sig_conn(&y),
            source: sig_conn(&x),
            gate: sig_conn(&b),
            body: sig_conn(&gnd),
            mos_type: MosType::Nmos,
        }
        .into(),
    );
    m.instances.push(
        Mosfet {
            name: "p1".to_string(),
            width: size.pmos_width,
            length,
            drain: sig_conn(&y),
            source: sig_conn(&vdd),
            gate: sig_conn(&a),
            body: sig_conn(&vdd),
            mos_type: MosType::Pmos,
        }
        .into(),
    );
    m.instances.push(
        Mosfet {
            name: "p2".to_string(),
            width: size.pmos_width,
            length,
            drain: sig_conn(&y),
            source: sig_conn(&vdd),
            gate: sig_conn(&b),
            body: sig_conn(&vdd),
            mos_type: MosType::Pmos,
        }
        .into(),
    );

    m
}

pub fn nor2(params: &GateParams) -> Module {
    let length = params.length;
    let size = params.size;

    let gnd = signal("gnd");
    let vdd = signal("vdd");
    let a = signal("a");
    let b = signal("b");
    let y = signal("y");
    let x = signal("x");

    let ports = vec![
        port_input(&a),
        port_input(&b),
        port_output(&y),
        port_inout(&vdd),
        port_inout(&gnd),
    ];

    let mut m = Module {
        name: params.name.clone(),
        ports,
        signals: vec![],
        instances: vec![],
        parameters: vec![],
    };

    m.instances.push(
        Mosfet {
            name: "n1".to_string(),
            width: size.nmos_width,
            length,
            drain: sig_conn(&y),
            source: sig_conn(&gnd),
            gate: sig_conn(&a),
            body: sig_conn(&gnd),
            mos_type: MosType::Nmos,
        }
        .into(),
    );
    m.instances.push(
        Mosfet {
            name: "n2".to_string(),
            width: size.nmos_width,
            length,
            drain: sig_conn(&y),
            source: sig_conn(&gnd),
            gate: sig_conn(&b),
            body: sig_conn(&gnd),
            mos_type: MosType::Nmos,
        }
        .into(),
    );
    m.instances.push(
        Mosfet {
            name: "p1".to_string(),
            width: size.pmos_width,
            length,
            drain: sig_conn(&y),
            source: sig_conn(&x),
            gate: sig_conn(&a),
            body: sig_conn(&vdd),
            mos_type: MosType::Pmos,
        }
        .into(),
    );
    m.instances.push(
        Mosfet {
            name: "p2".to_string(),
            width: size.pmos_width,
            length,
            drain: sig_conn(&x),
            source: sig_conn(&vdd),
            gate: sig_conn(&b),
            body: sig_conn(&vdd),
            mos_type: MosType::Pmos,
        }
        .into(),
    );

    m
}

pub fn nand3(params: &GateParams) -> Module {
    let length = params.length;
    let size = params.size;

    let mut m = Module {
        name: params.name.clone(),
        ports: vec![],
        signals: vec![],
        instances: vec![],
        parameters: vec![],
    };
    let gnd = signal("gnd");
    let vdd = signal("vdd");
    let a = signal("a");
    let b = signal("b");
    let c = signal("c");
    let y = signal("y");
    let x1 = signal("x1");
    let x2 = signal("x2");

    for sig in [gnd.clone(), vdd.clone()] {
        m.ports.push(Port {
            signal: Some(sig),
            direction: port::Direction::Inout as i32,
        });
    }
    for sig in [a.clone(), b.clone(), c.clone()] {
        m.ports.push(Port {
            signal: Some(sig),
            direction: port::Direction::Input as i32,
        });
    }
    m.ports.push(Port {
        signal: Some(y.clone()),
        direction: port::Direction::Output as i32,
    });

    m.instances.push(
        Mosfet {
            name: "n1".to_string(),
            width: size.nmos_width,
            length,
            drain: sig_conn(&x1),
            source: sig_conn(&gnd),
            gate: sig_conn(&a),
            body: sig_conn(&gnd),
            mos_type: MosType::Nmos,
        }
        .into(),
    );
    m.instances.push(
        Mosfet {
            name: "n2".to_string(),
            width: size.nmos_width,
            length,
            drain: sig_conn(&x2),
            source: sig_conn(&x1),
            gate: sig_conn(&b),
            body: sig_conn(&gnd),
            mos_type: MosType::Nmos,
        }
        .into(),
    );
    m.instances.push(
        Mosfet {
            name: "n3".to_string(),
            width: size.nmos_width,
            length,
            drain: sig_conn(&y),
            source: sig_conn(&x2),
            gate: sig_conn(&c),
            body: sig_conn(&gnd),
            mos_type: MosType::Nmos,
        }
        .into(),
    );
    m.instances.push(
        Mosfet {
            name: "p1".to_string(),
            width: size.pmos_width,
            length,
            drain: sig_conn(&y),
            source: sig_conn(&vdd),
            gate: sig_conn(&a),
            body: sig_conn(&vdd),
            mos_type: MosType::Pmos,
        }
        .into(),
    );
    m.instances.push(
        Mosfet {
            name: "p2".to_string(),
            width: size.pmos_width,
            length,
            drain: sig_conn(&y),
            source: sig_conn(&vdd),
            gate: sig_conn(&b),
            body: sig_conn(&vdd),
            mos_type: MosType::Pmos,
        }
        .into(),
    );
    m.instances.push(
        Mosfet {
            name: "p3".to_string(),
            width: size.pmos_width,
            length,
            drain: sig_conn(&y),
            source: sig_conn(&vdd),
            gate: sig_conn(&c),
            body: sig_conn(&vdd),
            mos_type: MosType::Pmos,
        }
        .into(),
    );

    m
}

pub fn inv(params: &GateParams) -> Module {
    let length = params.length;
    let size = params.size;

    let gnd = signal("gnd");
    let vdd = signal("vdd");
    let din = signal("din");
    let dinb = signal("din_b");

    let ports = vec![
        Port {
            signal: Some(gnd.clone()),
            direction: port::Direction::Inout as i32,
        },
        Port {
            signal: Some(vdd.clone()),
            direction: port::Direction::Inout as i32,
        },
        Port {
            signal: Some(din.clone()),
            direction: port::Direction::Input as i32,
        },
        Port {
            signal: Some(dinb.clone()),
            direction: port::Direction::Output as i32,
        },
    ];

    let mut m = Module {
        name: params.name.clone(),
        ports,
        signals: vec![],
        instances: vec![],
        parameters: vec![],
    };

    m.instances.push(
        Mosfet {
            name: "n".to_string(),
            width: size.nmos_width,
            length,
            drain: sig_conn(&dinb),
            source: sig_conn(&gnd),
            gate: sig_conn(&din),
            body: sig_conn(&gnd),
            mos_type: MosType::Nmos,
        }
        .into(),
    );
    m.instances.push(
        Mosfet {
            name: "p".to_string(),
            width: size.pmos_width,
            length,
            drain: sig_conn(&dinb),
            source: sig_conn(&vdd),
            gate: sig_conn(&din),
            body: sig_conn(&vdd),
            mos_type: MosType::Pmos,
        }
        .into(),
    );

    m
}
