use std::{borrow::Cow, fmt, str::FromStr};

use geoutils::Location;
use serde::Deserialize;

#[derive(Clone, Debug, bitcode::Encode, bitcode::Decode)]
pub struct Airport {
    pub ident: String,
    pub kind: String,
    pub name: String,
    pub elevation_ft: Option<i32>,
    pub continent: String,
    pub iso_country: String,
    pub iso_region: String,
    pub municipality: String,
    pub gps_code: String,
    pub iata_code: String,
    pub local_code: String,
    pub coordinates: Coords,
    pub runways: Vec<Runway>,
}

impl Airport {
    pub fn from_template(template: AirportTemplate) -> Option<Self> {
        let AirportTemplate {
            ident,
            kind,
            name,
            elevation_ft,
            continent,
            iso_country,
            iso_region,
            municipality,
            gps_code,
            iata_code,
            local_code,
            latitude_deg,
            longitude_deg,
        } = template;

        Some(Airport {
            ident,
            kind,
            name,
            elevation_ft,
            continent,
            iso_country,
            iso_region,
            municipality,
            gps_code,
            iata_code,
            local_code,
            coordinates: Coords {
                latitude: latitude_deg,
                longitude: longitude_deg,
            },
            runways: Default::default(),
        })
    }
}

impl fmt::Display for Airport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.elevation_ft {
            Some(elevation) => write!(
                f,
                "{} {} ({} feet)\n  {}\n  {}\n  {}",
                self.ident,
                self.name,
                elevation,
                self.municipality,
                self.iso_region,
                self.coordinates
            )?,

            None => write!(
                f,
                "{} {}\n  {}\n  {}\n  {}",
                self.ident, self.name, self.municipality, self.iso_region, self.coordinates
            )?,
        };

        if !self.runways.is_empty() {
            f.write_str("\n\nRunways:\n")?;
            for rwy in &self.runways {
                let name = &rwy.name;
                let length = rwy
                    .length
                    .map(|length| Cow::from(length.to_string() + "ft"))
                    .unwrap_or_else(|| Cow::from("unknown"));

                if rwy.is_lighted {
                    writeln!(f, "  {name} {length:>8}  +L")?;
                } else {
                    writeln!(f, "  {name} {length:>8}")?;
                }
            }
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct AirportTemplate {
    ident: String,
    #[serde(rename = "type")]
    kind: String,
    name: String,
    elevation_ft: Option<i32>,
    continent: String,
    iso_country: String,
    iso_region: String,
    municipality: String,
    gps_code: String,
    iata_code: String,
    local_code: String,
    latitude_deg: f64,
    longitude_deg: f64,
}

#[derive(Clone, Copy, Debug, bitcode::Encode, bitcode::Decode)]
pub struct Coords {
    pub latitude: f64,
    pub longitude: f64,
}

impl Coords {
    pub fn location(&self) -> Location {
        let &Coords {
            latitude,
            longitude,
        } = self;
        Location::new(latitude, longitude)
    }
}

impl fmt::Display for Coords {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let n = if self.latitude >= 0.0 { "N" } else { "S" };
        let e = if self.longitude >= 0.0 { "E" } else { "W" };

        let lat = self.latitude.abs();
        let lon = self.longitude.abs();

        write!(f, "{lat:.04}°{n} {lon:.04}°{e}")
    }
}

impl FromStr for Coords {
    type Err = ParseCoordsError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let point = latlon::parse(s).map_err(|_| ParseCoordsError)?;
        Ok(Self {
            latitude: point.y(),
            longitude: point.x(),
        })
    }
}

#[derive(Debug)]
pub struct ParseCoordsError;

impl fmt::Display for ParseCoordsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("bad coordinate format")
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct RunwayTemplate {
    airport_ident: String,
    length_ft: Option<i32>,
    lighted: i8,
    closed: i8,

    /// runway identifier, e.g. 34L, where le and he are inverse
    le_ident: String,

    /// runway identifier, e.g. 34L, where le and he are inverse
    he_ident: String,
}

#[derive(Clone, Debug, bitcode::Encode, bitcode::Decode)]
pub struct Runway {
    pub airport: String,
    pub name: String,
    pub length: Option<i32>,
    pub is_closed: bool,
    pub is_lighted: bool,
}

impl From<RunwayTemplate> for Runway {
    fn from(template: RunwayTemplate) -> Self {
        let RunwayTemplate {
            airport_ident,
            length_ft,
            lighted,
            closed,
            le_ident,
            he_ident,
        } = template;

        Self {
            airport: airport_ident,
            name: format!("{le_ident}/{he_ident}"),
            length: length_ft,
            is_closed: closed == 1,
            is_lighted: lighted == 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Coords;

    fn assert_coords(input: &str, expected_lat: f64, expected_lng: f64) {
        let coords = input
            .parse::<Coords>()
            .unwrap_or_else(|e| panic!("failed to parse {input:?}: {e}"));

        let lat_diff = (coords.latitude - expected_lat).abs();
        assert!(
            lat_diff < 1e-6,
            "latitude mismatch for {input:?}: got {}, want {} (diff {lat_diff})",
            coords.latitude,
            expected_lat,
        );

        let lng_diff = (coords.longitude - expected_lng).abs();
        assert!(
            lng_diff < 1e-6,
            "longitude mismatch for {input:?}: got {}, want {} (diff {lng_diff})",
            coords.longitude,
            expected_lng,
        );
    }

    #[test]
    fn parse_decimal_degrees() {
        assert_coords("40.6413 -73.7781", 40.6413, -73.7781);
    }

    #[test]
    fn parse_dms_with_hemisphere_suffix() {
        assert_coords("40° 26′ 46″ N 79° 58′ 56″ W", 40.446111, -79.982222);
    }

    #[test]
    fn parse_dms_with_hemisphere_prefix_southern_eastern() {
        assert_coords("S 33° 51′ 31″ E 151° 12′ 37″", -33.858611, 151.210278);
    }

    #[test]
    fn parse_dms_with_negative_degrees() {
        assert_coords("40° 26′ 46″ -79° 58′ 56″", 40.446111, -79.982222);
    }

    #[test]
    fn parse_dms_ascii_without_degree_symbol() {
        assert_coords("40 26' 46\" N 79 58' 56\" W", 40.446111, -79.982222);
    }
}
