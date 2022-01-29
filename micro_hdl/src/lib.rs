use crate::context::Context;
use crate::node::Node;

pub mod backend;
pub mod context;
pub mod node;
pub mod primitive;

pub use micro_hdl_derive::*;

pub trait Module: ModuleInstance + std::any::Any {}

pub enum PinType {
    Input,
    Output,
    InOut,
}

#[derive(Clone)]
pub enum Signal {
    Wire(Node),
    Bus(Vec<Node>),
}

impl Signal {
    pub fn width(&self) -> usize {
        match self {
            Signal::Wire(_) => 1,
            Signal::Bus(v) => v.len(),
        }
    }

    pub fn nodes(&self) -> SignalNodes {
        SignalNodes { s: self, idx: 0 }
    }
}

pub struct SignalNodes<'a> {
    s: &'a Signal,
    idx: usize,
}

impl<'a> Iterator for SignalNodes<'a> {
    type Item = Node;

    fn next(&mut self) -> Option<Self::Item> {
        let node = match self.s {
            Signal::Wire(n) => {
                if self.idx == 0 {
                    Some(*n)
                } else {
                    None
                }
            }
            Signal::Bus(v) => v.get(self.idx).copied(),
        };

        self.idx += 1;

        node
    }
}

pub struct Port {
    pub name: String,
    pub pin_type: PinType,
    pub signal: Signal,
}

pub struct InstancePin {
    pub signal: Node,
}

pub trait ModuleInstance {
    fn generate(&self, c: &mut Context) -> Vec<Signal>;
    fn spice(&self) -> String;
    fn name(&self) -> String;
    fn get_ports(&self) -> Vec<Port>;
    fn config(&self) -> ModuleConfig;
}

#[derive(Debug, Eq, PartialEq)]
pub enum ModuleConfig {
    Raw,
    Generate,
}