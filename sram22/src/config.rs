use magic_vlsi::units::{Area, Distance};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

pub struct SramConfig {
    pub rows: u32,
    pub cols: u32,
    pub output_dir: String,
    pub cell_dir: String,
}

#[derive(Debug, Deserialize)]
struct TechConfigRaw {
    grid: f64,
    tech: String,
    layers: HashMap<String, LayerConfigRaw>,
}

#[derive(Debug, Deserialize)]
struct EnclosureRaw {
    layer: String,
    enclosure: f64,
    one_side: bool,
}

#[derive(Debug, Deserialize)]
struct ExtensionRaw {
    layer: String,
    extend: f64,
}

#[derive(Debug, Deserialize)]
pub struct LayerConfigRaw {
    desc: String,
    width: f64,
    space: f64,
    area: f64,
    enclosures: Option<Vec<EnclosureRaw>>,
    extensions: Option<Vec<ExtensionRaw>>,
}

#[derive(Debug)]
pub struct TechConfig {
    pub grid: Distance,
    pub tech: String,
    layers: HashMap<String, LayerConfig>,
}

#[derive(Debug)]
pub struct Enclosure {
    pub layer: String,
    pub enclosure: Distance,
    pub one_side: bool,
}

#[derive(Debug)]
pub struct Extension {
    pub layer: String,
    pub extend: Distance,
}

#[derive(Debug)]
pub struct LayerConfig {
    pub desc: String,
    pub width: Distance,
    pub space: Distance,
    pub area: Area,
    pub enclosures: Vec<Enclosure>,
    pub extensions: Vec<Extension>,
}

impl TechConfig {
    fn from_raw(raw: TechConfigRaw) -> Self {
        let mut layers = HashMap::new();
        for (layer, config) in raw.layers {
            layers.insert(layer, LayerConfig::from_raw(config));
        }
        let grid = Distance::from_nm((raw.grid * 1000.0).round() as i64);
        Self {
            grid,
            tech: raw.tech,
            layers,
        }
    }

    pub fn load(path: impl AsRef<Path>) -> Result<Self, Box<dyn std::error::Error>> {
        let txt = std::fs::read_to_string(path)?;
        let tcr: TechConfigRaw = toml::from_str(&txt)?;
        Ok(TechConfig::from_raw(tcr))
    }

    pub fn layer(&self, l: &str) -> &LayerConfig {
        self.layers.get(l).unwrap()
    }
}

impl LayerConfig {
    fn from_raw(raw: LayerConfigRaw) -> Self {
        let width = Distance::from_nm((raw.width * 1_000.0).round() as i64);
        let space = Distance::from_nm((raw.space * 1_000.0).round() as i64);
        let area = Area::from_nm2((raw.area * 1_000_000.0).round() as i64);
        Self {
            desc: raw.desc,
            width,
            space,
            area,
            enclosures: raw
                .enclosures
                .unwrap_or_default()
                .into_iter()
                .map(Enclosure::from_raw)
                .collect(),
            extensions: raw
                .extensions
                .unwrap_or_default()
                .into_iter()
                .map(Extension::from_raw)
                .collect(),
        }
    }

    pub fn extension(&self, l: &str) -> Distance {
        self.extensions
            .iter()
            .find(|ext| ext.layer == l)
            .take()
            .map(|ext| ext.extend)
            .unwrap_or_default()
    }

    fn enclosure_inner(&self, l: &str, one_sided: bool) -> Distance {
        let x = self
            .enclosures
            .iter()
            .filter(|enc| enc.layer == l)
            .collect::<Vec<_>>();
        if x.is_empty() {
            return Distance::zero();
        }

        if one_sided {
            x.into_iter().map(|x| x.enclosure).max().unwrap()
        } else {
            x.into_iter()
                .filter(|x| !x.one_side)
                .map(|x| x.enclosure)
                .max()
                .unwrap()
        }
    }

    pub fn enclosure(&self, l: &str) -> Distance {
        self.enclosure_inner(l, false)
    }

    pub fn one_side_enclosure(&self, l: &str) -> Distance {
        self.enclosure_inner(l, true)
    }
}

impl Enclosure {
    fn from_raw(raw: EnclosureRaw) -> Self {
        let enc = Distance::from_nm((raw.enclosure * 1000.0).round() as i64);
        Self {
            layer: raw.layer,
            enclosure: enc,
            one_side: raw.one_side,
        }
    }
}

impl Extension {
    fn from_raw(raw: ExtensionRaw) -> Self {
        let ext = Distance::from_nm((raw.extend * 1000.0).round() as i64);
        Self {
            layer: raw.layer,
            extend: ext,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_sky130_design_rules() -> Result<(), Box<dyn std::error::Error>> {
        let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        p.push("../tech/sky130/drc_config.toml");
        let tc = TechConfig::load(p)?;

        println!("loaded config {:?}", tc);

        assert_eq!(&tc.tech, "sky130A");
        assert_eq!(tc.layer("poly").extension("ndiff").nm(), 130);
        assert_eq!(tc.layer("poly").extension("pdiff").nm(), 130);

        assert_eq!(tc.layer("poly").extension("pdiff").nm(), 130);

        assert_eq!(tc.layer("licon").enclosure("poly").nm(), 50);
        assert_eq!(tc.layer("licon").one_side_enclosure("poly").nm(), 80);

        Ok(())
    }
}
